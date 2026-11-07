use auto_tzim_core::batch::{self, RootInput};
use auto_tzim_core::config::{self, Config, ConfigError, Folder};
use clap::{Args, CommandFactory, FromArgMatches, Parser, Subcommand};
use std::ffi::OsString;
use std::path::PathBuf;

/// Companion orchestration tool for tzim.
#[derive(Parser)]
#[command(version = auto_tzim_core::VERSION, disable_help_subcommand = true)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Inspect folders without invoking tzim or changing their contents.
    Scan {
        #[arg(value_parser = clap::builder::ValueParser::os_string())]
        paths: Vec<OsString>,
        #[arg(long, value_name = "NAME")]
        folder: Vec<String>,
        #[arg(long, conflicts_with = "recursive")]
        no_recursive: bool,
        #[arg(long, conflicts_with = "no_recursive")]
        recursive: bool,
        #[arg(long)]
        dry_run: bool,
    },
    /// Inspect the configured binary and local state.
    Doctor,
    /// Validate the configuration.
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// Manage saved folder profiles.
    Folder {
        #[command(subcommand)]
        command: FolderCommand,
    },
}

#[derive(Subcommand)]
enum ConfigCommand {
    /// Report configuration errors.
    Validate,
}

#[derive(Args)]
struct ProfileOptions {
    #[arg(long)]
    name: Option<String>,
    #[arg(long, conflicts_with = "no_recursive")]
    recursive: bool,
    #[arg(long, conflicts_with = "recursive")]
    no_recursive: bool,
    #[arg(long)]
    formats: Option<String>,
    #[arg(long)]
    exclude: Vec<String>,
    #[arg(long, conflicts_with = "no_allow_cloud_sync")]
    allow_cloud_sync: bool,
    #[arg(long, conflicts_with = "allow_cloud_sync")]
    no_allow_cloud_sync: bool,
}

#[derive(Subcommand)]
enum FolderCommand {
    /// Add one or more folders without processing their files.
    Add {
        #[arg(required = true, value_parser = clap::builder::ValueParser::os_string())]
        paths: Vec<OsString>,
        #[command(flatten)]
        options: ProfileOptions,
    },
    /// List stored folder profiles.
    List,
    /// Show a folder profile.
    Show { name: String },
    /// Edit a folder profile.
    Edit {
        #[arg(value_name = "NAME")]
        folder_name: String,
        #[command(flatten)]
        options: ProfileOptions,
    },
    /// Enable a folder.
    Enable { name: String },
    /// Disable a folder.
    Disable { name: String },
    /// Remove a folder profile without touching its files.
    Remove { name: String },
}

fn invalid(message: impl Into<String>) -> ConfigError {
    ConfigError::Invalid(vec![message.into()])
}

fn folder_index(config: &Config, name: &str) -> Result<usize, ConfigError> {
    config
        .folder
        .iter()
        .position(|f| f.name.to_lowercase() == name.to_lowercase())
        .ok_or_else(|| invalid(format!("folder not found: {name}")))
}

fn apply_options(folder: &mut Folder, options: &ProfileOptions) {
    if let Some(name) = &options.name {
        folder.name.clone_from(name);
    }
    if options.recursive {
        folder.recursive = true;
    } else if options.no_recursive {
        folder.recursive = false;
    }
    if let Some(formats) = &options.formats {
        folder.formats = if formats.is_empty() {
            Vec::new()
        } else {
            formats.split(',').map(str::to_owned).collect()
        };
    }
    folder.exclude_globs.extend(options.exclude.iter().cloned());
    if options.allow_cloud_sync {
        folder.allow_cloud_sync = true;
    } else if options.no_allow_cloud_sync {
        folder.allow_cloud_sync = false;
    }
}

fn show(folder: &Folder) -> Result<String, ConfigError> {
    config::format_folder(folder)
}

fn scan_inputs(matches: &clap::ArgMatches) -> Vec<RootInput> {
    let mut inputs = Vec::new();
    if let (Some(indices), Some(values)) = (
        matches.indices_of("paths"),
        matches.get_many::<OsString>("paths"),
    ) {
        inputs.extend(
            indices
                .zip(values)
                .map(|(i, value)| (i, RootInput::Path(PathBuf::from(value)))),
        );
    }
    if let (Some(indices), Some(values)) = (
        matches.indices_of("folder"),
        matches.get_many::<String>("folder"),
    ) {
        inputs.extend(
            indices
                .zip(values)
                .map(|(i, value)| (i, RootInput::Folder(value.clone()))),
        );
    }
    inputs.sort_by_key(|(index, _)| *index);
    inputs.into_iter().map(|(_, input)| input).collect()
}

