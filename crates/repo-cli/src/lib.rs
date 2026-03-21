pub mod cli;
pub mod commands;
pub mod config;
pub mod output;
pub mod plugin;
pub mod progress;

use clap::{CommandFactory, Parser};
use clap_complete::generate;
use cli::{Cli, Commands};

#[must_use]
pub fn run_cli(args: &[String]) -> i32 {
    let cli = match Cli::try_parse_from(args.iter().cloned()) {
        Ok(cli) => cli,
        Err(err) => {
            let _ = err.print();
            return err.exit_code();
        }
    };

    if cli.plain {
        output::disable_color();
    }

    let json = cli.json;

    match cli.command {
        Some(Commands::Docs(cmd)) => commands::docs::run(cmd, json),
        Some(Commands::Health(cmd)) => commands::health::run(&cmd, json),
        Some(Commands::Skills(cmd)) => commands::skills::run(cmd, json),
        Some(Commands::Prompt(cmd)) => commands::prompt::run(cmd, json),
        Some(Commands::Ulid(cmd)) => commands::ulid::run(&cmd),
        Some(Commands::Plugins(cmd)) => commands::plugins::run(cmd, json),
        Some(Commands::Completions(cmd)) => {
            let mut command = Cli::command();
            generate(cmd.shell, &mut command, "repo", &mut std::io::stdout());
            0
        }
        Some(Commands::External(args)) => commands::external::run(&args),
        None => commands::overview::run(cli.json),
    }
}
