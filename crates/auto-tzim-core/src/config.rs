//! Configuration model, validation and atomic storage.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Schema-one settings; a missing installation section is invalid.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub schema: u32,
    pub installation: Installation,
    #[serde(default)]
    pub folder: Vec<Folder>,
}

/// Application-wide settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Installation {
    pub tzim_path: String,
    pub mode: String,
    pub update_policy: String,
    pub allowed_major_versions: Vec<u32>,
}

impl Default for Installation {
    fn default() -> Self {
        Self {
            tzim_path: String::new(),
            mode: "confirm".into(),
            update_policy: "notify".into(),
            allowed_major_versions: vec![0],
        }
    }
}

/// One named folder profile, with the schema defaults applied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Folder {
    pub name: String,
    pub root: PathBuf,
    #[serde(default = "enabled_by_default")]
    pub enabled: bool,
    #[serde(default)]
    pub recursive: bool,
    #[serde(default)]
    pub include_hidden: bool,
    #[serde(default)]
    pub cross_filesystems: bool,
    #[serde(default)]
    pub follow_symlinks: bool,
    #[serde(default)]
    pub allow_hard_links: bool,
    #[serde(default)]
    pub allow_cloud_sync: bool,
    #[serde(default)]
    pub formats: Vec<String>,
    #[serde(default)]
    pub transforms: Vec<String>,
    #[serde(default = "default_excludes")]
    pub exclude_globs: Vec<String>,
    #[serde(default = "default_min_age")]
    pub min_file_age_seconds: u64,
    #[serde(default = "default_jobs")]
    pub jobs: u32,
    #[serde(default = "default_min_free")]
    pub min_free_bytes: u64,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u32,
}

fn enabled_by_default() -> bool {
    true
}
fn default_excludes() -> Vec<String> {
    vec![".git/**".into(), ".hg/**".into(), ".svn/**".into()]
}
fn default_min_age() -> u64 {
    60
}
fn default_jobs() -> u32 {
    1
}
fn default_min_free() -> u64 {
    268_435_456
}
fn default_timeout() -> u32 {
    900
}

