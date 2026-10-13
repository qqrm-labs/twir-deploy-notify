//! Exact-content checkpoint for a single production issue; no tokens or chat IDs.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{io, path::Path};

#[derive(Default, Serialize, Deserialize)]
pub struct Progress {
    pub digest: String,
    pub parts: usize,
    pub target: String,
    pub message_ids: Vec<i64>,
}

impl Progress {
    pub fn new(posts: &[String], chat_id: &str) -> Self {
        Self {
            digest: format!(
                "{:x}",
                Sha256::digest(serde_json::to_vec(posts).expect("serializable strings"))
            ),
            parts: posts.len(),
            target: format!(
                "{:x}",
                Sha256::digest(super::generator_shared::normalize_chat_id(chat_id).as_bytes())
            ),
            message_ids: Vec::new(),
        }
    }

    pub fn load(path: &Path, posts: &[String], chat_id: &str) -> io::Result<Self> {
        let previous: Self = match std::fs::read(path) {
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(io::Error::other)?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => Self::default(),
            Err(error) => return Err(error),
        };
        if previous.message_ids.len() > previous.parts
            || previous.message_ids.iter().any(|id| *id <= 0)
        {
            return Err(io::Error::other("Invalid delivery checkpoint"));
        }
        let next = Self::new(posts, chat_id);
        if !previous.target.is_empty() && previous.target != next.target {
            return Err(io::Error::other(
                "Delivery target changed; reconcile checkpoint before sending",
            ));
        }
        if previous.digest == next.digest && previous.parts == next.parts {
            return Ok(previous);
        }
        if previous.message_ids.len() != previous.parts {
            return Err(io::Error::other(
                "Unfinished issue differs from current posts; reconcile before sending",
            ));
        }
        Ok(next)
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        crate::checkpoint::save(path, &serde_json::to_vec(self).map_err(io::Error::other)?)
    }

    pub fn confirm(&self, path: &Path) -> io::Result<()> {
        crate::checkpoint::confirm(path, &serde_json::to_vec(self).map_err(io::Error::other)?)
    }

    pub fn complete(&self) -> bool {
        self.parts > 0 && self.message_ids.len() == self.parts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resumes_exact_parts_and_rejects_changed_partial_issue_or_target() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("progress.json");
        let posts = vec!["one".into(), "two".into()];
        let mut progress = Progress::load(&path, &posts, "42").unwrap();
        progress.message_ids.push(42);
        progress.save(&path).unwrap();
        assert_eq!(
            Progress::load(&path, &posts, "42").unwrap().message_ids,
            vec![42]
        );
        assert!(Progress::load(&path, &["changed".into()], "42").is_err());
        assert!(Progress::load(&path, &posts, "different target").is_err());
        progress.message_ids.push(43);
        progress.save(&path).unwrap();
        assert!(Progress::load(&path, &posts, "42").unwrap().complete());
        assert!(
            Progress::load(&path, &["next issue".into()], "42")
                .unwrap()
                .message_ids
                .is_empty()
        );
        assert!(!std::fs::read_to_string(path).unwrap().contains("one"));
    }

    #[test]
    fn unreadable_or_corrupt_checkpoint_fails_closed() {
        let dir = tempfile::tempdir().unwrap();
        assert!(Progress::load(dir.path(), &["post".into()], "42").is_err());
        let path = dir.path().join("broken.json");
        std::fs::write(&path, "not json").unwrap();
        assert!(Progress::load(&path, &["post".into()], "42").is_err());
    }
}
