//! Configuration data model. Validation and persistence are not implemented yet.

use serde::{Deserialize, Serialize};

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
#[serde(default, deny_unknown_fields)]
pub struct Folder {
    pub name: String,
    pub root: String,
    pub enabled: bool,
    pub recursive: bool,
    pub include_hidden: bool,
    pub cross_filesystems: bool,
    pub follow_symlinks: bool,
    pub allow_hard_links: bool,
    pub allow_cloud_sync: bool,
    pub formats: Vec<String>,
    pub transforms: Vec<String>,
    pub exclude_globs: Vec<String>,
    pub min_file_age_seconds: u64,
    pub jobs: u32,
    pub min_free_bytes: u64,
    pub timeout_seconds: u32,
}

impl Default for Folder {
    fn default() -> Self {
        Self {
            name: String::new(),
            root: String::new(),
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
