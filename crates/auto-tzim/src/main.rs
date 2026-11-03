use auto_tzim_core::config::{self, Config, ConfigError, Folder};
use clap::{Args, Parser, Subcommand};
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
    #[arg(long)]
    recursive: bool,
    #[arg(long)]
    formats: Option<String>,
    #[arg(long)]
    exclude: Vec<String>,
}

#[derive(Subcommand)]
enum FolderCommand {
    /// Add one or more folders without processing their files.
    Add {
        #[arg(required = true)]
        paths: Vec<PathBuf>,
        #[command(flatten)]
        options: ProfileOptions,
    },
    /// List stored folder profiles.
    List,
    /// Show a folder profile.
    Show { name: String },
    /// Edit a folder profile.
    Edit {
        name: String,
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
    }
    if let Some(formats) = &options.formats {
        folder.formats = if formats.is_empty() {
            Vec::new()
        } else {
            formats.split(',').map(str::to_owned).collect()
        };
    }
    folder.exclude_globs.extend(options.exclude.iter().cloned());
}

fn show(folder: &Folder) -> Result<String, ConfigError> {
    config::format_folder(folder)
}

fn execute(cli: Cli) -> Result<(), ConfigError> {
    let path = config::config_path()?;
    let mut settings = config::load(&path)?;
    match cli.command {
        Command::Config {
            command: ConfigCommand::Validate,
        } => println!("configuration valid"),
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
                    let root = config::validate_root(&path, false).map_err(invalid)?;
                    let name = match &options.name {
                        Some(name) => name.clone(),
                        None => root
                            .file_name()
                            .ok_or_else(|| invalid("root has no basename"))?
                            .to_string_lossy()
                            .into_owned(),
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
                config::save(&path, &settings)?;
                for folder in &added {
                    println!("[[folder]]\n{}", show(folder)?);
                }
            }
            FolderCommand::Edit { name, options } => {
                let i = folder_index(&settings, &name)?;
                apply_options(&mut settings.folder[i], &options);
                let errors = config::validate_config(&settings);
                if !errors.is_empty() {
                    return Err(ConfigError::Invalid(errors));
                }
                config::save(&path, &settings)?;
                print!("{}", show(&settings.folder[i])?);
            }
            FolderCommand::Enable { name } => {
                let i = folder_index(&settings, &name)?;
                settings.folder[i].enabled = true;
                config::save(&path, &settings)?;
                println!("enabled {}", settings.folder[i].name);
            }
            FolderCommand::Disable { name } => {
                let i = folder_index(&settings, &name)?;
                settings.folder[i].enabled = false;
                config::save(&path, &settings)?;
                println!("disabled {}", settings.folder[i].name);
            }
            FolderCommand::Remove { name } => {
                let i = folder_index(&settings, &name)?;
                let removed = settings.folder.remove(i);
                config::save(&path, &settings)?;
                println!("removed {}", removed.name);
            }
        },
    }
    Ok(())
}

fn main() {
    if let Err(error) = execute(Cli::parse()) {
        eprintln!("{error}");
        std::process::exit(2);
    }
}
