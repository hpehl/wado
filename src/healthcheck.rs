use crate::progress::Progress;
use reqwest::Client;
use std::time::Duration;
use tokio::time::{Instant, sleep};

const POLL_INTERVAL: Duration = Duration::from_secs(2);
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(120);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(3);

/// Polls the WildFly management interface until it responds or the timeout expires.
///
/// Tries `/health/ready` first (WildFly 17+). If that returns 404, falls back to
/// `/management` and accepts any HTTP response as proof the management interface is up.
pub async fn wait_for_healthy(management_port: u16, progress: &Progress) -> bool {
    let client = match Client::builder().timeout(REQUEST_TIMEOUT).build() {
        Ok(c) => c,
        Err(_) => return false,
    };

    let health_url = format!("http://localhost:{}/health/ready", management_port);
    let management_url = format!("http://localhost:{}/management", management_port);
    let deadline = Instant::now() + DEFAULT_TIMEOUT;
    let mut use_fallback = false;

    progress.show_progress("Waiting for server...");

    while Instant::now() < deadline {
        let healthy = if use_fallback {
            check_management(&client, &management_url).await
        } else {
            match check_health_ready(&client, &health_url).await {
                HealthResult::Healthy => true,
                HealthResult::NotFound => {
                    use_fallback = true;
                    check_management(&client, &management_url).await
                }
                HealthResult::Unavailable => false,
            }
        };

        if healthy {
            return true;
        }
        sleep(POLL_INTERVAL).await;
    }

    false
}

enum HealthResult {
    Healthy,
    NotFound,
    Unavailable,
}

async fn check_health_ready(client: &Client, url: &str) -> HealthResult {
    match client.get(url).send().await {
        Ok(response) => {
            let status = response.status().as_u16();
            if status == 200 {
                HealthResult::Healthy
            } else if status == 404 {
                HealthResult::NotFound
            } else {
                HealthResult::Unavailable
            }
        }
        Err(_) => HealthResult::Unavailable,
    }
}

async fn check_management(client: &Client, url: &str) -> bool {
    client.get(url).send().await.is_ok()
}
