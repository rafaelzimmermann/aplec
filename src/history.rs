use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_ENTRIES: usize = 50;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipEntry {
    pub content: String,
    pub timestamp: u64,
}

fn history_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".local/share/aplec/history.json")
}

pub fn load() -> Vec<ClipEntry> {
    std::fs::read_to_string(history_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(entries: &[ClipEntry]) {
    let path = history_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(entries) {
        let _ = std::fs::write(path, json);
    }
}

pub fn add(content: String) {
    let mut entries = load();
    entries.retain(|e| e.content != content);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    entries.insert(0, ClipEntry { content, timestamp });
    entries.truncate(MAX_ENTRIES);
    save(&entries);
}

#[cfg(test)]
mod tests {
    use super::*;

    // HOME is a global env var — tests that mutate it must not run in parallel.
    static HOME_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn with_tmp_home<F: FnOnce()>(tag: &str, f: F) {
        let _guard = HOME_LOCK.lock().unwrap();
        let tmp = std::env::temp_dir().join(format!("aplec-test-{}", tag));
        std::fs::create_dir_all(&tmp).unwrap();
        std::env::set_var("HOME", &tmp);
        f();
    }

    #[test]
    fn test_add_dedup_moves_to_top() {
        with_tmp_home("dedup", || {
            add("hello".into());
            add("world".into());
            add("hello".into());

            let entries = load();
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].content, "hello");
            assert_eq!(entries[1].content, "world");
        });
    }

    #[test]
    fn test_add_truncates_at_50() {
        with_tmp_home("trunc", || {
            for i in 0..51 {
                add(format!("item-{}", i));
            }
            let entries = load();
            assert_eq!(entries.len(), MAX_ENTRIES);
        });
    }

    #[test]
    fn test_load_missing_file_returns_empty() {
        with_tmp_home("missing", || {
            let entries = load();
            assert!(entries.is_empty());
        });
    }
}
