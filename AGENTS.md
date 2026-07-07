# CogStateIR

**This is a concept-first project.** The real artifact is `README.md`, which
describes the Cognitive State IR architecture for AI characters. The Rust crate
is a placeholder until implementation begins.

## Rust toolchain

- **Edition 2024** — requires Rust 1.85+. This environment has `rustc 1.98.0-nightly`.
- No `rust-toolchain.toml` — relies on whatever nightly is in PATH.

## Commands

```
cargo build
cargo run
```

No test, lint, format, or CI config exists yet.

## Design doc

`README.md` is the central document. It lays out:

- The problem with current LLM character systems
- The proposed architecture: Cognitive State Compiler -> Character State Engine -> Conversation Model
- Dataset and annotation philosophy
- Model sizing ideas (2B-4B compiler, 8B-14B+ renderer)

Any implementation work should be grounded in and consistent with that document.
