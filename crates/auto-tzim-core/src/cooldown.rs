//! Per-canonical-root cooldown state. Read and update under the run lock.

use crate::{history::encode_value, json::Value, planner};
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::{ffi::OsStrExt, fs::OpenOptionsExt};
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Default)]
pub struct Cooldown(BTreeMap<String, Value>);

impl Cooldown {
    pub fn load(state: &Path) -> io::Result<Self> {
        let path = state.join("last-compression.json");
        let file = match File::open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(error) => return Err(error),
        };
        let mut bytes = Vec::new();
        file.take(crate::json::MAX_BYTES as u64 + 1)
            .read_to_end(&mut bytes)?;
        let document = crate::json::parse(&bytes).map_err(io::Error::other)?;
        if document.integer("schema").map_err(io::Error::other)? != 1 {
            return Err(io::Error::other("unknown cooldown schema"));
        }
        let Value::Object(roots) = document.member("roots").map_err(io::Error::other)? else {
            return Err(io::Error::other("invalid cooldown roots"));
        };
        for value in roots.values() {
            value.integer("at_unix_seconds").map_err(io::Error::other)?;
        }
        Ok(Self(roots.clone()))
    }

    pub fn blocks(&self, root: &Path, hours: u32, now: SystemTime) -> bool {
        let Some(entry) = self.0.get(&root_key(root)) else {
            return false;
        };
        let Ok(seconds) = entry.integer("at_unix_seconds") else {
            return false;
        };
        hours != 0
            && now.duration_since(UNIX_EPOCH).is_ok_and(|now| {
                now < Duration::from_secs(seconds.saturating_add(u64::from(hours) * 3600))
            })
    }

    pub(crate) fn record(state: &Path, root: &Path, run_id: &str, hours: u32) -> io::Result<()> {
        let mut entries = Self::load(state).unwrap_or_default().0;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?;
        entries.insert(
            root_key(root),
            Value::Object(BTreeMap::from([
                (
                    "at".into(),
                    Value::String(planner::timestamp(now.as_secs(), now.subsec_nanos())),
                ),
                (
                    "at_unix_seconds".into(),
                    Value::Number(now.as_secs().to_string()),
                ),
                ("run_id".into(), Value::String(run_id.into())),
                ("hours".into(), Value::Number(hours.to_string())),
            ])),
        );
        let body = format!(
            "{{\"schema\":1,\"roots\":{}}}\n",
            encode_value(&Value::Object(entries))
        );
        if body.len() > crate::json::MAX_BYTES {
            return Err(io::Error::other("cooldown state exceeds size limit"));
        }
        let temp = state.join("last-compression.json.next");
        match fs::remove_file(&temp) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        let mut output = OpenOptions::new()
            .create_new(true)
            .write(true)
            .mode(0o600)
            .open(&temp)?;
        output.write_all(body.as_bytes())?;
        output.sync_all()?;
        fs::rename(temp, state.join("last-compression.json"))?;
        File::open(state)?.sync_all()
    }
}

fn root_key(root: &Path) -> String {
    root.to_str()
        .map(str::to_owned)
        .unwrap_or_else(|| format!("b64:{}", planner::base64(root.as_os_str().as_bytes())))
}
