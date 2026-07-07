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
    /// Validate all training examples under a directory
    ValidateAll {
        dir: PathBuf,
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
        Command::ValidateAll { dir } => {
            let results = dataset::validate_all(&dir);
            let total = results.len();
            let passed = results.iter().filter(|r| r.is_ok()).count();
            let failed = total - passed;

            for r in &results {
                if r.is_ok() {
                    println!("✓ {} + {}", r.input_path, r.output_path);
                } else {
                    println!("✗ {} + {}", r.input_path, r.output_path);
                    for err in &r.errors {
                        println!("  - {err}");
                    }
                }
            }

            println!();
            println!("Results: {passed} passed, {failed} failed");

            if failed > 0 {
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
