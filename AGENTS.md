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

---

## Worklog

### Objective
Implement all actionable improvements from the code review, culminating in
pre-tokenizing the dataset to avoid re-encoding every batch.

### Completed (all 31 items)
- All items in `IMPROVEMENTS.md` resolved (3 intentionally deferred as
  backward-compat or deliberate design choices).
- Pre-tokenization: `TrainingExample` now stores `(input_ids, target_ids):
  Vec<u32>`. Tokenizer is called exactly once per example in
  `load_training_data`; `prepare_batch` concatenates + pads pre-tokenized IDs
  directly. `train()` no longer takes a tokenizer reference.
- Builds with zero warnings.

### Architecture Notes
- EOS fallback: `<|im_end|>` → `</s>` → 2.
- Temperature: `Option<f64>` on `generate()`; greedy when `None` or `<= 0`.
- `TrainConfig.batch_size` is `Option<usize>` (`None` = full batch).
- 90/10 validation split via `StdRng(42)`.
- LR schedule: linear warmup (10% steps) + cosine decay.
