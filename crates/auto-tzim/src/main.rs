mod signals;

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
    Scan(FolderInputs),
    /// Apply an approved plan for one or more folders.
    Run {
        #[command(flatten)]
        inputs: FolderInputs,
        #[arg(long, conflicts_with = "mode")]
        auto: bool,
        #[arg(long, value_enum)]
        mode: Option<RunMode>,
        #[arg(long, value_name = "ID")]
        plan: Option<String>,
        #[arg(long = "time", value_name = "HOURS", default_value_t = 0, value_parser = clap::value_parser!(u32).range(0..=100_000))]
        cooldown_hours: u32,
        #[arg(long)]
        force: bool,
        #[arg(long, hide = true)]
        once: bool,
    },
    /// Inspect the configured binary and local state.
    Doctor {
        #[arg(long)]
        json: bool,
    },
    /// Show the pause flag, active run and latest results.
    Status {
        #[arg(long)]
        json: bool,
    },
    /// Block future apply runs.
    Pause {
        #[arg(long)]
        json: bool,
    },
    /// Allow future apply runs.
    Resume {
        #[arg(long)]
        json: bool,
    },
    /// Show recorded file outcomes, newest first.
    History {
        #[arg(long)]
        folder: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long, default_value_t = 50)]
        limit: usize,
        #[arg(long)]
        json: bool,
    },
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

