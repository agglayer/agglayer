mod commands;

use anyhow::Result;
use clap::Parser;
use nu_ansi_term::Color::Green;

fn main() -> Result<()> {
    let app = Xtask::parse();
    app.run()
}

#[derive(Debug, clap::Parser)]
#[structopt(name = "xtask")]
struct Xtask {
    #[command(subcommand)]
    pub command: Command,
}

impl Xtask {
    pub fn run(&self) -> Result<()> {
        match &self.command {
            Command::Elf(elf) => elf.run(),
        }?;
        eprintln!("{}", Green.bold().paint("Success!"));
        Ok(())
    }
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    Elf(commands::Elf),
}
