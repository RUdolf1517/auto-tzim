//! File identity and injected-time stability decisions.
use std::os::unix::fs::MetadataExt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Identity captured from the one lstat result for an entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity {
    pub dev: u64,
    pub ino: u64,
    pub size: u64,
    pub mtime_ns: i128,
    pub links: u64,
}

impl Identity {
    /// Snapshot all identity fields from metadata already obtained by the walker.
    pub fn from_metadata(metadata: &std::fs::Metadata) -> Self {
        Self {
            dev: metadata.dev(),
            ino: metadata.ino(),
            size: metadata.len(),
            mtime_ns: i128::from(metadata.mtime()) * 1_000_000_000
                + i128::from(metadata.mtime_nsec()),
            links: metadata.nlink(),
        }
    }
}

/// Report hard links before age; the caller supplies the current time.
pub fn refusal(identity: &Identity, now: SystemTime, min_age_seconds: u64) -> Option<&'static str> {
    if identity.links > 1 {
        return Some("hardlink_skip");
    }
    let now_ns = match now.duration_since(UNIX_EPOCH) {
        Ok(value) => value.as_nanos() as i128,
        Err(error) => -(error.duration().as_nanos() as i128),
    };
    let minimum = Duration::from_secs(min_age_seconds).as_nanos() as i128;
    if now_ns.saturating_sub(identity.mtime_ns) < minimum {
        Some("skipped_unstable")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn injected_clock_and_hardlink_precedence() {
        let mut id = Identity {
            dev: 1,
            ino: 2,
            size: 3,
            mtime_ns: 100_000_000_000,
            links: 2,
        };
        assert_eq!(
            refusal(&id, UNIX_EPOCH + Duration::from_secs(101), 60),
            Some("hardlink_skip")
        );
        id.links = 1;
        assert_eq!(
            refusal(&id, UNIX_EPOCH + Duration::from_secs(101), 60),
            Some("skipped_unstable")
        );
        assert_eq!(
            refusal(&id, UNIX_EPOCH + Duration::from_secs(160), 60),
            None
        );
    }
    #[test]
    fn nanosecond_boundary_and_future_times() {
        let id = Identity {
            dev: 1,
            ino: 2,
            size: 3,
            mtime_ns: 100_000_000_500,
            links: 1,
        };
        assert_eq!(
            refusal(&id, UNIX_EPOCH + Duration::new(160, 499), 60),
            Some("skipped_unstable")
        );
        assert_eq!(refusal(&id, UNIX_EPOCH + Duration::new(160, 500), 60), None);
        assert_eq!(
            refusal(&id, UNIX_EPOCH + Duration::from_secs(99), 0),
            Some("skipped_unstable")
        );
        let pre_epoch = Identity {
            mtime_ns: -1_000_000_000,
            ..id
        };
        assert_eq!(refusal(&pre_epoch, UNIX_EPOCH, 1), None);
    }
}
