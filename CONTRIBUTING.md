# Contributing to CogStateIR

## 1. Project philosophy

CogStateIR is a **concept-first** project. The real artifact is `README.md`, which describes the Cognitive State IR architecture. The Rust implementation is currently experimental and evolving.

The core insight: a character's identity should not live inside an LLM. Separate **persistent state** from **dialogue generation**:

- A small **compiler model** interprets interactions and emits state-change instructions (IR ops).
- A deterministic **state engine** applies those ops to a persistent character state.
- A large **conversation model** generates dialogue from the state snapshot.

The dataset teaches *interpretation*, not writing. Examples are short interaction fragments annotated with qualitative direction changes — no numerical values, no literary scenes.

---

## Current priority areas

The project currently needs help mainly in:

### 1. Dataset expansion

The biggest bottleneck is building a high-quality Cognitive IR dataset.

Useful contributions:
- creating new examples;
- reviewing generated examples;
- covering missing cognitive patterns;
- improving annotation consistency.

### 2. GPU experiments

GPU experiments do not require modifying the code.
If you have access to a powerful GPU, running documented experiments and reporting results is already a valuable contribution.
Useful contributions:
- training larger models;
- comparing LoRA and full fine-tuning;
- running scaling experiments;
- reporting hardware and results.

### 3. Evaluation

The project needs better ways to measure:
- IR correctness;
- generalization;
- hallucination rate;
- missing state transitions.