#[derive(Args)]
struct FolderInputs {
    #[arg(long)]
    json: bool,
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
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum RunMode {
    Auto,
    Confirm,
}

#[derive(Subcommand)]
enum ConfigCommand {
    /// Report configuration errors.
    Validate {
        #[arg(long)]
        json: bool,
    },
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

fn execute(cli: Cli, matches: &clap::ArgMatches) -> Result<i32, ConfigError> {
    let processing = match &cli.command {
        Command::Scan(inputs) => Some(("scan", inputs, false, None, 0, false)),
        Command::Run {
            inputs,
            auto,
            mode,
            plan,
            cooldown_hours,
            force,
            ..
        } => Some((
            "run",
            inputs,
            *auto || matches!(mode, Some(RunMode::Auto)),
            plan.as_deref(),
            *cooldown_hours,
            *force,
        )),
        _ => None,
    };
    if let Some((command, options, auto, plan_id, cooldown_hours, force)) = processing {
        let apply = command == "run" && !options.dry_run;
        let matches = matches
            .subcommand_matches(command)
            .ok_or_else(|| ConfigError::Internal("missing folder arguments".into()))?;
        let inputs = scan_inputs(matches);
        let config_path = config::config_path()?;
        let (settings, config_hash) = config::load_snapshot(&config_path)?;
        let override_recursive = if options.recursive {
            Some(true)
        } else if options.no_recursive {
            Some(false)
        } else {
            None
        };
        let profiles =
            batch::resolve_inputs(inputs, &settings, override_recursive).map_err(invalid)?;
        let mut profiles = batch::preflight(profiles).map_err(invalid)?;
        if plan_id.is_some() && (profiles.len() != 1 || !apply) {
            return Err(invalid("--plan requires an apply to one folder"));
        }
        if apply && profiles.iter().any(|profile| !profile.enabled) {
            return Err(ConfigError::TreeChanged("folder_disabled".into()));
        }
        let mode = if auto {
            RunMode::Auto
        } else {
            RunMode::Confirm
        };
        let _run_lock = if apply {
            use std::io::IsTerminal;
            if matches!(mode, RunMode::Confirm)
                && (options.json
                    || !std::io::stdin().is_terminal()
                    || !std::io::stderr().is_terminal())
            {
                return Err(ConfigError::TreeChanged(
                    "confirmation requires a terminal; use --auto for unattended runs".into(),
                ));
            }
            Some(auto_tzim_core::runner::acquire_run_lock()?)
        } else {
            None
        };
        let state_home = config_path
            .parent()
            .ok_or_else(|| ConfigError::Internal("missing state directory".into()))?;
        let plans_dir = state_home.join("state/plans");
        let total = profiles.len();
        if apply {
            let state = plans_dir.parent().expect("state directory");
            let paused = auto_tzim_core::pause::is_paused(state)
                .map_err(|error| ConfigError::Internal(format!("pause state: {error}")))?;
            if paused {
                return Err(ConfigError::TreeChanged(
                    "paused: run auto-tzim resume".into(),
                ));
            }
            let cooldown = match auto_tzim_core::cooldown::Cooldown::load(state) {
                Ok(cooldown) => cooldown,
                Err(error) => {
                    eprintln!("ignoring unreadable cooldown state: {error}");
                    Default::default()
                }
            };
            let now = std::time::SystemTime::now();
            if !force {
                profiles.retain(|profile| {
                    let blocked = cooldown.blocks(&profile.root, cooldown_hours, now);
                    if blocked {
                        eprintln!("too_soon: {}", profile.root.display());
                    }
                    !blocked
                });
            }
            if profiles.is_empty() {
                return Err(ConfigError::TreeChanged(
                    "too_soon: no eligible folders".into(),
                ));
            }
        }
        let run_profiles: Vec<_> = profiles
            .iter()
            .map(|profile| (profile.name.clone(), profile.timeout_seconds))
            .collect();
        let now = std::time::SystemTime::now();
        let discovered = batch::discover(profiles, now).map_err(|error| match error {
            batch::BatchError::Invalid(message) => invalid(message),
            batch::BatchError::Discovery(message) if message.starts_with("tree_changed:") => {
                ConfigError::TreeChanged(message)
            }
            batch::BatchError::Discovery(message) => ConfigError::Internal(message),
        })?;
        if apply {
            let state = plans_dir.parent().expect("state directory");
            let available = auto_tzim_core::staging::available_bytes(&state.join("staging"))
                .map_err(|error| ConfigError::Internal(format!("staging free space: {error}")))?;
            for (folder, discovery) in &discovered {
                let peak = discovery
                    .candidates
                    .iter()
                    .map(|row| row.identity.size)
                    .max()
                    .unwrap_or(0);
                let required = u128::from(folder.min_free_bytes) + u128::from(peak);
                if u128::from(available) < required {
                    return Err(ConfigError::TreeChanged(format!(
                        "min_free_bytes: {} needs {required} bytes in staging, {available} available",
                        folder.root.display(),
                    )));
                }
            }
        }
        let identity = inspect_binary(&settings)?;
        if !apply && !options.json {
            print_identity(&identity);
        }
        let plans = auto_tzim_core::planner::build_plans(discovered, |folder, candidate| {
            Ok(auto_tzim_core::planner::arguments(folder, candidate))
        })
        .map_err(|error| ConfigError::Internal(format!("plan creation: {error:?}")))?;
        if apply {
            return apply_folders(
                &plans_dir,
                plans,
                &run_profiles,
                &identity,
                &config_hash,
                ApplyOptions {
                    mode,
                    json: options.json,
                    plan_id,
                    cooldown_hours,
                    total,
                },
            );
        }
        let mut reported_temp = false;
        for (plan, (folder, _)) in plans.iter().zip(&run_profiles) {
            let created = std::time::SystemTime::now();
            auto_tzim_core::planner::publish(&plans_dir, plan, created)
                .map_err(|e| ConfigError::Internal(format!("plan publication: {e}")))?;
            if options.json {
                write_scan_json(plan, folder, &identity, created)?;
                for diagnostic in &plan.discovery.diagnostics {
                    eprintln!("{}: {}", diagnostic.path.display(), diagnostic.reason);
                }
            } else {
                print!(
                    "{}",
                    auto_tzim_core::summary::plan_with_diagnostics(plan, !reported_temp)
                );
            }
            reported_temp |= !plan.discovery.diagnostics.is_empty();
        }
        return Ok(0);
    }

    if let Command::Status { json } = &cli.command {
        use auto_tzim_core::json::Value;
        use auto_tzim_core::{history, pause, runner};
        let path = config::config_path()?;
        let state = path
            .parent()
            .ok_or_else(|| ConfigError::Internal("missing state parent".into()))?
            .join("state");
        let internal = |error: std::io::Error| ConfigError::Internal(format!("status: {error}"));
        let paused = pause::is_paused(&state).map_err(internal)?;
        let active = runner::active_run()?;
        let records = history::status_records(&state).map_err(internal)?;
        if *json {
            let active_run = active
                .as_ref()
                .map(history::encode_value)
                .unwrap_or_else(|| "null".into());
            println!("{{\"schema\":1,\"type\":\"result\",\"command\":\"status\",\"paused\":{paused},\"active\":{},\"active_run\":{active_run},\"last_runs\":{},\"pending_plans\":{}}}", active.is_some(), history::encode_value(&Value::Array(records.last_runs)), history::encode_value(&Value::Array(records.pending_plans)));
        } else {
            println!("paused: {paused}");
            if let Some(active) = active {
                println!("active run: {}", history::encode_value(&active));
            } else {
                println!("active run: none");
            }
            for run in records.last_runs {
                let field = |name| {
                    run.member(name)
                        .map(history::encode_value)
                        .map_err(|error| ConfigError::Internal(format!("status: {error}")))
                };
                println!(
                    "{}: {} ({}) counts={}",
                    field("folder")?,
                    field("state")?,
                    field("started_at")?,
                    field("counts")?
                );
            }
            for plan in records.pending_plans {
                println!("pending plan: {}", history::encode_value(&plan));
            }
        }
        return Ok(0);
    }

    if let Command::Pause { json } | Command::Resume { json } = &cli.command {
        let paused = matches!(cli.command, Command::Pause { .. });
        let path = config::config_path()?;
        let state = path
            .parent()
            .ok_or_else(|| ConfigError::Internal("missing state parent".into()))?
            .join("state");
        auto_tzim_core::pause::set(&state, paused)
            .map_err(|error| ConfigError::Internal(format!("pause state: {error}")))?;
        if *json {
            let command = if paused { "pause" } else { "resume" };
            println!("{{\"schema\":1,\"type\":\"result\",\"command\":\"{command}\",\"paused\":{paused}}}");
        } else {
            println!("{}", if paused { "paused" } else { "resumed" });
        }
        return Ok(0);
    }

    if let Command::History {
        folder,
        status,
        limit,
        json,
    } = &cli.command
    {
        use auto_tzim_core::history;
        if let Some(status) = status {
            if !history::STATUSES.contains(&status.as_str()) {
                return Err(invalid(format!("unknown history status: {status}")));
            }
        }
        let path = config::config_path()?;
        let state = path
            .parent()
            .ok_or_else(|| ConfigError::Internal("missing state parent".into()))?
            .join("state");
        let internal = |error: std::io::Error| ConfigError::Internal(error.to_string());
        let rows = history::list(&state, folder.as_deref(), status.as_deref(), *limit)
            .map_err(internal)?;
        if *json {
            use std::io::Write;
            let mut stdout = std::io::stdout().lock();
            write!(stdout, "{{\"schema\":1,\"type\":\"history\",\"rows\":[").map_err(internal)?;
            for (index, row) in rows.iter().enumerate() {
                if index != 0 {
                    write!(stdout, ",").map_err(internal)?;
                }
                write!(stdout, "{}", row.json()).map_err(internal)?;
            }
            writeln!(stdout, "]}}").map_err(internal)?;
        } else {
            for row in rows {
                println!("{}", row.summary().map_err(internal)?);
            }
        }
        return Ok(0);
    }

    if let Command::Doctor { json } = &cli.command {
        let path = config::config_path()?;
        let settings = config::load(&path)?;
        let state = path
            .parent()
            .ok_or_else(|| ConfigError::Internal("missing state parent".into()))?
            .join("state");
        let state_status = inspect_state(&state)?;
        let identity = inspect_binary(&settings)?;
        if *json {
            use auto_tzim_core::{history::encode_value, planner::json_string};
            println!(
                "{{\"schema\":1,\"type\":\"result\",\"command\":\"doctor\",\"tzim\":{{\"path\":{},\"realpath\":{},\"sha256\":{},\"size\":{},\"version\":{},\"adapter\":{},\"capabilities_schema\":{},\"result_schema\":{},\"version_status\":\"supported\",\"capabilities\":{}}},\"platform\":{{\"os\":{},\"arch\":{}}},\"config\":{{\"path\":{},\"schema\":{},\"folders\":{}}},\"state\":{{\"path\":{},\"ok\":true,\"summary\":{}}}}}",
                json_string(&identity.path.to_string_lossy()), json_string(&identity.realpath.to_string_lossy()),
                json_string(&identity.sha256), identity.size, json_string(&identity.version),
                json_string(identity.adapter), identity.capabilities_schema, identity.result_schema,
                encode_value(&identity.capabilities), json_string(std::env::consts::OS),
                json_string(std::env::consts::ARCH), json_string(&path.to_string_lossy()),
                settings.schema, settings.folder.len(), json_string(&state.to_string_lossy()),
                json_string(&state_status)
            );
            return Ok(0);
        }
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
        return Ok(0);
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
        Command::Doctor { .. }
        | Command::History { .. }
        | Command::Status { .. }
        | Command::Pause { .. }
        | Command::Resume { .. } => {
            return Err(ConfigError::Internal("doctor dispatch failure".into()))
        }
        Command::Scan(_) | Command::Run { .. } => {
            return Err(ConfigError::Internal("scan dispatch failure".into()))
        }
        Command::Config {
            command: ConfigCommand::Validate { json },
        } => {
            let errors = config::validate_environment(&settings);
            if !errors.is_empty() {
                return Err(ConfigError::Invalid(errors));
            }
            if json {
                println!(
                    "{{\"schema\":1,\"type\":\"result\",\"command\":\"config validate\",\"valid\":true,\"config\":{{\"path\":{},\"schema\":{},\"folders\":{}}}}}",
                    auto_tzim_core::planner::json_string(&path.to_string_lossy()), settings.schema, settings.folder.len()
                );
            } else {
                println!("configuration valid");
            }
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
    Ok(0)
}

fn write_scan_json(
    plan: &auto_tzim_core::planner::Plan,
    folder: &str,
    binary: &auto_tzim_core::adapter::Identity,
    created: std::time::SystemTime,
) -> Result<(), ConfigError> {
    use auto_tzim_core::planner::{base64, json_string};
    use std::io::Write;
    use std::os::unix::ffi::OsStrExt;

    let mut stdout = std::io::stdout().lock();
    let internal = |error: std::io::Error| ConfigError::Internal(format!("scan output: {error}"));
    for row in &plan.rows {
        let path = &row.candidate.path;
        let identity = &row.candidate.identity;
        let argv = row
            .argv
            .iter()
            .map(|arg| {
                arg.to_str().map(json_string).unwrap_or_else(|| {
                    format!("{{\"b64\":{}}}", json_string(&base64(arg.as_bytes())))
                })
            })
            .collect::<Vec<_>>()
            .join(",");
        writeln!(stdout,
            "{{\"schema\":1,\"type\":\"file\",\"folder\":{},\"plan_id\":{},\"relative_path\":{},\"path_raw_b64\":{},\"status\":\"planned\",\"identity\":{{\"dev\":{},\"ino\":{},\"size\":{},\"mtime_ns\":{}}},\"argv\":[{argv}]}}",
            json_string(folder), json_string(&plan.id), json_string(&path.to_string_lossy()),
            json_string(&base64(path.as_os_str().as_bytes())), identity.dev, identity.ino, identity.size, identity.mtime_ns
        ).map_err(internal)?;
    }
    let created =
        json_string(&auto_tzim_core::planner::format_timestamp(created).map_err(internal)?);
    writeln!(stdout,
        "{{\"schema\":1,\"type\":\"plan\",\"folder\":{},\"plan_id\":{},\"plan_hash\":{},\"created\":{created},\"candidates\":{},\"excluded\":{},\"tzim\":{{\"version\":{},\"sha256\":{}}}}}",
        json_string(folder), json_string(&plan.id), json_string(&plan.hash), plan.rows.len(),
        plan.discovery.skipped.len(), json_string(&binary.version), json_string(&binary.sha256)
    ).map_err(internal)
}

struct ApplyOptions<'a> {
    mode: RunMode,
    json: bool,
    plan_id: Option<&'a str>,
    cooldown_hours: u32,
    total: usize,
}

fn apply_folders(
    plans_dir: &std::path::Path,
    mut plans: Vec<auto_tzim_core::planner::Plan>,
    profiles: &[(String, u32)],
    binary: &auto_tzim_core::adapter::Identity,
    config_hash: &str,
    options: ApplyOptions<'_>,
) -> Result<i32, ConfigError> {
    let ApplyOptions {
        mode,
        json,
        plan_id,
        cooldown_hours,
        total,
    } = options;
    use auto_tzim_core::{apply, history, planner};
    use std::io::Write;
    use std::time::{Duration, SystemTime};
    let internal = |error: std::io::Error| ConfigError::Internal(error.to_string());

    let state = plans_dir.parent().expect("plans directory has a parent");
    if let Some(id) = plan_id {
        let stale = || ConfigError::TreeChanged("stale_plan: scan the folder again".into());
        if id.len() != 32 || !id.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(stale());
        }
        let plan = &mut plans[0];
        if !planner::matches_file(plan, &plans_dir.join(format!("{id}.json")), id)
            .map_err(|_| stale())?
        {
            return Err(stale());
        }
        plan.id = id.into();
    } else {
        for plan in &plans {
            planner::publish(plans_dir, plan, SystemTime::now()).map_err(internal)?;
        }
    }

    let declined = if matches!(mode, RunMode::Confirm) {
        let mut prompt = std::io::stderr().lock();
        for plan in &plans {
            write!(
                prompt,
                "{}",
                auto_tzim_core::summary::plan_with_diagnostics(plan, true)
            )
            .map_err(internal)?;
        }
        write!(prompt, "Originals will be deleted after each replacement is verified. There is no undo.\nContinue? [y/N] ").map_err(internal)?;
        prompt.flush().map_err(internal)?;
        let mut answer = String::new();
        std::io::stdin().read_line(&mut answer).is_err()
            || !matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes")
    } else {
        false
    };

    let batch_id = (total > 1)
        .then(planner::plan_id)
        .transpose()
        .map_err(internal)?;
    let signals = signals::ApplySignals::install().map_err(internal)?;
    let mut completed = 0;
    let mut code = 0;
    for (plan, (folder, timeout)) in plans.iter().zip(profiles) {
        let metadata = history::RunMetadata {
            folder,
            mode: match (mode, declined) {
                (_, true) => history::Mode::Declined,
                (RunMode::Auto, false) => history::Mode::Auto,
                (RunMode::Confirm, false) => history::Mode::Confirm,
            },
            batch_id: batch_id.as_deref(),
            config_hash,
            cooldown_hours,
        };
        let outcome = apply::apply_plan(
            state,
            plan,
            binary,
            &metadata,
            Duration::from_secs(u64::from(*timeout)),
            &signals.cancellation,
        )
        .map_err(|error| ConfigError::Internal(format!("apply: {error:?}")))?;
        if json {
            history::write_run_json(state, &outcome.run_id, std::io::stdout().lock())
                .map_err(internal)?;
        }
        for recovery in outcome.recovered {
            eprintln!("recovered run {}: {:?}", recovery.id, recovery.outcomes);
        }
        if outcome.exit_code != 0 {
            code = i32::from(outcome.exit_code);
            if outcome.stopped_early {
                eprintln!("run {} stopped; see local history", outcome.run_id);
                break;
            }
        }
        completed += 1;
    }
    if !json {
        if declined {
            println!("declined {completed}/{total} folders");
        } else {
            println!("completed {completed}/{total} folders");
        }
    }
    Ok(signals
        .cancellation
        .requested()
        .map_or(code, |signal| i32::from(signal as u8)))
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
            && arg != "run"
            && arg != "doctor"
            && arg != "history"
            && arg != "status"
            && arg != "pause"
            && arg != "resume"
    });
    if path_first {
        args.insert(1, "run".into());
    }
    let matches = Cli::command().get_matches_from(args);
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|error| error.exit());
    match execute(cli, &matches) {
        Ok(0) => {}
        Ok(code) => std::process::exit(code),
        Err(error) => {
            let code = exit_code(&error);
            eprintln!("{error}");
            std::process::exit(code);
        }
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
