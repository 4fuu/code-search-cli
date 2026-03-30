mod cli;
mod command;
mod core;
mod lang;

use clap::Parser;
use cli::{Cli, Command};
use core::error::AppError;
use core::output::print_error;

fn main() {
    let cli = Cli::parse();

    let threads = cli.threads.unwrap_or_else(|| {
        let cpus = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(2);
        (cpus / 2).max(1)
    });
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
        .expect("failed to initialize thread pool");

    let format = command_format(&cli.command);
    if let Err(err) = run(cli) {
        if let Some(app_err) = err.downcast_ref::<AppError>() {
            print_error(app_err, &format);
            std::process::exit(1);
        }
        eprintln!("Error: {err:#}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Command::Overview(args) => command::overview::run(args),
        Command::Symbols(args) => command::symbols::run(args),
        Command::Definition(args) => command::definition::run(args),
        Command::References(args) => command::references::run(args),
        Command::Index => command::index::run(),
        Command::ClearCache => command::clear_cache::run(),
        Command::Skill(sub) => command::skill::run(sub),
    }
}

fn command_format(cmd: &Command) -> cli::OutputFormat {
    match cmd {
        Command::Overview(args) => args.format.clone(),
        Command::Symbols(args) => args.format.clone(),
        Command::Definition(args) => args.format.clone(),
        Command::References(args) => args.format.clone(),
        Command::Index | Command::ClearCache | Command::Skill(_) => cli::OutputFormat::Text,
    }
}
