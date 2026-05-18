//! Progress bar and status reporting for long-running container operations.
//!
//! Wraps [`indicatif`] spinners to show per-container progress during builds,
//! pushes, starts, and stops. Also provides summary output with success/failure counts.

use crate::constants::FQN_LENGTH;
use console::{style, truncate_str};
use indicatif::{HumanDuration, MultiProgress, ProgressBar, ProgressStyle};
use std::process::Output;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader, Lines};
use tokio::process::{Child, ChildStderr, ChildStdout};
use tokio::time::Instant;

// ------------------------------------------------------ command status

/// Outcome of a container command (build, push, start, or stop).
#[derive(Clone)]
pub struct CommandStatus {
    pub identifier: String,
    pub success: bool,
    pub error_message: String,
    pub http: Option<u16>,
    pub management: Option<u16>,
}

impl CommandStatus {
    /// Creates a successful status for the given identifier.
    pub fn success(identifier: &str) -> Self {
        CommandStatus {
            identifier: identifier.to_string(),
            success: true,
            error_message: "".to_string(),
            http: None,
            management: None,
        }
    }

    /// Creates a failed status with an error message.
    pub fn error(identifier: &str, error_message: &str) -> Self {
        CommandStatus {
            identifier: identifier.to_string(),
            success: false,
            error_message: error_message.to_string(),
            http: None,
            management: None,
        }
    }

    /// Returns a new status with HTTP and management port information.
    pub fn with_ports(self, http: u16, management: u16) -> Self {
        CommandStatus {
            http: Some(http),
            management: Some(management),
            ..self
        }
    }

    /// Returns a new status marking a health check timeout failure.
    pub fn with_health_failure(self) -> Self {
        CommandStatus {
            success: false,
            error_message: "Health check timed out".to_string(),
            ..self
        }
    }
}

/// Prints a colored summary line showing how many operations succeeded/failed and elapsed time.
pub fn summary(verb: &str, noun: &str, count: usize, instant: Instant, status: Vec<CommandStatus>) {
    let successful = status.iter().filter(|&bs| bs.success).count();
    let failed = status.iter().filter(|&bs| !bs.success).count();
    println!("\n");
    if failed > 0 {
        println!(
            "{} {} of {} {} in {}. {} container failed:",
            verb,
            style(successful).green(),
            style(count).cyan(),
            noun,
            style(HumanDuration(instant.elapsed())).cyan(),
            style(failed).red()
        );
        for cs in status {
            if !cs.success {
                println!(
                    "{}: {}",
                    style(cs.identifier).cyan(),
                    style(cs.error_message).red()
                );
            }
        }
    } else {
        println!(
            "{} {} {} in {}.",
            verb,
            style(successful).cyan(),
            noun,
            style(HumanDuration(instant.elapsed())).cyan()
        );
    }
}

// ------------------------------------------------------ progress

/// A spinner-based progress indicator for a single container operation.
#[derive(Clone)]
pub struct Progress {
    prefix: String,
    image_name: String,
    pub bar: ProgressBar,
}

impl Progress {
    /// Creates a progress spinner and adds it to a [`MultiProgress`] group.
    pub fn join(multi_progress: &MultiProgress, prefix: &str, image_name: &str) -> Progress {
        let progress = Progress {
            prefix: prefix.to_string(),
            image_name: image_name.to_string(),
            bar: Self::spinner(prefix),
        };
        progress.bar.enable_steady_tick(Duration::from_millis(100));
        multi_progress.add(progress.bar.clone());
        progress
            .bar
            .set_message(format!("{:<41}", style(image_name).cyan()));
        progress
    }

    /// Creates a standalone progress spinner (not attached to a [`MultiProgress`]).
    pub fn new(prefix: &str, image_name: &str) -> Progress {
        let progress = Progress {
            prefix: prefix.to_string(),
            image_name: image_name.to_string(),
            bar: Self::spinner(prefix).with_message(format!(
                "{:<width$}",
                style(image_name).cyan(),
                width = FQN_LENGTH
            )),
        };
        progress.bar.enable_steady_tick(Duration::from_millis(100));
        progress
    }

    /// Creates a hidden progress bar that produces no terminal output.
    pub fn hidden(prefix: &str, image_name: &str) -> Progress {
        Progress {
            prefix: prefix.to_string(),
            image_name: image_name.to_string(),
            bar: ProgressBar::hidden(),
        }
    }

    fn spinner(prefix: &str) -> ProgressBar {
        ProgressBar::new_spinner()
            .with_style(
                ProgressStyle::default_spinner()
                    // https://github.com/sindresorhus/cli-spinners
                    .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", " "])
                    .template("{spinner:.dim.bold} {prefix}{wide_msg}")
                    .expect("Invalid spinner template"),
            )
            .with_prefix(format!("{:<4}   ", style(prefix).yellow()))
    }