impl Default for Folder {
    fn default() -> Self {
        Self {
            name: String::new(),
            root: PathBuf::new(),
            enabled: true,
            recursive: false,
            include_hidden: false,
            cross_filesystems: false,
            follow_symlinks: false,
            allow_hard_links: false,
            allow_cloud_sync: false,
            formats: Vec::new(),
            transforms: Vec::new(),
            exclude_globs: vec![".git/**".into(), ".hg/**".into(), ".svn/**".into()],
            min_file_age_seconds: 60,
            jobs: 1,
            min_free_bytes: 268_435_456,
            timeout_seconds: 900,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn defaults_match_schema_reference() {
        let config: Config = toml::from_str(
            "schema = 1\n[installation]\n[[folder]]\nname = 'photos'\nroot = '/photos'\n",
        )
        .unwrap();
        assert_eq!(config.installation.mode, "confirm");
        assert_eq!(config.installation.update_policy, "notify");
        assert_eq!(config.installation.allowed_major_versions, [0]);
        let folder = &config.folder[0];
        assert!(folder.enabled);
        assert!(!folder.recursive);
        assert_eq!(folder.min_file_age_seconds, 60);
        assert_eq!(folder.jobs, 1);
        assert_eq!(folder.min_free_bytes, 268_435_456);
        assert_eq!(folder.timeout_seconds, 900);
        assert_eq!(folder.exclude_globs, [".git/**", ".hg/**", ".svn/**"]);
    }
}

/// Inspect independent fields without stopping at the first semantic error.
/// Root and filesystem validation is not yet implemented.
pub fn validate_document(input: &str) -> Vec<String> {
    let document = match input.parse::<toml::Value>() {
        Ok(value) => value,
        Err(error) => return vec![format!("invalid TOML: {error}")],
    };
    let Some(top) = document.as_table() else {
        return vec!["config must be a table".into()];
    };
    let mut errors = Vec::new();
    unknown_fields(top, "", &["schema", "installation", "folder"], &mut errors);
    match top.get("schema").and_then(toml::Value::as_integer) {
        Some(1) => (),
        _ => errors.push("schema: expected supported schema 1".into()),
    }
    match top.get("installation") {
        Some(toml::Value::Table(table)) => {
            unknown_fields(
                table,
                "installation",
                &[
                    "tzim_path",
                    "mode",
                    "update_policy",
                    "allowed_major_versions",
                ],
                &mut errors,
            );
            if table
                .get("mode")
                .and_then(toml::Value::as_str)
                .is_some_and(|v| v != "confirm")
            {
                errors.push("installation.mode: default must be confirm".into());
            }
            if table
                .get("update_policy")
                .and_then(toml::Value::as_str)
                .is_some_and(|v| v != "notify")
            {
                errors.push("installation.update_policy: scheduled is not in v0.1".into());
            }
            if let Some(value) = table.get("tzim_path").and_then(toml::Value::as_str) {
                let path = std::path::Path::new(value);
                if !value.is_empty() && (!path.is_absolute() || path.is_dir()) {
                    errors.push(
                        "installation.tzim_path: must be absolute and not a directory".into(),
                    );
                }
            }
            for key in ["tzim_path", "mode", "update_policy"] {
                if table.get(key).is_some_and(|v| !v.is_str()) {
                    errors.push(format!("installation.{key}: expected string"));
                }
            }
            if table.get("allowed_major_versions").is_some_and(|v| {
                v.as_array().map_or(true, |a| {
                    a.iter().any(|v| {
                        v.as_integer()
                            .map_or(true, |n| !(0..=u32::MAX as i64).contains(&n))
                    })
                })
            }) {
                errors.push(
                    "installation.allowed_major_versions: expected array of nonnegative integers"
                        .into(),
                );
            }
        }
        _ => errors.push("installation: required table".into()),
    }
    match top.get("folder") {
        None => (),
        Some(toml::Value::Array(folders)) => {
            for (index, folder) in folders.iter().enumerate() {
                let path = format!("folder[{index}]");
                if let Some(table) = folder.as_table() {
                    validate_folder_fields(table, &path, &mut errors);
                } else {
                    errors.push(format!("{path}: expected table"));
                }
            }
        }
        _ => errors.push("folder: expected array of tables".into()),
    }
    errors
}

fn unknown_fields(
    table: &toml::map::Map<String, toml::Value>,
    prefix: &str,
    allowed: &[&str],
    errors: &mut Vec<String>,
) {
    for key in table.keys().filter(|key| !allowed.contains(&key.as_str())) {
        let path = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{prefix}.{key}")
        };
        let suggestion = allowed.iter().min_by_key(|name| edit_distance(key, name));
        if let Some(name) = suggestion.filter(|name| edit_distance(key, name) <= 3) {
            errors.push(format!(
                "unknown field \"{path}\": did you mean \"{name}\"?"
            ));
        } else {
            errors.push(format!("unknown field \"{path}\""));
        }
    }
}

fn edit_distance(a: &str, b: &str) -> usize {
    let mut row: Vec<usize> = (0..=b.chars().count()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut previous = row[0];
        row[0] = i + 1;
        for (j, cb) in b.chars().enumerate() {
            let old = row[j + 1];
            row[j + 1] = (row[j + 1] + 1)
                .min(row[j] + 1)
                .min(previous + usize::from(ca != cb));
            previous = old;
        }
    }
    row[b.chars().count()]
}

fn validate_folder_fields(
    table: &toml::map::Map<String, toml::Value>,
    path: &str,
    errors: &mut Vec<String>,
) {
    const FIELDS: &[&str] = &[
        "name",
        "root",
        "enabled",
        "recursive",
        "include_hidden",
        "cross_filesystems",
        "follow_symlinks",
        "allow_hard_links",
        "allow_cloud_sync",
        "formats",
        "transforms",
        "exclude_globs",
        "min_file_age_seconds",
        "jobs",
        "min_free_bytes",
        "timeout_seconds",
        "schedule",
    ];
    unknown_fields(table, path, FIELDS, errors);
    if let Some(name) = table.get("name").and_then(toml::Value::as_str) {
        if name.is_empty()
            || name.trim() != name
            || name.chars().count() > 120
            || name.chars().any(char::is_control)
        {
            errors.push(format!(
                "{path}.name: expected 1–120 characters without control or surrounding spaces"
            ));
        }
    }
    if let Some(root) = table.get("root").and_then(toml::Value::as_str) {
        let cloud = table
            .get("allow_cloud_sync")
            .and_then(toml::Value::as_bool)
            .unwrap_or(false);
        if let Err(reason) = validate_root(std::path::Path::new(root), cloud) {
            errors.push(format!("{path}.root: {reason}"));
        }
    }
    if table
        .get("min_file_age_seconds")
        .and_then(toml::Value::as_integer)
        .is_some_and(|n| !(0..=86_400).contains(&n))
    {
        errors.push(format!("{path}.min_file_age_seconds: expected 0–86400"));
    }
    if table
        .get("timeout_seconds")
        .and_then(toml::Value::as_integer)
        .is_some_and(|n| !(1..=21_600).contains(&n))
    {
        errors.push(format!("{path}.timeout_seconds: expected 1–21600"));
    }
    if let Some(formats) = table.get("formats").and_then(toml::Value::as_array) {
        for format in formats.iter().filter_map(toml::Value::as_str) {
            if !["png", "gzip", "zip", "ico", "jpeg", "tiff", "zlib"].contains(&format) {
                errors.push(format!("{path}.formats: unsupported format {format}"));
            }
        }
    }
    for key in ["name", "root"] {
        if !table.get(key).is_some_and(toml::Value::is_str) {
            errors.push(format!("{path}.{key}: required string"));
        }
    }
    for key in ["formats", "transforms", "exclude_globs"] {
        if table
            .get(key)
            .is_some_and(|v| v.as_array().map_or(true, |a| a.iter().any(|v| !v.is_str())))
        {
            errors.push(format!("{path}.{key}: expected array of strings"));
        }
    }
    for key in [
        "min_file_age_seconds",
        "jobs",
        "min_free_bytes",
        "timeout_seconds",
    ] {
        if table
            .get(key)
            .and_then(toml::Value::as_integer)
            .is_some_and(|n| {
                n < 0 || ((key == "jobs" || key == "timeout_seconds") && n > u32::MAX as i64)
            })
        {
            errors.push(format!(
                "{path}.{key}: expected nonnegative integer in range"
            ));
        }
    }

    for key in [
        "enabled",
        "recursive",
        "include_hidden",
        "cross_filesystems",
        "follow_symlinks",
        "allow_hard_links",
        "allow_cloud_sync",
    ] {
        if table.get(key).is_some_and(|value| !value.is_bool()) {
            errors.push(format!("{path}.{key}: expected boolean"));
        }
    }
    for key in [
        "min_file_age_seconds",
        "jobs",
        "min_free_bytes",
        "timeout_seconds",
    ] {
        if table.get(key).is_some_and(|value| !value.is_integer()) {
            errors.push(format!("{path}.{key}: expected integer"));
        }
    }
    if table
        .get("jobs")
        .and_then(toml::Value::as_integer)
        .is_some_and(|jobs| jobs != 1)
    {
        errors.push(format!(
            "{path}.jobs: not_in_v01: parallel files are a later stage"
        ));
    }
    for key in ["follow_symlinks", "allow_hard_links"] {
        if table.get(key).and_then(toml::Value::as_bool) == Some(true) {
            errors.push(format!(
                "{path}.{key}: not_in_v01: reserved for a later stage"
            ));
        }
    }
    if table.contains_key("schedule") {
        errors.push(format!("{path}.schedule: not_in_v01: scheduler is v0.2"));
    }
    if table
        .get("transforms")
        .and_then(toml::Value::as_array)
        .is_some_and(|values| !values.is_empty())
    {
        errors.push(format!("{path}.transforms: transforms_forbidden: structural transformations are forbidden in the MVP"));
    }
}

#[cfg(test)]
mod validation_tests {
    use super::validate_document;

