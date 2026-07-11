use std::io::Write;
use std::path::Path;
use std::time::Duration;

use colored::*;
use indicatif::{ProgressBar, ProgressStyle};

use crate::engine::{self, CharacterState};
use crate::infer;
use crate::model;
use crate::predict;
use crate::renderer::{self, ChatMessage, Renderer, RendererConfig};
use crate::spec;
use crate::tokenizer::TokenizerWrapper;
use crate::train;

pub struct ChatConfig<'a> {
    pub state_path: &'a Path,
    pub compiler_weights: &'a Path,
    pub compiler_model_id: &'a str,
    pub renderer_model: Option<&'a str>,
    pub renderer_port: u16,
    pub output_state: Option<&'a Path>,
    pub no_metal: bool,
}

fn with_spinner<T>(msg: &str, f: impl FnOnce() -> T) -> T {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["▁", "▃", "▄", "▅", "▆", "▇", "█", "▇", "▆", "▅", "▄", "▃", "▁"]),
    );
    spinner.set_message(msg.to_string());
    spinner.enable_steady_tick(Duration::from_millis(80));
    let result = f();
    spinner.finish_and_clear();
    result
}

fn print_turn_separator() {
    println!("{}", "─".repeat(50).bright_black());
}

pub fn run(config: &ChatConfig) -> anyhow::Result<()> {
    let device = if config.no_metal {
        candle_core::Device::Cpu
    } else if candle_core::utils::metal_is_available() {
        candle_core::Device::new_metal(0)?
    } else {
        candle_core::Device::Cpu
    };

    // ── Startup banner ──
    let banner_mode = config.renderer_model.map_or("compiler only", |_| "full pipeline");
    let banner = format!(
        "\n\
         ╭──────────────────────────────────────╮\n\
         │        CogState Chat                 │\n\
         │   compiler + engine + {banner_mode:<15}│\n\
         │                                      │\n\
         │   Commands: /quit /save /state /help │\n\
         ╰──────────────────────────────────────╯"
    );
    println!("{}", banner.bright_green().bold());
    println!("  device: {device:?}");

    // ── Load compiler model (once, reused across turns) ──
    eprint!("{}  loading compiler tokenizer…", "●".bright_cyan());
    let _ = std::io::stderr().flush();
    let tokenizer = TokenizerWrapper::from_pretrained(config.compiler_model_id)?;
    eprintln!(" {}", "✓".bright_green());

    eprint!("{}  loading compiler config…", "●".bright_cyan());
    let _ = std::io::stderr().flush();
    let llama_config = super::download_config(config.compiler_model_id)?;
    eprintln!(" {}", "✓".bright_green());

    eprint!(
        "{}  loading compiler weights from {}…",
        "●".bright_cyan(),
        config.compiler_weights.display()
    );
    let _ = std::io::stderr().flush();
    let mut varmap = candle_nn::VarMap::new();
    let compiler = model::load_model(
        &mut varmap,
        config.compiler_weights,
        &llama_config,
        &device,
    )?;
    eprintln!(" {}", "✓".bright_green());

    // ── Load character state ──
    let state_content = std::fs::read_to_string(config.state_path)?;
    let mut state = CharacterState::from_json(&state_content)?;
    eprintln!(
        "{}  loaded character state from {}",
        "●".bright_cyan(),
        config.state_path.display()
    );

    // ── Start renderer (optional) ──
    let renderer: Option<Renderer> = if let Some(model_path) = config.renderer_model {
        if !Path::new(model_path).exists() {
            anyhow::bail!(
                "{} renderer model not found: {model_path}\n\
                 Download a GGUF instruct model and pass --renderer <path>",
                "✗".bright_red()
            );
        }
        eprint!(
            "{}  starting renderer ({model_path} on port {})…",
            "●".bright_cyan(),
            config.renderer_port
        );
        let _ = std::io::stderr().flush();
        let r = Renderer::start(&RendererConfig {
            model_path: model_path.to_string(),
            port: config.renderer_port,
        })?;
        eprintln!(" {}", "✓".bright_green());
        Some(r)
    } else {
        eprintln!(
            "{}  no renderer — you will write the character's responses",
            "●".bright_cyan(),
        );
        eprintln!(
            "{}  to use an LLM renderer, pass --renderer <path_to_gguf>",
            " ".bright_black(),
        );
        None
    };

    // ── Conversation state ──
    let mut conversation: Vec<ChatMessage> = Vec::new();
    let mut last_char_message: Option<String> = None;

    // ── REPL loop ──
    let mut line = String::new();
    'chat: loop {
        print_turn_separator();
        println!();
        print!("{} ", "You:".bright_cyan().bold());
        std::io::stdout().flush()?;

        line.clear();
        let n = std::io::stdin().read_line(&mut line)?;
        if n == 0 {
            println!();
            break 'chat;
        }

        let user_msg = line.trim();
        if user_msg.is_empty() {
            continue;
        }

        // Handle slash commands at the You: prompt
        if user_msg.starts_with('/') {
            match user_msg {
                "/quit" | "/exit" => break 'chat,
                "/save" => {
                    let json = state.to_json()?;
                    let path = config.output_state.unwrap_or(config.state_path);
                    std::fs::write(path, &json)?;
                    println!(
                        "{} state saved to {}",
                        "●".bright_green(),
                        path.display()
                    );
                    continue;
                }
                "/state" => {
                    println!("{}", state.to_json()?);
                    continue;
                }
                "/help" => {
                    println!("{}", "Commands:".bright_yellow().bold());
                    println!("  {}    exit the chat", "/quit".bright_cyan());
                    println!("  {}    save current state to file", "/save".bright_cyan());
                    println!("  {}   print current character state", "/state".bright_cyan());
                    println!("  {}    show this help", "/help".bright_cyan());
                    continue;
                }
                _ => {
                    println!(
                        "{} unknown command: {user_msg}. Try /help",
                        "✗".bright_red()
                    );
                    continue;
                }
            }
        }

        // ── Step 1: Run compiler ──
        let input =
            infer::state_to_input(&state, user_msg, last_char_message.as_deref());
        let input_text = train::format_input(&input);

        let raw_output = with_spinner("compiler thinking…", || {
            predict::generate(
                &compiler,
                &tokenizer,
                &input_text,
                &device,
                &llama_config,
                256,
                None,
            )
        })?;

        let state_changes = predict::pipe_to_state_changes(&raw_output);

        if !matches!(state_changes, spec::StateChanges::NoChanges(_)) {
            let target = spec::Target {
                state_changes: state_changes.clone(),
            };
            if let Ok(yaml) = serde_yaml::to_string(&target) {
                let count = yaml.lines().count().saturating_sub(1);
                if count > 0 {
                    println!(
                        "{}  [state: {} change(s)]",
                        "●".bright_blue(),
                        count.to_string().bright_blue()
                    );
                }
            }
        }

        // ── Step 2: Apply state engine ──
        let ir_ops = engine::ops_from_state_changes(&state_changes);
        state = engine::Engine::apply_state(&state, &ir_ops);

        // ── Step 3: Generate character response ──
        let char_response: String = if let Some(renderer) = &renderer {
            let system_prompt = renderer::build_system_prompt(&state);
            let mut messages = Vec::new();
            messages.push(ChatMessage {
                role: "system".to_string(),
                content: system_prompt,
            });
            messages.extend(conversation.iter().cloned());
            messages.push(ChatMessage {
                role: "user".to_string(),
                content: user_msg.to_string(),
            });
            with_spinner("renderer thinking…", || renderer.generate(&messages))?
        } else {
            loop {
                print!("{} ", "Character response:".bright_yellow());
                std::io::stdout().flush()?;
                let mut resp_line = String::new();
                let n = std::io::stdin().read_line(&mut resp_line)?;
                if n == 0 {
                    println!();
                    break 'chat;
                }
                let trimmed = resp_line.trim();
                if trimmed.starts_with('/') {
                    match trimmed {
                        "/quit" | "/exit" => break 'chat,
                        "/save" => {
                            let json = state.to_json()?;
                            let path = config.output_state.unwrap_or(config.state_path);
                            std::fs::write(path, &json)?;
                            println!(
                                "{} state saved to {}",
                                "●".bright_green(),
                                path.display()
                            );
                            continue;
                        }
                        "/state" => {
                            println!("{}", state.to_json()?);
                            continue;
                        }
                        "/help" => {
                            println!("{}", "Commands:".bright_yellow().bold());
                            println!("  {}    exit the chat", "/quit".bright_cyan());
                            println!("  {}    save current state to file", "/save".bright_cyan());
                            println!("  {}   state", "/state".bright_cyan());
                            println!("  {}    show this help", "/help".bright_cyan());
                            continue;
                        }
                        _ => {
                            println!(
                                "{} unknown command: {trimmed}. Try /help",
                                "✗".bright_red()
                            );
                            continue;
                        }
                    }
                }
                if trimmed.is_empty() {
                    println!("{}", "  [no response]".bright_black());
                    continue 'chat;
                }
                break trimmed.to_string();
            }
        };

        // ── Step 4: Update conversation history ──
        conversation.push(ChatMessage {
            role: "user".to_string(),
            content: user_msg.to_string(),
        });
        conversation.push(ChatMessage {
            role: "assistant".to_string(),
            content: char_response.clone(),
        });
        last_char_message = Some(char_response.clone());

        // Prevent context overflow: keep last 10 exchanges (20 messages)
        if conversation.len() > 20 {
            let excess = conversation.len() - 20;
            conversation.drain(0..excess);
        }

        // ── Step 5: Display ──
        println!();
        println!(
            "{} {}",
            "Character:".bright_yellow().bold(),
            char_response
        );
        println!();
    }

    // ── Save state on exit ──
    let save_path = config.output_state.unwrap_or(config.state_path);
    let json = state.to_json()?;
    std::fs::write(save_path, &json)?;
    println!(
        "{} state saved to {}",
        "●".bright_green(),
        save_path.display()
    );
    println!("{}", "goodbye".bright_black());

    Ok(())
}
