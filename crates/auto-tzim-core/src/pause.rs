//! Durable pause flag for future runs.

use crate::config::{self, ConfigError};
use std::fs::{self, File, OpenOptions};
use std::io::{self, ErrorKind};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

pub fn is_paused(state: &Path) -> io::Result<bool> {
    match fs::symlink_metadata(state.join("paused")) {
        Ok(metadata) if metadata.is_file() && metadata.len() == 0 => Ok(true),
        Ok(_) => Err(io::Error::other("invalid pause marker")),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

pub fn set(state: &Path, paused: bool) -> Result<(), ConfigError> {
    if paused {
        config::create_private_directory(state)?;
    }
    let path = state.join("paused");
    if paused {
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)
        {
            Ok(file) => file.sync_all()?,
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {
                is_paused(state)?;
            }
            Err(error) => return Err(error.into()),
        }
    } else {
        if !is_paused(state)? {
            return Ok(());
        }
        match fs::remove_file(&path) {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    File::open(state)?.sync_all()?;
    Ok(())
}