    #[test]
    fn independent_errors_are_reported_together() {
        let root = std::env::current_dir().unwrap();
        let input = format!("schema = 1\n[installation]\n[[folder]]\nroot = {:?}\nname = 'x'\nrecurse = true\nenabled = 3\njobs = 2\n", root.to_str().unwrap());
        let errors = validate_document(&input);
        assert_eq!(errors.len(), 3, "{errors:?}");
        assert!(errors
            .iter()
            .any(|e| e.contains("folder[0].recurse") && e.contains("recursive")));
        assert!(errors
            .iter()
            .any(|e| e.contains("enabled") && e.contains("boolean")));
        assert!(errors
            .iter()
            .any(|e| e.contains("jobs") && e.contains("not_in_v01")));
    }
}

/// Configuration and storage failures; no error causes the core to exit the process.
#[derive(Debug)]
pub enum ConfigError {
    Invalid(Vec<String>),
    Io(std::io::Error),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(errors) => write!(f, "{}", errors.join("\n")),
            Self::Io(error) => write!(f, "configuration I/O: {error}"),
        }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            schema: 1,
            installation: Installation::default(),
            folder: Vec::new(),
        }
    }
}

/// The config file location, as selected by the operating system and environment.
pub fn config_path() -> Result<PathBuf, ConfigError> {
    if let Some(home) = std::env::var_os("AUTO_TZIM_HOME").filter(|value| !value.is_empty()) {
        return Ok(PathBuf::from(home).join("config.toml"));
    }
    let home = std::env::var_os("HOME")
        .ok_or_else(|| ConfigError::Invalid(vec!["HOME is not set".into()]))?;
    #[cfg(target_os = "macos")]
    let dir = PathBuf::from(home).join("Library/Application Support");
    #[cfg(not(target_os = "macos"))]
    let dir = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(home).join(".config"));
    Ok(dir.join("auto-tzim/config.toml"))
}

