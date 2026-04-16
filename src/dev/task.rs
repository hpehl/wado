use console::style;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::time::Duration;

const ERROR_BUFFER_CAPACITY: usize = 20;
pub(super) const NAME_WIDTH: usize = 20;

pub(super) struct DevTask {
    pub(super) name: String,
    spinner: ProgressBar,
    pub(super) error_buffer: VecDeque<String>,
    pub(super) log_path: Option<PathBuf>,
    pub(super) finished: bool,
}

impl DevTask {
    pub(super) fn new(multi: &MultiProgress, name: &str) -> Self {
        let spinner = multi.add(
            ProgressBar::new_spinner().with_style(
                ProgressStyle::default_spinner()
                    .tick_strings(&[
                        "\u{280b}", "\u{2819}", "\u{2839}", "\u{2838}", "\u{283c}", "\u{2834}",
                        "\u{2826}", "\u{2827}", "\u{2807}", "\u{280f}", " ",
                    ])
                    .template("  {spinner:.dim.bold} {wide_msg}")
                    .expect("Invalid spinner template"),
            ),
        );
        spinner.enable_steady_tick(Duration::from_millis(100));
        spinner.set_message(style(name).cyan().to_string());

        DevTask {
            name: name.to_string(),
            spinner,
            error_buffer: VecDeque::with_capacity(ERROR_BUFFER_CAPACITY),
            log_path: None,
            finished: false,
        }
    }

    pub(super) fn append_line(&mut self, line: &str) {
        if self.error_buffer.len() >= ERROR_BUFFER_CAPACITY {
            self.error_buffer.pop_front();
        }
        self.error_buffer.push_back(line.to_string());
    }

    pub(super) fn set_progress(&self, msg: &str) {
        let padded = format!("{:<width$}", self.name, width = NAME_WIDTH);
        self.spinner
            .set_message(format!("{} {}", style(padded).cyan(), style(msg).dim()));
    }

    pub(super) fn finish_success(&mut self, detail: Option<&str>) {
        self.finished = true;
        self.spinner.set_style(
            ProgressStyle::default_spinner()
                .template("  {wide_msg}")
                .expect("Invalid template"),
        );
        let padded = format!("{:<width$}", self.name, width = NAME_WIDTH);
        let msg = match detail {
            Some(d) => format!(
                "{} {} {}",
                style("\u{2713}").green().bold(),
                style(padded).cyan(),
                style(d).dim()
            ),
            None => format!(
                "{} {}",
                style("\u{2713}").green().bold(),
                style(padded).cyan()
            ),
        };
        self.spinner.finish_with_message(msg);
    }

    pub(super) fn finish_error(&mut self, err: &str) {
        self.finished = true;
        self.spinner.set_style(
            ProgressStyle::default_spinner()
                .template("  {wide_msg}")
                .expect("Invalid template"),
        );
        let padded = format!("{:<width$}", self.name, width = NAME_WIDTH);
        self.spinner.abandon_with_message(format!(
            "{} {} {}",
            style("\u{2717}").red().bold(),
            style(padded).cyan(),
            style(err).red()
        ));
    }

    pub(super) fn print_errors(&self) {
        if self.error_buffer.is_empty() && self.log_path.is_none() {
            return;
        }
        println!("\n  {} errors:", style(&self.name).cyan().bold());
        for line in &self.error_buffer {
            println!("    {}", style(line).dim());
        }
        if let Some(log_path) = &self.log_path {
            println!("    {} {}", style("full log:").dim(), log_path.display());
        }
    }
}

impl Drop for DevTask {
    fn drop(&mut self) {
        if !self.finished {
            self.finish_error("cancelled");
        }
    }
}
