# CogStateIR — Implementation Plan

## Architecture (from README)

```
User message → [Compiler] → IR ops → [State Engine] → state snapshot → [Renderer] → response
```

| Component | What it is | Plan |
|---|---|---|
| **Compiler** | Small model (train it) | Tiny transformer fine-tuned in Rust + candle (<100M params) |
| **State Engine** | Rust code applying state deltas | `cogstate` library + CLI |
| **Renderer** | Large model, state → dialogue | Future: llama.cpp bindings from Rust. Not built now. |

---

## Phase 0 — Dataset & IR spec [COMPLETED]

Design the schema, define the IR vocabulary. The dataset is hand-authored YAML/JSONL.

**Input example**:
```yaml
character:
  personality: [distrustful, proud, sarcastic]
  relationship_user: medium
  emotional_state: { anger: high }
user_message: "I'm sorry for lying to you."
target_ir:
  - relationship.trust.increase_small
  - emotion.anger.decrease
  - memory.reinforce_previous_conflict
  - reflection.start
```

**Deliverable**: formal Serde structs for the training example format, documented in code.

---

## Phase 1 — State Engine (Rust) [COMPLETED]

**Core types**:
- `Character` — identity, beliefs, relationships, emotions, memories, goals, attention, habits, state_history
- `IrOp` — enum of all primitive operations with their semantics
- `Engine::apply(state, ops) -> state` — applies IR ops to produce new state

**Persistence**: Serde + JSON or MsgPack.

**CLI**: `cogstate apply <state.json> <ops.yaml>` — load state, apply operations, show new state.

---

## Phase 2 — Training pipeline (Rust + candle) [COMPLETED]

**Model**: fine-tune a small pre-trained transformer via candle (e.g. GPT-2 124M or a tiny 50M variant). -> Supra50M

**Training format**: each example → text prompt encoding (state + message) with target IR ops as text.

**CLI**: `cogstate train --dataset data.jsonl --output model.bin`

**Key challenges**:
- Tokenization — encoding structured state as flat text
- Output parsing — model emits free text → parse into structured `IrOp`s
- Generalization from a small dataset — regularization, augmentation

---

## Phase 3 — Inference [COMPLETED]

**CLI**: `cogstate infer --state state.json --message "..." [--weights model.safetensors] [--model-id ...] [-o new_state.json]`

- Runs compiler over the input (converts `CharacterState` → `spec::Input` with f32→label mapping)
- Outputs predicted IR ops
- Applies IR to state engine
- Displays or saves new state

---

## Phase 4 — Full pipeline [COMPLETED]

**CLI**: `cogstate chat --state state.json --compiler model.safetensors --renderer model.gguf [--port 8080] [-o state.json]`

### Architecture

```
User input → [Compiler (candle 50M)] → IR ops → [State Engine] → new state → [Renderer (llama-server)] → character response
```

### Components

| File | Purpose |
|---|---|
| `src/renderer.rs` | Spawn/manage `llama-server` child process + HTTP client for `/v1/chat/completions` |
| `src/chat.rs` | Interactive REPL: read input → compiler → engine → renderer → display |

### Features

- Compiler model loaded once, reused across turns
- Renderer is optional — without it, you write the character's responses manually
- When `--renderer` is provided: starts `llama-server` with the GGUF model, calls `/v1/chat/completions`, persists KV cache between turns
- System prompt built dynamically from character state (personality, emotions, relationships, beliefs, memory)
- OpenAI-compatible `/v1/chat/completions` endpoint — llama.cpp handles chat template
- Slash commands: `/quit`, `/save`, `/state`, `/help`
- Auto-saves state on exit
- 10-turn conversation window limit to prevent context overflow

### Usage

```bash
# Start interactive chat
cogstate-ir chat \
  --state state.json \
  --compiler model.safetensors \
  --renderer Qwen3-14B-Q4_K_M.gguf

# With custom port and output path
cogstate-ir chat \
  --state state.json \
  --renderer model.gguf \
  -o updated_state.json \
  --port 9090
```

### Status

- ✅ Full pipeline verified end-to-end
- ✅ Compiler predicts IR ops based on user message + character state
- ✅ State engine applies ops and updates state
- ✅ Renderer generates in-character responses using updated state
- ✅ 14B model demonstrates behavior consistent with personality traits

---

## Phase 5 (Research) — Improving the IR compiler

Objectives:

- Expand the dataset (500 → 1500 → 2500+ examples)
- Improve annotation consistency
- Train larger compiler models (360M, 1B...)
- Experiment with LoRA vs full fine-tuning
- Benchmark different base models
- Build an evaluation suite
- Measure generalization on held-out scenarios
- Study failure modes (hallucinations, wrong categories, missing ops)

## Phase 6 (Future) — Ecosystem

- Community-contributed datasets
- Community-trained compiler models
- Automatic evaluation leaderboard
- Integrations with chat frontends
- Renderer adapters (llama.cpp, vLLM, Ollama...)
