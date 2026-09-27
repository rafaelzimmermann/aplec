use std::time::Duration;
use tokio::process::Command;

/// Discover candidate Wayland socket names.
///
/// Prefers `$WAYLAND_DISPLAY` (imported by most session launchers) and falls
/// back to scanning `$XDG_RUNTIME_DIR` for `wayland-*` sockets: needed when
/// the daemon is started by systemd before the compositor imports its
/// environment (e.g. Hyprland 0.56's `start-hyprland` launcher, which does
/// not activate `graphical-session.target`). Re-evaluated on every poll so a
/// socket that appears after startup is picked up without a service restart.
fn wayland_candidates() -> Vec<String> {
    if let Ok(name) = std::env::var("WAYLAND_DISPLAY") {
        if !name.is_empty() {
            return vec![name];
        }
    }

    let mut sockets: Vec<String> = Vec::new();
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        if let Ok(entries) = std::fs::read_dir(&runtime_dir) {
            sockets = entries
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .filter(|name| name.starts_with("wayland-") && !name.ends_with(".lock"))
                .collect();
            sockets.sort();
        }
    }
    sockets
}

pub async fn run() {
    let mut last = String::new();
    let mut logged_socket = String::new();
    let mut preferred: Option<String> = None;
    loop {
        tokio::time::sleep(Duration::from_millis(500)).await;

        let candidates = wayland_candidates();
        if candidates.is_empty() {
            continue; // no compositor yet; retry
        }
        // Try the socket that worked last time first, then the rest.
        let mut order = candidates.clone();
        if let Some(pref) = &preferred {
            order.retain(|c| c != pref);
            order.insert(0, pref.clone());
        }

        let mut connected: Option<(String, std::process::Output)> = None;
        for socket in &order {
            if let Ok(output) = Command::new("wl-paste")
                .args(["-n"])
                .env("WAYLAND_DISPLAY", socket)
                .output()
                .await
            {
                if output.status.success() {
                    connected = Some((socket.clone(), output));
                    break;
                }
            }
        }

        let Some((socket, output)) = connected else {
            preferred = None;
            continue;
        };
        preferred = Some(socket.clone());
        if socket != logged_socket {
            eprintln!("aplec: using Wayland socket {socket}");
            logged_socket = socket;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wayland_candidates_never_include_lock_files() {
        // The live session has at least one wayland-* socket next to its
        // .lock file; the scan must never return the lock file itself.
        for name in wayland_candidates() {
            assert!(name.starts_with("wayland-"));
            assert!(!name.ends_with(".lock"));
        }
    }
}
