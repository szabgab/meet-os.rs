use clap::{Parser, Subcommand};

#[derive(Subcommand)]
enum Commands {
    Adduser { name: Option<String> },
}

#[derive(Parser, Debug)]
#[command(version)]
struct Cli {
    #[arg(long, default_value_t = false)]
    adduser: bool,

    #[arg(long, default_value = "")]
    username: String,
}

fn main() {
    let args = Cli::parse();
    simple_logger::init_with_env().unwrap();
    log::info!("Starting CLI");

    if args.adduser {
        log::info!("adding user");
    }
}
