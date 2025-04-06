use console::style;
use indicatif::HumanDuration;
use tokio::time::Instant;

#[derive(Clone)]
pub struct CommandStatus {
    pub identifier: String,
    pub success: bool,
    pub error_message: String,
}

impl CommandStatus {
    pub fn success(identifier: &str) -> Self {
        CommandStatus {
            identifier: identifier.to_string(),
            success: true,
            error_message: "".to_string(),
        }
    }

    pub fn error(identifier: &str, error_message: &str) -> Self {
        CommandStatus {
            identifier: identifier.to_string(),
            success: false,
            error_message: error_message.to_string(),
        }
    }
}

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
