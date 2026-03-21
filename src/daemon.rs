use std::time::Duration;
use tokio::process::Command;

pub async fn run() {
    let mut last = String::new();
    loop {
        tokio::time::sleep(Duration::from_millis(500)).await;

        let Ok(output) = Command::new("wl-paste").args(["-n"]).output().await else {
            continue;
        };
        if !output.status.success() {
            continue;
        }
        let Ok(content) = String::from_utf8(output.stdout) else {
            continue; // skip binary / image clipboard content
        };
        if content.is_empty() || content == last {
            continue;
        }
        last = content.clone();
        crate::history::add(content);
    }
}