/// Reads and validates existing settings. Missing files are an empty default configuration.
pub fn load(path: &std::path::Path) -> Result<Config, ConfigError> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Config::default()),
        Err(error) => return Err(error.into()),
    };
    let mut errors = validate_document(&text);
    if !errors.is_empty() {
        return Err(ConfigError::Invalid(errors));
    }
    let config: Config = toml::from_str(&text)
        .map_err(|error| ConfigError::Invalid(vec![format!("invalid config: {error}")]))?;
    errors.extend(validate_config(&config));
    if errors.is_empty() {
        Ok(config)
    } else {
        Err(ConfigError::Invalid(errors))
    }
}

/// Validate all values that have been decoded into the schema.
pub fn validate_config(config: &Config) -> Vec<String> {
    let mut errors = Vec::new();
    if config.schema != 1 {
        errors.push("schema: expected supported schema 1".into());
    }
    if config.installation.mode != "confirm" {
        errors.push("installation.mode: default must be confirm".into());
    }
    if config.installation.update_policy != "notify" {
        errors.push(
            "installation.update_policy: only notify is supported; scheduled is not in v0.1".into(),
        );
    }
    if !config.installation.tzim_path.is_empty() {
        let path = std::path::Path::new(&config.installation.tzim_path);
        if !path.is_absolute() || path.is_dir() {
            errors.push("installation.tzim_path: must be absolute and not a directory".into());
        }
    }
    for (index, folder) in config.folder.iter().enumerate() {
        let label = format!("folder[{index}]");
        let name = &folder.name;
        if name.trim() != name
            || name.is_empty()
            || name.chars().count() > 120
            || name.chars().any(char::is_control)
        {
            errors.push(format!(
                "{label}.name: expected 1–120 characters without control or surrounding spaces"
            ));
        }
        for previous in &config.folder[..index] {
            if previous.name.to_lowercase() == name.to_lowercase() {
                errors.push(format!("{label}.name: duplicate_name"));
            }
            if previous.root == folder.root
                || matches!((std::fs::canonicalize(&previous.root), std::fs::canonicalize(&folder.root)), (Ok(a), Ok(b)) if a == b)
            {
                errors.push(format!("{label}.root: duplicate_root"));
            }
        }
        if let Err(reason) = validate_root(&folder.root, folder.allow_cloud_sync) {
            errors.push(format!("{label}.root: {reason}"));
        }
        if folder.follow_symlinks {
            errors.push(format!("{label}.follow_symlinks: not_in_v01"));
        }
        if folder.allow_hard_links {
            errors.push(format!("{label}.allow_hard_links: not_in_v01"));
        }
        if folder.jobs != 1 {
            errors.push(format!(
                "{label}.jobs: not_in_v01: parallel files are a later stage"
            ));
        }
        if !folder.transforms.is_empty() {
            errors.push(format!("{label}.transforms: transforms_forbidden"));
        }
        if folder.min_file_age_seconds > 86_400 {
            errors.push(format!("{label}.min_file_age_seconds: expected 0–86400"));
        }
        if !(1..=21_600).contains(&folder.timeout_seconds) {
            errors.push(format!("{label}.timeout_seconds: expected 1–21600"));
        }
        for format in &folder.formats {
            if !["png", "gzip", "zip", "ico", "jpeg", "tiff", "zlib"].contains(&format.as_str()) {
                errors.push(format!("{label}.formats: unsupported format {format}"));
            }
        }
    }
    errors
}