You can find more informations in [ways to contribute](#3-ways-to-contribute).
---

## 2. Development setup

```
git clone https://github.com/VRAM-RAM/cogstate-ir
cargo install --path . # builds the CLI and install it
cargo run -- <cmd>   # run during development
```

**Rust toolchain**: Edition 2024, requires nightly 1.85+. The repo has no `rust-toolchain.toml` — rely on whatever nightly is in your PATH.

**No test, lint, format, or CI configuration exists yet.** Contributions that introduce any of these are welcome.

Available commands (see `cogstate-ir --help` or README for details):

| Command | Purpose |
|---|---|
| `validate` | Validate a single example pair |
| `validate-all` | Validate all pairs under a directory |
| `init` | Create a character state with given traits |
| `apply` | Apply YAML operations to a character state |
| `train` | Fine-tune the compiler model on your dataset |
| `predict` | Run the trained compiler on an input |

---

## 3. Ways to contribute


### Dataset creation

Hand-craft `input.yaml` / `output.yaml` pairs under `data/`. Each pair is a training example for the compiler. See `DATASET_CREATION_GUIDE.md` for the full schema, IR vocabulary, and annotation principles.

Validate your examples before submitting:

```
cargo run -- validate-all data/
```

### Annotation improvements

The IR vocabulary and schema are still early. Contributions that refine or extend them are valuable:

- New subsystems: goals, commitments, attention, self-model, etc.
- Richer magnitude or action types.
- Better annotation guidelines that reduce ambiguity.
- Updated `DATASET_CREATION_GUIDE.md` reflecting the above.

### Model experiments

Train the compiler under different configurations and report results in `TRAINING_RESULTS.md`. Experiments to try:

- Larger/smaller base models (50M → 360M → 2B+).
- LoRA fine-tuning vs full weights.
- Different learning rates, batch sizes, epoch counts.
- Dataset scaling: 50, 100, 500, 1500 examples.
- Alternative tokenization or prompt formatting.

If you are satisfied by your model, you can, in addition to report your results :

1. Publish it on your own Hugging Face account.
2. Create a GitHub issue describing the experiment.
3. Include:
   - base model;
   - dataset version;
   - training configuration;
   - hardware;
   - evaluation results;
   - example predictions.

The `README.md` of your model repository should follow the rules from [experiment reporting format](#6-experiment-reporting-format).

### Rust implementation

The Rust crate covers CLI, data structures, training pipeline, and the state engine. Areas to work on:

- Compiler model training and inference (`train.rs`, `model.rs`, `predict.rs`).
- State engine features (`engine.rs`) — new operation types, richer state fields.
- Data validation and schema (`spec.rs`).
- CLI ergonomics (`main.rs`).
- Future: renderer bindings (llama.cpp, etc.).

Match existing code conventions: `serde` + `clap` + `anyhow`, edition 2024.

Avoid unnecessary comments. Prefer clear code and meaningful names.
Add comments when explaining non-obvious design decisions.

### Evaluation

The project has no formal evaluation framework yet. Contributions could create:

- Held-out test sets in `test/`.
- Scripts to batch-predict and compare against targets.
- Metrics: exact match rate, edit distance, semantic similarity of IR ops.
- Analysis of failure modes (hallucination, missing ops, wrong magnitudes).

---

## 4. Dataset contribution rules

1. **Must validate.** Run `cogstate-ir validate-all data/` — every pair must pass.
2. **Must be original.** No unreviewed LLM-generated examples.
LLMs may be used as assistants for brainstorming or draft generation (see `.opencode/agents/sampler.md`), but every submitted example must be manually reviewed, corrected, and validated by a human contributor. For now, it isn't really critical since we are experimenting, but it will be critical later.
3. **Prefer short fragments.** 2–4 conversational turns. Avoid full scenes or stories.
4. **Keep character consistent.** Personality, relationship, and emotional state should be coherent within each example.
5. **Use qualitative directions.** Avoid numerical values in annotations. Use the defined magnitude scale (`increases_a_lot` … `decreases_a_lot`).
6. **When uncertain, stay mid-scale.** Prefer `increases` / `decreases` over extremes.
7. **`previous_character_message` is meaningful.** Include it when the character has just spoken — the compiler needs to see what the character expressed.
8. **One logical change per output field.** Don't pack unrelated changes into the same emotion or belief entry.

---

## 5. Code contribution rules

1. **Edition 2024, nightly Rust.** No `rust-toolchain.toml` — use whatever nightly is in PATH.
2. **Match existing style.** The codebase uses `serde` for serialization, `clap` for CLI, `anyhow` for errors, `candle` for ML. You can add comments to explain your decisions, but don't overwrite comments.
3. **Keep CLI + library structure.** Subcommands are the entry point; reusable logic lives in library modules.
4. **No test framework defined yet.** If you add tests, choose a light approach (e.g. `#[test]` in-module) and document it.
5. **Don't commit secrets, model weights, or large files.** The repo already excludes `model.safetensors` via `.gitignore`.
6. **Don't add a `rust-toolchain.toml`.** Rely on PATH for the nightly toolchain.

---

## 6. Experiment reporting format

When you train a model and report results in `TRAINING_RESULTS.md`, follow this structure:

```md
# <Model Name / Size>

## Training setup

- Base model: <HF url or id>
- Parameters: <number>
- Dataset: <number of examples>
- Epochs: <number>
- Learning rate: <value>
- Hardware: <CPU/GPU model>
- Duration: <time>

## Results

Loss after training: <final loss value>
(Optional: loss curve over epochs)

## Prediction — training example

Pick one example from the training set.

Raw model output:


<raw text as emitted by the model>


Parsed output:
<yaml of predicted state_changes>

Target:
<yaml of expected state_changes>

Analysis:
<what the model got right, what it missed, patterns>

## Prediction — held-out example

Pick one example the model was never trained on.

Raw model output:
<...>

Parsed output:
<...>

Target:
<...>

Analysis:
<failure analysis, hallucination patterns, generalization quality>

## Next steps

<what to try next: bigger model, more data, LoRA, different LR, etc.>
```

This format keeps reports comparable across experiments. Append new reports chronologically — do not overwrite prior entries.


---

## 7. Agents

The idea of the project and its architecture are man-made. Anyway, as it is not my main project (just an experiment), I decided to code it almost completely using agentic.
You can find the **project subagents** in `.opencode/agents/` and call them in your *opencode* session using `@<agent_name>`. Currently, I entirely wrote the 15 first examples, and wrote the other ones using `@sampler` (but I verified afterward by myself).
