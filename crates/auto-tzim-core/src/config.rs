//! Configuration data model. Validation and persistence are not implemented yet.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Schema-one settings; a missing installation section is invalid.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub schema: u32,
    pub installation: Installation,
    #[serde(default)]
    pub folder: Vec<Folder>,
}

/// Application-wide settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
        let errors = validate_document("schema = 1\n[installation]\n[[folder]]\nroot = '/x'\nname = 'x'\nrecurse = true\nenabled = 3\njobs = 2\n");
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