/// Refuse dangerous roots before returning the canonical path.
pub fn validate_root(root: &std::path::Path, allow_cloud_sync: bool) -> Result<PathBuf, String> {
    use std::path::Path;
    if !root.is_absolute() {
        return Err("root_not_absolute".into());
    }
    // Rebuilding from components strips trailing separators and `.` so a symlink
    // to a directory cannot be laundered by appending a slash.
    let inspected: PathBuf = root.components().collect();
    let metadata = std::fs::symlink_metadata(&inspected).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            "root_missing"
        } else {
            "root_not_a_directory"
        }
    })?;
    if metadata.file_type().is_symlink() {
        return Err("root_is_symlink".into());
    }
    if !metadata.is_dir() {
        return Err("root_not_a_directory".into());
    }
    let canonical = std::fs::canonicalize(root).map_err(|_| "root_not_a_directory")?;
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .and_then(|path| std::fs::canonicalize(path).ok());
    if canonical == Path::new("/")
        || home.as_ref() == Some(&canonical)
        || ["/usr", "/etc", "/System", "/var", "/bin", "/sbin"]
            .iter()
            .any(|p| {
                canonical == Path::new(p)
                    || std::fs::canonicalize(p).ok().as_ref() == Some(&canonical)
            })
    {
        return Err("forbidden_root".into());
    }
    if let Some(state) = std::env::var_os("AUTO_TZIM_HOME").filter(|v| !v.is_empty()) {
        let state = PathBuf::from(state);
        let state = std::fs::canonicalize(&state).unwrap_or(state);
        if canonical.starts_with(&state) || state.starts_with(&canonical) {
            return Err("state_inside_root".into());
        }
    }
    if !allow_cloud_sync {
        if let Some(home) = home {
            if let Ok(relative) = canonical.strip_prefix(home) {
                let parts: Vec<_> = relative
                    .components()
                    .map(|part| part.as_os_str().to_string_lossy().to_lowercase())
                    .collect();
                if parts.iter().any(|part| {
                    [
                        "dropbox",
                        "dropbox (personal)",
                        "dropbox (business)",
                        "google drive",
                        "googledrive",
                        "my drive",
                        "onedrive",
                        "yandex.disk",
                        "яндекс.диск",
                        "pcloud drive",
                        "megasync",
                        "nextcloud",
                        "owncloud",
                        "sync",
                    ]
                    .contains(&part.as_str())
                        || part.starts_with("onedrive - ")
                }) || parts
                    .windows(2)
                    .any(|part| part[0] == "library" && part[1] == "mobile documents")
                {
                    return Err("cloud_sync_root: use allow_cloud_sync = true to override".into());
                }
            }
        }
    }
    Ok(canonical)
}

