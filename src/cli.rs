//use std::env;
use clap::{Parser, Subcommand};

use meetings::db;

#[derive(Subcommand)]
enum Commands {
    Admin {
        #[arg(long)]
        add: String,
    },
}

#[derive(Parser)]
#[command(version)]
struct Cli {
    #[arg(long)]
    db: String,

    // #[arg(long, default_value_t = false)]
    // adduser: bool,

    // #[arg(long, default_value = "")]
    // username: String,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[tokio::main]
async fn main() -> surrealdb::Result<()> {
    let args = Cli::parse();
    simple_logger::init_with_env().unwrap();
    log::info!("Starting CLI");

    let _dbh = db::get_database().await;
    match args.command {
        Some(Commands::Admin { add }) => {
            log::info!("add: {}", add);
        }
        None => {
            log::info!("There was no subcommand given");
        }
    }

    Ok(())
}
