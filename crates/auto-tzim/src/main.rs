use clap::Parser;

/// Companion orchestration tool for tzim (skeleton only).
#[derive(Parser)]
#[command(version = auto_tzim_core::VERSION, disable_help_subcommand = true)]
struct Cli {}

fn main() {
    Cli::parse();
}