/// Write validated settings atomically, without overwriting an invalid existing file.
#[cfg(unix)]
pub fn save(path: &std::path::Path, config: &Config) -> Result<(), ConfigError> {
    use std::io::Write;
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
    // Never replace a file that cannot be loaded and validated.
    let existing = load(path)?;
    let errors = validate_config(config);
    if !errors.is_empty() {
        return Err(ConfigError::Invalid(errors));
    }
    let contents = match std::fs::read_to_string(path) {
        Ok(original) => preserve_profiles(&original, &existing, config)?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            toml::to_string_pretty(config)
                .map_err(|error| ConfigError::Invalid(vec![error.to_string()]))?
        }
        Err(error) => return Err(error.into()),
    };
    let dir = path
        .parent()
        .ok_or_else(|| ConfigError::Invalid(vec!["configuration path has no parent".into()]))?;
    create_private_directory(dir)?;
    let mut attempt = 0_u32;
    let (temporary, mut file) = loop {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| ConfigError::Invalid(vec![error.to_string()]))?
            .as_nanos();
        let name = format!(".config.toml.{timestamp}.{attempt}.tmp");
        let temporary = dir.join(name);
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&temporary)
        {
            Ok(file) => break (temporary, file),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists && attempt < 1000 => {
                attempt += 1
            }
            Err(error) => return Err(error.into()),
        }
    };
    let result = (|| -> Result<(), ConfigError> {
        file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
        file.write_all(contents.as_bytes())?;
        file.sync_all()?;
        std::fs::rename(&temporary, path)?;
        std::fs::File::open(dir)?.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}

#[cfg(unix)]
fn create_private_directory(path: &std::path::Path) -> Result<(), ConfigError> {
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        create_private_directory(parent)?;
    }
    match std::fs::DirBuilder::new().mode(0o700).create(path) {
        Ok(()) => std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))?,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists && path.is_dir() => (),
        Err(error) => return Err(error.into()),
    }
    Ok(())
}

/// Serialize one folder profile for display.
pub fn format_folder(folder: &Folder) -> Result<String, ConfigError> {
    toml::to_string_pretty(folder).map_err(|error| ConfigError::Invalid(vec![error.to_string()]))
}

// Keep the user's original bytes outside the folder blocks being modified.
fn preserve_profiles(original: &str, old: &Config, new: &Config) -> Result<String, ConfigError> {
    if original.contains("\"\"\"") || original.contains("\'\'\'") {
        return Err(ConfigError::Invalid(vec![
            "cannot safely edit a configuration containing multiline strings".into(),
        ]));
    }
    let mut offsets = Vec::new();
    let mut offset = 0;
    for line in original.split_inclusive('\n') {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("[[") {
            if let Some((name, _)) = rest.split_once("]]") {
                if name.trim() == "folder" {
                    offsets.push(offset);
                }
            }
        }
        offset += line.len();
    }
    if offsets.len() != old.folder.len() {
        return Err(ConfigError::Invalid(vec![
            "cannot safely edit folder profiles in this TOML layout".into(),
        ]));
    }
    if old.folder.is_empty()
        && !new.folder.is_empty()
        && original
            .parse::<toml::Value>()
            .ok()
            .and_then(|v| v.get("folder").cloned())
            .is_some()
    {
        return Err(ConfigError::Invalid(vec![
            "cannot safely append to inline folder array".into(),
        ]));
    }
    let mut result = original[..offsets.first().copied().unwrap_or(original.len())].to_owned();
    let mut seen = Vec::new();
    for (index, folder) in old.folder.iter().enumerate() {
        let start = offsets[index];
        let end = offsets.get(index + 1).copied().unwrap_or(original.len());
        if let Some(next) = new.folder.iter().find(|item| item.root == folder.root) {
            seen.push(next.root.clone());
            if next == folder {
                result.push_str(&original[start..end]);
            } else {
                append_folder(&mut result, next)?;
            }
        }
    }
    for folder in &new.folder {
        if !seen.contains(&folder.root) {
            append_folder(&mut result, folder)?;
        }
    }
    Ok(result)
}

fn append_folder(text: &mut String, folder: &Folder) -> Result<(), ConfigError> {
    if !text.ends_with('\n') {
        text.push('\n');
    }
    text.push_str("\n[[folder]]\n");
    text.push_str(&format_folder(folder)?);
    Ok(())
}
