use std::path::PathBuf;

use clap::{Parser, Subcommand};

mod dataset;
mod engine;
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
    /// Create a new character state with the given personality traits
    Init {
        traits: Vec<String>,
    },
    /// Apply operations from a YAML file to a character state
    Apply {
        state: PathBuf,
        ops: PathBuf,
        #[arg(short = 'o')]
        output: Option<PathBuf>,
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
        Command::Init { traits } => {
            let state = engine::CharacterState::new(traits);
            println!("{}", state.to_json()?);
        }
        Command::Apply { state, ops, output } => {
            let state_content = std::fs::read_to_string(&state)?;
            let state = engine::CharacterState::from_json(&state_content)?;

            let ops_content = std::fs::read_to_string(&ops)?;
            let target: spec::Target = serde_yaml::from_str(&ops_content)?;
            let ir_ops = engine::ops_from_state_changes(&target.state_changes);
            let new_state = engine::Engine::apply_state(&state, &ir_ops);

            let json = new_state.to_json()?;
            match output {
                Some(path) => std::fs::write(path, &json)?,
                None => println!("{}", json),
            }
        }
    }

    Ok(())
}
