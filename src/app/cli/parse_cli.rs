use std::env;
use std::ffi::OsString;
use clap::{Arg, ArgAction};
use crate::app::cli::command::{CLI, CLICommand, GenerateClientCommand, GenerateCommand, GenerateEntityCommand, MigrateCommand, PurgeCommand, SeedCommand, SeedCommandAction, ServeCommand};
use crate::app::entrance::Entrance;
use crate::app::new_app::ctx::AppCtx;
use crate::app::new_app::new_result::Result;
use crate::app::program::Program;

pub(crate) fn parse_cli() -> Result<CLI> {
    let app_ctx = AppCtx::get()?;
    let program = app_ctx.program();
    let entrance = app_ctx.entrance();
    let version = Box::leak(Box::new(format!("Teo {} ({}) [{}]", env!("CARGO_PKG_VERSION"), program.to_string(), entrance.to_str())));
    let about = Box::leak(Box::new(match entrance {
        Entrance::CLI => format!("{version}\n\nRun Teo application with CLI."),
        Entrance::APP => format!("{version}\n\nRun Teo application with custom code loaded."),
    }));
    let matches = ClapCommand::new("teo")
        .version(version.as_str())
        .disable_version_flag(true)
        .disable_help_subcommand(true)
        .arg_required_else_help(true)
        .about(about.as_str())
        .subcommand_required(true)
        .arg(Arg::new("SCHEMA_FILE")
            .short('s')
            .long("schema")
            .help("The schema file to load").action(ArgAction::Set)
            .required(false)
            .num_args(1)
            .global(true))
        .arg(Arg::new("ENV")
            .short('e')
            .long("env")
            .help("The environment to use")
            .action(ArgAction::Set)
            .required(false)
            .num_args(1)
            .global(true))
        .arg(Arg::new("version")
            .short('v')
            .long("version")
            .help("Print version information")
            .action(ArgAction::Version))
        .subcommand(ClapCommand::new("serve")
            .about("Run migration and start the server")
            .arg_required_else_help(false)
            .arg(Arg::new("no-migration")
                .short('M')
                .long("no-migration")
                .help("Start server without running migration")
                .action(ArgAction::SetTrue))
            .arg(Arg::new("no-autoseed")
                .short('S')
                .long("no-autoseed")
                .help("Start server without auto seeding autoseed dataset")
                .action(ArgAction::SetTrue)))
        .subcommand(ClapCommand::new("generate")
            .about("Generate code")
            .arg_required_else_help(true)
            .subcommand(ClapCommand::new("client")
                .about("Generate client")
                .arg(Arg::new("all")
                    .short('a')
                    .long("all")
                    .help("Generate all clients")
                    .action(ArgAction::SetTrue)
                    .conflicts_with("NAME"))
                .arg(Arg::new("NAME")
                    .action(ArgAction::Append)
                    .conflicts_with("all")
                    .help("Client names to generate")
                    .num_args(1..)))
            .subcommand(ClapCommand::new("entity")
                .about("Generate model entities")
                .arg_required_else_help(false)
                .arg(Arg::new("all")
                    .short('a')
                    .long("all")
                    .help("Generate all clients")
                    .action(ArgAction::SetTrue)
                    .conflicts_with("NAME"))
                .arg(Arg::new("NAME")
                    .action(ArgAction::Append)
                    .conflicts_with("all")
                    .help("Entity names to generate")
                    .num_args(1..))))
        .subcommand(ClapCommand::new("migrate")
            .about("Run migration")
            .arg(Arg::new("dry")
                .short('d')
                .long("dry")
                .help("Dry run")
                .action(ArgAction::SetTrue)))
        .subcommand(ClapCommand::new("seed")
            .about("Seed data")
            .arg(Arg::new("unseed")
                .short('u')
                .long("unseed")
                .help("Unseed records")
                .action(ArgAction::SetTrue))
            .arg(Arg::new("reseed")
                .short('r')
                .long("reseed")
                .help("Reseed records")
                .action(ArgAction::SetTrue))
            .arg(Arg::new("all")
                .short('a')
                .long("all")
                .help("Do for all data sets")
                .action(ArgAction::SetTrue)
                .conflicts_with("NAME"))
            .arg(Arg::new("NAME")
                .action(ArgAction::Append)
                .conflicts_with("all")
                .help("Data set names to process")
                .num_args(1..)))
        .subcommand(ClapCommand::new("purge")
            .about("Purge and clear the database without dropping tables."))
        .get_matches_from(match program {
            Program::Python(_) | Program::NodeJS(_) => {
                env::args_os().enumerate().filter(|(i, x)| (*i != 1) && (!x.to_str().unwrap().ends_with("ts-node") && !x.to_str().unwrap().ends_with(".ts"))).map(|(_i, x)| x).collect::<Vec<OsString>>()
            },
            Program::Rust(_) => env::args_os().enumerate().filter(|(i, x)| {
                !((*i == 1) && x.to_str().unwrap() == "teo")
            }).map(|(_i, x)| x).collect::<Vec<OsString>>(),
            _ => env::args_os().collect::<Vec<OsString>>(),
        });
    let schema: Option<&String> = matches.get_one("SCHEMA_FILE");
    let command = match matches.subcommand() {
        Some(("serve", submatches)) => {
            let env: Option<&String> = submatches.get_one("ENV");
            CLICommand::Serve(ServeCommand { no_migration: submatches.get_flag("no-migration"), no_autoseed: submatches.get_flag("no-autoseed"), env: env.cloned() })
        }
        Some(("generate", submatches)) => {
            match submatches.subcommand() {
                Some(("client", submatches)) => {
                    let names: Option<Vec<String>> = submatches.get_many::<String>("NAME").map(|s| s.map(|v| v.to_string()).collect::<Vec<String>>());
                    CLICommand::Generate(GenerateCommand::GenerateClientCommand(GenerateClientCommand { all: submatches.get_flag("all"), names }))
                }
                Some(("entity", submatches)) => {
                    let names: Option<Vec<String>> = submatches.get_many::<String>("NAME").map(|s| s.map(|v| v.to_string()).collect::<Vec<String>>());
                    CLICommand::Generate(GenerateCommand::GenerateEntityCommand(GenerateEntityCommand { all: submatches.get_flag("all"), names }))
                }
                _ => unreachable!()
            }
        }
        Some(("migrate", submatches)) => {
            CLICommand::Migrate(MigrateCommand { dry: submatches.get_flag("dry") })
        }
        Some(("seed", submatches)) => {
            let action = if submatches.get_flag("reseed") {
                SeedCommandAction::Reseed
            } else if submatches.get_flag("unseed") {
                SeedCommandAction::Unseed
            } else {
                SeedCommandAction::Seed
            };
            let names: Option<Vec<String>> = submatches.get_many::<String>("NAME").map(|s| s.map(|v| v.to_string()).collect::<Vec<String>>());
            CLICommand::Seed(SeedCommand {
                action,
                all: submatches.get_flag("all"),
                names,
            })
        }
        Some(("purge", _submatches)) => {
            CLICommand::Purge(PurgeCommand { })
        }
        _ => unreachable!()
    };
    Ok(CLI { command, schema: schema.map(|s| s.to_string()) })
}