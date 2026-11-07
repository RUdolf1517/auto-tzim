//! A controllable tzim-shaped process for adapter integration tests.
use std::io::{Read, Write};
use std::time::Duration;

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if let Some(path) = std::env::var_os("AT04_LOG") {
        let mut log = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .expect("open call log");
        writeln!(log, "{}", args.join(" ")).expect("write call log");
    }
    if args.iter().any(|s| {
        matches!(
            s.as_str(),
            "--info" | "--list-formats" | "--dry-run" | "--check-deps"
        )
    }) {
        std::process::exit(99);
    }
    if args.len() != 1 {
        std::process::exit(98);
    }
    if std::env::var_os("AT04_STDIN_CHECK").is_some() {
        let mut data = Vec::new();
        std::io::stdin().read_to_end(&mut data).expect("read stdin");
        if !data.is_empty() {
            std::process::exit(97);
        }
    }
    if let Ok(ms) = std::env::var("AT04_DELAY_MS") {
        std::thread::sleep(Duration::from_millis(ms.parse().expect("delay")));
    }
    if std::env::var("AT04_FAIL_FLAG").is_ok_and(|flag| flag == args[0]) {
        eprintln!("unknown flag {}", args[0]);
        std::process::exit(42);
    }
    match args[0].as_str() {
        "--version" => print!("{}", std::env::var("AT04_VERSION").unwrap_or_else(|_| "tzim 0.16.1\n".into())),
        "--capabilities" => print!("{}", std::env::var("AT04_CAPABILITIES").unwrap_or_else(|_|
            r#"{"capabilities_schema_version":1,"result_schema_version":2,"build_features":{},"compiled_formats":[],"default_enabled_formats":[],"selection_flags":[],"handler_opt_ins":[],"transformations":[],"confirmation_policies":[]}"#.into())),
        _ => std::process::exit(98),
    }
}