fn inspect_binary(settings: &Config) -> Result<auto_tzim_core::adapter::Identity, ConfigError> {
    auto_tzim_core::adapter::inspect(std::path::Path::new(&settings.installation.tzim_path))
        .map_err(|error| ConfigError::IncompatibleTzim(error.to_string()))
}

fn print_identity(identity: &auto_tzim_core::adapter::Identity) {
    println!("tzim: path={} realpath={} size={} sha256={} version={} capabilities_schema={} result_schema={} adapter={}",
        identity.path.display(), identity.realpath.display(), identity.size, identity.sha256,
        identity.version, identity.capabilities_schema, identity.result_schema, identity.adapter);
}

fn inspect_state(state: &std::path::Path) -> Result<String, ConfigError> {
    let err = |error: std::io::Error| {
        ConfigError::Internal(format!("state {}: {error}", state.display()))
    };
    match std::fs::read_dir(state) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok("not initialized".into())
        }
        Err(error) => return Err(err(error)),
        Ok(entries) => {
            for entry in entries {
                entry.map_err(err)?;
            }
        }
    }
    let schema = match std::fs::read(state.join("schema.json")) {
        Ok(bytes) => {
            let value = auto_tzim_core::json::parse(&bytes)
                .map_err(|e| ConfigError::Internal(format!("state schema: {e}")))?;
            let number = value
                .integer("schema")
                .map_err(|e| ConfigError::Internal(format!("state schema: {e}")))?;
            if number != 1 {
                return Err(ConfigError::Internal(format!(
                    "unsupported state schema: {number}"
                )));
            }
            "1"
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => "not initialized",
        Err(error) => return Err(err(error)),
    };
    let runs = state.join("runs");
    let mut tail_recovered = false;
    match std::fs::read_dir(&runs) {
        Ok(entries) => {
            for entry in entries {
                let entry = entry.map_err(err)?;
                let header = std::fs::read(entry.path().join("run.json")).map_err(err)?;
                auto_tzim_core::json::parse(&header)
                    .map_err(|e| ConfigError::Internal(format!("run header: {e}")))?;
                for name in ["files.jsonl", "journal.jsonl"] {
                    let file = entry.path().join(name);
                    let bytes = match std::fs::read(&file) {
                        Ok(bytes) => bytes,
                        Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                        Err(e) => return Err(err(e)),
                    };
                    let mut lines: Vec<_> = bytes.split(|b| *b == b'\n').collect();
                    if lines.last() == Some(&&b""[..]) {
                        lines.pop();
                    }
                    for (index, line) in lines.iter().enumerate() {
                        if let Err(error) = auto_tzim_core::json::parse(line) {
                            if index + 1 == lines.len() {
                                tail_recovered = true;
                            } else {
                                return Err(ConfigError::Internal(format!(
                                    "{}: invalid non-tail line: {error}",
                                    file.display()
                                )));
                            }
                        }
                    }
                }
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
        Err(error) => return Err(err(error)),
    }
    Ok(format!(
        "schema={schema} runs=readable tail_recovered={tail_recovered}"
    ))
}

fn execute(cli: Cli, matches: &clap::ArgMatches, path_first: bool) -> Result<(), ConfigError> {
    if let Command::Scan {
        no_recursive,
        recursive,
        dry_run,
        ..
    } = &cli.command
    {
        if path_first && !dry_run {
            return Err(invalid(
                "path-first invocation requires --dry-run at this stage",
            ));
        }
        let matches = matches
            .subcommand_matches("scan")
            .ok_or_else(|| ConfigError::Internal("missing scan arguments".into()))?;
        let inputs = scan_inputs(matches);
        let settings = config::load(&config::config_path()?)?;
        let override_recursive = if *recursive {
            Some(true)
        } else if *no_recursive {
            Some(false)
        } else {
            None
        };
        let profiles =
            batch::resolve_inputs(inputs, &settings, override_recursive).map_err(invalid)?;
        let profiles = batch::preflight(profiles).map_err(invalid)?;
        let identity = inspect_binary(&settings)?;
        print_identity(&identity);
        let now = std::time::SystemTime::now();
        let discovered = batch::discover(profiles, now).map_err(|error| match error {
            batch::BatchError::Invalid(message) => invalid(message),
            batch::BatchError::Discovery(message) if message.starts_with("tree_changed:") => {
                ConfigError::TreeChanged(message)
            }
            batch::BatchError::Discovery(message) => ConfigError::Internal(message),
        })?;
        let plans = auto_tzim_core::planner::build_plans(discovered, |folder, candidate| {
            Ok(auto_tzim_core::planner::arguments(folder, candidate))
        })
        .map_err(|error| ConfigError::Internal(format!("plan creation: {error:?}")))?;
        let config_path = config::config_path()?;
        let state_home = config_path
            .parent()
            .ok_or_else(|| ConfigError::Internal("missing state directory".into()))?;
        let plans_dir = state_home.join("state/plans");
        let mut reported_temp = false;
        for plan in &plans {
            auto_tzim_core::planner::publish(&plans_dir, plan, std::time::SystemTime::now())
                .map_err(|e| ConfigError::Internal(format!("plan publication: {e}")))?;
            print!(
                "{}",
                auto_tzim_core::summary::plan_with_diagnostics(plan, !reported_temp)
            );
            reported_temp |= !plan.discovery.diagnostics.is_empty();
        }
        return Ok(());
    }

    if matches!(&cli.command, Command::Doctor) {
        let path = config::config_path()?;
        let settings = config::load(&path)?;
        let state = path
            .parent()
            .ok_or_else(|| ConfigError::Internal("missing state parent".into()))?
            .join("state");
        let state_status = inspect_state(&state)?;
        let identity = inspect_binary(&settings)?;
        print_identity(&identity);
        println!(
            "platform: {} {}",
            std::env::consts::OS,
            std::env::consts::ARCH
        );
        println!("config: {} schema={}", path.display(), settings.schema);
        println!("state: {} {}", state.display(), state_status);
        println!(
            "capabilities (description of this binary, not a guarantee): {:?}",
            identity.capabilities
        );
        return Ok(());
    }
    let path = config::config_path()?;
    let mutating = !matches!(
        &cli.command,
        Command::Config { .. }
            | Command::Folder {
                command: FolderCommand::List | FolderCommand::Show { .. }
            }
    );
    let lock = if mutating {
        Some(config::acquire_config_lock()?)
    } else {
        None
    };
    let mut settings = config::load(&path)?;
    match cli.command {
        Command::Doctor => return Err(ConfigError::Internal("doctor dispatch failure".into())),
        Command::Scan { .. } => return Err(ConfigError::Internal("scan dispatch failure".into())),
        Command::Config {
            command: ConfigCommand::Validate,
        } => {
            let errors = config::validate_environment(&settings);
            if !errors.is_empty() {
                return Err(ConfigError::Invalid(errors));
            }
            println!("configuration valid");
        }
        Command::Folder { command } => match command {
            FolderCommand::List => {
                for folder in &settings.folder {
                    println!(
                        "{}\t{}\t{}",
                        folder.name,
                        folder.root.display(),
                        folder.enabled
                    );
                }
            }
            FolderCommand::Show { name } => {
                let i = folder_index(&settings, &name)?;
                print!("{}", show(&settings.folder[i])?);
                println!("last run and counts: not recorded yet");
            }
            FolderCommand::Add { paths, options } => {
                if paths.len() > 1 && options.name.is_some() {
                    return Err(invalid("--name requires a single path"));
                }
                let mut added = Vec::new();
                for path in paths {
                    let path = PathBuf::from(path);
                    let root =
                        config::validate_root(&path, options.allow_cloud_sync).map_err(invalid)?;
                    let name = match &options.name {
                        Some(name) => name.clone(),
                        None => root
                            .file_name()
                            .ok_or_else(|| invalid("root has no basename"))?
                            .to_str()
                            .ok_or_else(|| invalid("non-UTF-8 folder name: specify --name"))?
                            .to_owned(),
                    };
                    let mut folder = Folder {
                        name,
                        root,
                        ..Folder::default()
                    };
                    apply_options(&mut folder, &options);
                    added.push(folder.clone());
                    settings.folder.push(folder);
                }
                let errors = config::validate_config(&settings);
                if !errors.is_empty() {
                    return Err(ConfigError::Invalid(errors));
                }
                config::save(
                    &path,
                    &settings,
                    lock.as_ref().ok_or_else(|| {
                        ConfigError::Internal("mutating command without config lock".into())
                    })?,
                )?;
                for folder in &added {
                    println!("[[folder]]\n{}", show(folder)?);
                }
            }
            FolderCommand::Edit {
                folder_name,
                options,
            } => {
                let i = folder_index(&settings, &folder_name)?;
                apply_options(&mut settings.folder[i], &options);
                let errors = config::validate_config(&settings);
                if !errors.is_empty() {
                    return Err(ConfigError::Invalid(errors));
                }
                config::save(
                    &path,
                    &settings,
                    lock.as_ref().ok_or_else(|| {
                        ConfigError::Internal("mutating command without config lock".into())
                    })?,
                )?;
                print!("{}", show(&settings.folder[i])?);
            }
            FolderCommand::Enable { name } => {
                let i = folder_index(&settings, &name)?;
                settings.folder[i].enabled = true;
                config::save(
                    &path,
                    &settings,
                    lock.as_ref().ok_or_else(|| {
                        ConfigError::Internal("mutating command without config lock".into())
                    })?,
                )?;
                println!("enabled {}", settings.folder[i].name);
            }
            FolderCommand::Disable { name } => {
                let i = folder_index(&settings, &name)?;
                settings.folder[i].enabled = false;
                config::save(
                    &path,
                    &settings,
                    lock.as_ref().ok_or_else(|| {
                        ConfigError::Internal("mutating command without config lock".into())
                    })?,
                )?;
                println!("disabled {}", settings.folder[i].name);
            }
            FolderCommand::Remove { name } => {
                let i = folder_index(&settings, &name)?;
                let removed = settings.folder.remove(i);
                config::save(
                    &path,
                    &settings,
                    lock.as_ref().ok_or_else(|| {
                        ConfigError::Internal("mutating command without config lock".into())
                    })?,
                )?;
                println!("removed {}", removed.name);
            }
        },
    }
    Ok(())
}

fn exit_code(error: &ConfigError) -> i32 {
    match error {
        ConfigError::Internal(_) => 7,
        ConfigError::IncompatibleTzim(_) => 3,
        ConfigError::TreeChanged(_) => 6,
        ConfigError::Locked(_) => 8,
        ConfigError::Invalid(_) | ConfigError::Io(_) => 2,
    }
}

fn main() {
    config::record_process_start();
    let mut args: Vec<OsString> = std::env::args_os().collect();
    let path_first = args.get(1).is_some_and(|arg| {
        !arg.as_encoded_bytes().starts_with(b"-")
            && arg != "config"
            && arg != "folder"
            && arg != "scan"
            && arg != "doctor"
    });
    if path_first {
        args.insert(1, "scan".into());
    }
    let matches = Cli::command().get_matches_from(args);
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|error| error.exit());
    if let Err(error) = execute(cli, &matches, path_first) {
        let code = exit_code(&error);
        eprintln!("{error}");
        std::process::exit(code);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_parser_preserves_interleaved_inputs() {
        let matches = Cli::command()
            .try_get_matches_from([
                "auto-tzim",
                "scan",
                "/one",
                "--folder",
                "photos",
                "/two",
                "--folder",
                "work",
                "--no-recursive",
            ])
            .unwrap();
        assert_eq!(
            scan_inputs(matches.subcommand_matches("scan").unwrap()),
            vec![
                RootInput::Path("/one".into()),
                RootInput::Folder("photos".into()),
                RootInput::Path("/two".into()),
                RootInput::Folder("work".into()),
            ]
        );
        assert!(Cli::command()
            .try_get_matches_from(["auto-tzim", "scan", "--recursive", "--no-recursive", "/one"])
            .is_err());
    }

    #[test]
    fn scan_parser_preserves_non_utf8_root_bytes() {
        use std::os::unix::ffi::OsStringExt;
        let path = OsString::from_vec(b"/tmp/bad\xff".to_vec());
        let matches = Cli::command()
            .try_get_matches_from(vec![
                OsString::from("auto-tzim"),
                OsString::from("scan"),
                path.clone(),
                OsString::from("--folder"),
                OsString::from("photos"),
            ])
            .unwrap();
        assert_eq!(
            scan_inputs(matches.subcommand_matches("scan").unwrap()),
            vec![
                RootInput::Path(path.into()),
                RootInput::Folder("photos".into())
            ]
        );
    }

    #[test]
    fn internal_error_maps_to_exit_seven() {
        assert_eq!(exit_code(&ConfigError::Internal("missing lock".into())), 7);
        assert_eq!(
            exit_code(&ConfigError::Invalid(vec!["bad input".into()])),
            2
        );
        assert_eq!(exit_code(&ConfigError::Locked("holder".into())), 8);
        assert_eq!(
            exit_code(&ConfigError::TreeChanged("tree_changed".into())),
            6
        );
    }
}