    /// Reads lines from an async reader and updates the spinner message with each line.
    pub async fn trace_progress<R>(&self, mut reader: Lines<BufReader<R>>)
    where
        R: tokio::io::AsyncRead + Unpin,
    {
        while let Some(line) = reader
            .next_line()
            .await
            .expect("Unable to read output from command.")
        {
            self.show_progress(line.as_str());
        }
    }

    /// Updates the spinner message with the given progress text.
    pub fn show_progress(&self, progress: &str) {
        self.bar.set_message(format!(
            "{:<width$}   {}",
            style(self.image_name.clone()).cyan(),
            style(truncate_str(progress, 80, "...")).dim(),
            width = FQN_LENGTH
        ));
    }

    /// Completes the spinner based on the command output, returning a [`CommandStatus`].
    ///
    /// When `identifier` is provided, it is used as the [`CommandStatus`] identifier
    /// (e.g. the container name). Otherwise the image name is used.
    pub fn finish(
        &self,
        output: std::io::Result<Output>,
        identifier: Option<&str>,
    ) -> CommandStatus {
        let id = identifier.unwrap_or(self.image_name.as_str());
        match output {
            Ok(output) => {
                if output.status.success() {
                    self.success(identifier);
                    CommandStatus::success(id)
                } else {
                    self.error(
                        format!("Command failed with code {}", output.status.code().unwrap())
                            .as_str(),
                    );
                    CommandStatus::error(
                        id,
                        String::from_utf8_lossy(&output.stderr)
                            .replace('\n', " ")
                            .as_str(),
                    )
                }
            }
            Err(e) => {
                self.error(format!("Command failed: {}", e).as_str());
                CommandStatus::error(id, e.to_string().as_str())
            }
        }
    }

    /// Like [`finish`], but keeps the spinner alive on success so it can be
    /// reused for health-check progress. On failure, the spinner is abandoned
    /// immediately.
    pub fn finish_keep_alive(
        &self,
        output: std::io::Result<Output>,
        identifier: Option<&str>,
    ) -> CommandStatus {
        let id = identifier.unwrap_or(self.image_name.as_str());
        match output {
            Ok(output) => {
                if output.status.success() {
                    self.show_progress("Waiting for server...");
                    CommandStatus::success(id)
                } else {
                    self.error(
                        format!("Command failed with code {}", output.status.code().unwrap())
                            .as_str(),
                    );
                    CommandStatus::error(
                        id,
                        String::from_utf8_lossy(&output.stderr)
                            .replace('\n', " ")
                            .as_str(),
                    )
                }
            }
            Err(e) => {
                self.error(format!("Command failed: {}", e).as_str());
                CommandStatus::error(id, e.to_string().as_str())
            }
        }
    }

    /// Finishes all progress bars that are still alive (not yet finished).
    /// Used for containers that don't get health-checked.
    pub fn finish_if_alive(&self, identifier: Option<&str>) {
        if !self.bar.is_finished() {
            self.success(identifier);
        }
    }

    /// Completes the spinner as successful without checking command output.
    pub fn finish_no_output(&self, identifier: Option<&str>) -> CommandStatus {
        let id = identifier.unwrap_or(self.image_name.as_str());
        self.success(identifier);
        CommandStatus::success(id)
    }

    pub fn finish_healthy(&self, container_name: &str) {
        self.success(Some(container_name));
    }

    pub fn finish_unhealthy(&self) {
        self.error("health check timed out");
    }

    fn success(&self, status: Option<&str>) {
        self.bar.set_prefix(format!(
            "{:<4}   ",
            style(self.prefix.as_str()).green().bold()
        ));
        self.bar.finish_with_message(match status {
            Some(status) => format!(
                "{:<41}   {}",
                style(self.image_name.as_str()).cyan(),
                style(status).green()
            ),
            None => format!("{:<41}", style(self.image_name.as_str()).cyan()),
        });
    }

    fn error(&self, err: &str) {
        self.bar.set_prefix(format!(
            "{:<4}   ",
            style(self.prefix.as_str()).red().bold()
        ));
        self.bar.abandon_with_message(format!(
            "{:<41}   {}",
            style(self.image_name.as_str()).cyan(),
            style(err).red()
        ));
    }
}

// ------------------------------------------------------ stdout / stderr

/// Takes stdout from a child process and returns a line-buffered async reader.
pub fn stdout_reader(child: &mut Child) -> Lines<BufReader<ChildStdout>> {
    let stdout = child
        .stdout
        .take()
        .expect("Command did not have a handle to stdout.");
    BufReader::new(stdout).lines()
}

/// Takes stderr from a child process and returns a line-buffered async reader.
pub fn stderr_reader(child: &mut Child) -> Lines<BufReader<ChildStderr>> {
    let stderr = child
        .stderr
        .take()
        .expect("Command did not have a handle to stderr.");
    BufReader::new(stderr).lines()
}
