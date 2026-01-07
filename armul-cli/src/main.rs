use std::path::PathBuf;

use clap::{Parser, Subcommand, command};

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "Assemble a `.s` file")]
    Assemble {
        /// `.s` file to assemble
        file: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Assemble { file } => {
            let output = armul::assemble::assemble(&std::fs::read_to_string(file)?).map_err(|errs| {
                anyhow::anyhow!(
                    "{}",
                    errs.into_iter()
                        .map(|err| format!("line {}: {}", err.line_number, err.error))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            })?;
            println!("Assembled in {} passes.", output.passes);
            Ok(())
        }
    }
}
