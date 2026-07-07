use std::path::PathBuf;

use clap::{Parser, Subcommand};

mod dataset;
mod spec;

#[derive(Parser)]
#[command(name = "cogstate", about = "Cognitive State IR toolchain")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Validate a training example (input + output YAML pair)
    Validate {
        input: PathBuf,
        output: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Validate { input, output } => {
            let example = dataset::load_example(&input, &output)?;
            let errors = dataset::validate_example(&example);

            if errors.is_empty() {
                println!("✓ {} + {}: valid", example.input_path, example.output_path);
            } else {
                println!("✗ {} + {}: invalid", example.input_path, example.output_path);
                for err in &errors {
                    println!("  - {err}");
                }
            }
        }
    }

    Ok(())
}
