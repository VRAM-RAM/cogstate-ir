use std::path::PathBuf;

use candle_nn::VarMap;
use candle_transformers::models::llama;
use clap::{Parser, Subcommand};

mod chat;
mod dataset;
mod engine;
mod infer;
mod model;
mod predict;
mod progress_handler;
mod renderer;
mod spec;
mod tokenizer;
mod train;
mod util;

fn download_config(model_id: &str) -> anyhow::Result<llama::Config> {
    let (owner, name) = util::split_model_id(model_id);
    let client = hf_hub::HFClientSync::new()?;
    let repo = client.model(owner, name);
    let config_path = repo
        .download_file()
        .filename("config.json".to_string())
        .send()
        .map_err(|e| anyhow::anyhow!("downloading config.json: {e}"))?;
    model::config_from_json(&config_path)
}

#[derive(Parser)]
#[command(name = "cogstate", about = "Cognitive State IR toolchain")]
struct Cli {
    /// Select compute device: auto, cpu, metal, cuda
    #[arg(long, default_value = "auto", global = true)]
    device: String,

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
    /// Train the compiler model from the dataset
    Train {
        #[arg(long, default_value = "data/")]
        dataset: PathBuf,
        #[arg(long, default_value = "model.safetensors")]
        output: PathBuf,
        #[arg(long, default_value_t = 100)]
        epochs: usize,
        #[arg(long, default_value_t = 0.0001)]
        lr: f64,
        #[arg(long, default_value = "SupraLabs/Supra-50M-Instruct")]
        model_id: String,
        #[arg(long, default_value_t = 0)]
        checkpoint_every: usize,
        /// Path to a checkpoint to resume training from (downloads only config.json).
        #[arg(long)]
        resume: Option<PathBuf>,
        /// Batch size for training (smaller = less GPU memory)
        #[arg(long, default_value_t = 8)]
        batch_size: usize,
    },
    /// Predict IR operations for a given input using a trained model
    Predict {
        #[arg(long, default_value = "model.safetensors")]
        weights: PathBuf,
        input: PathBuf,
        #[arg(long, default_value = "SupraLabs/Supra-50M-Instruct")]
        model_id: String,
    },
    /// Run the compiler and apply the resulting IR ops to a character state
    Infer {
        /// Path to the character state JSON
        #[arg(long)]
        state: PathBuf,
        /// User message to process
        #[arg(long)]
        message: String,
        /// Previous character message (optional)
        #[arg(long)]
        previous_message: Option<String>,
        /// Path to the fine-tuned model weights
        #[arg(long, default_value = "model.safetensors")]
        weights: PathBuf,
        /// HuggingFace model ID for config
        #[arg(long, default_value = "SupraLabs/Supra-50M-Instruct")]
        model_id: String,
        /// Save updated state to file instead of printing to stdout
        #[arg(short = 'o')]
        output: Option<PathBuf>,
    },
    /// Interactive chat: compiler + state engine + renderer (llama.cpp)
    Chat {
        /// Path to the initial character state JSON
        #[arg(long)]
        state: PathBuf,
        /// Path to the fine-tuned compiler weights
        #[arg(long, default_value = "model.safetensors")]
        compiler: PathBuf,
        /// HuggingFace model ID for the compiler config
        #[arg(long, default_value = "SupraLabs/Supra-50M-Instruct")]
        model_id: String,
        /// Path to the renderer GGUF model (optional — without it you write the character's responses)
        #[arg(long)]
        renderer: Option<PathBuf>,
        /// Port for llama-server
        #[arg(long, default_value_t = 8080)]
        port: u16,
        /// Save updated state to this file on exit (default: overwrite --state)
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
        Command::Train {
            dataset,
            output,
            epochs,
            lr,
            model_id,
            checkpoint_every,
            resume,
            batch_size,
        } => {
            let device = util::select_device(&cli.device)?;
            println!("device: {device:?}");
            println!("model: {model_id}");

            println!("loading tokenizer…");
            let tokenizer = tokenizer::TokenizerWrapper::from_pretrained(&model_id)?;
            println!("vocab size: {}", tokenizer.vocab_size());

            let mut varmap = VarMap::new();
            let (model, llama_config) = if let Some(resume_path) = &resume {
                println!("downloading model config…");
                let llama_config = download_config(&model_id)?;

                println!("loading checkpoint from {}…", resume_path.display());
                let model = model::load_model(&mut varmap, resume_path, &llama_config, &device)?;
                (model, llama_config)
            } else {
                println!("downloading and loading model…");
                model::build_model(&model_id, &mut varmap, &device)?
            };

            let examples = train::load_training_data(&dataset, &tokenizer)?;
            println!("loaded {} examples", examples.len());

            if examples.is_empty() {
                eprintln!("error: no training examples found in {}", dataset.display());
                std::process::exit(1);
            }

            let train_config = train::TrainConfig {
                epochs,
                learning_rate: lr,
                checkpoint_every,
                batch_size: Some(batch_size),
                output: output.clone(),
                ..Default::default()
            };

            train::train(
                &model,
                &varmap,
                &examples,
                tokenizer.eos_id,
                &device,
                &train_config,
                &llama_config,
            )?;
            model::save_model(&varmap, &output)?;
            println!("model saved to {}", output.display());
        }
        Command::Predict {
            weights,
            input,
            model_id,
        } => {
            let device = util::select_device(&cli.device)?;
            println!("device: {device:?}");
            println!("model: {model_id}");

            println!("loading tokenizer…");
            let tokenizer = tokenizer::TokenizerWrapper::from_pretrained(&model_id)?;

            println!("downloading model config…");
            let llama_config = download_config(&model_id)?;

            println!("loading fine-tuned weights from {}…", weights.display());
            let mut varmap = VarMap::new();
            let model = model::load_model(&mut varmap, &weights, &llama_config, &device)?;

            // load input yaml and format as text
            let input_content = std::fs::read_to_string(&input)?;
            let parsed_input: spec::Input = serde_yaml::from_str(&input_content)?;
            let input_text = train::format_input(&parsed_input);

            let pipe = predict::generate(
                &model,
                &tokenizer,
                &input_text,
                &device,
                &llama_config,
                256,
                None,
            )?;
            println!("RAW OUTPUT:");
            println!("{:?}", pipe);
            let changes = predict::pipe_to_state_changes(&pipe);
            let target = spec::Target { state_changes: changes };
            let yaml = serde_yaml::to_string(&target)?;
            println!("{}", yaml);
        }
        Command::Infer {
            state,
            message,
            previous_message,
            weights,
            model_id,
            output,
        } => {
            let device = util::select_device(&cli.device)?;
            infer::run(
                &state,
                &message,
                previous_message.as_deref(),
                &weights,
                &model_id,
                output.as_deref(),
                &device,
            )?;
        }
        Command::Chat {
            state,
            compiler,
            model_id,
            renderer,
            port,
            output,
        } => {
            let device = util::select_device(&cli.device)?;
            let renderer_model = renderer.as_ref().map(|p| p.to_string_lossy().to_string());
            let config = chat::ChatConfig {
                state_path: &state,
                compiler_weights: &compiler,
                compiler_model_id: &model_id,
                renderer_model: renderer_model.as_deref(),
                renderer_port: port,
                output_state: output.as_deref(),
                device,
            };
            chat::run(&config)?;
        }
    }

    Ok(())
}

