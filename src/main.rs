use clap::Parser;

mod config;
mod event;
mod journal;
mod monitor;
mod notifier;
mod state;
mod terminal;
mod text;
mod time;

#[derive(Parser)]
#[command(name = "ed-afk-watch", version)]
struct Cli {}

fn main() {
    let _ = Cli::parse();
}
