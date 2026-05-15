use clap::Parser;
use code_search_cli::{command_output_format, print_error, run_cli, AppError, Cli};

fn main() {
    let cli = Cli::parse();
    let format = command_output_format(&cli.command);
    if let Err(err) = run_cli(cli) {
        if let Some(app_err) = err.downcast_ref::<AppError>() {
            print_error(app_err, &format);
            std::process::exit(1);
        }
        eprintln!("Error: {err:#}");
        std::process::exit(1);
    }
}
