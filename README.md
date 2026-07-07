# CogStateIR - Cognitive State Intermediate Representation for AI Characters

## Overview

Current LLM-based character systems mostly rely on a simple architecture:

```
User message
      |
      v
Character prompt + context
      |
      v
LLM
      |
      v
Response
```

This approach has a major limitation: the LLM is responsible for everything at once:

* personality simulation;
* memory;
* emotional state;
* relationship evolution;
* reasoning;
* dialogue generation.

As a result, different characters often converge toward similar behaviors because the underlying model has strong default conversational patterns.

The goal of CogStateIR is not to reproduce human cognition itself, but to reproduce the **observable properties that make a character feel like a persistent individual**:

* continuity over time;
* stable personality;
* evolving relationships;
* consistent reactions;
* memory-driven behavior;
* self-consistent dialogue.

---

# Warning

`CogStateIR` is experimental, not efficient and not intended for production.

---

# Core Idea

Separate the character's internal evolution from language generation.

The LLM should not be the character.

The LLM should be the **voice of a character whose internal state is maintained externally**.

Architecture:

```
User message
      |
      |
Previous character message
      |
      |
Current cognitive state
      |
      v
Cognitive State Compiler
      |
      v
Cognitive IR operations
      |
      v
Character State Engine
      |
      v
Conversation Model
      |
      v
User-visible response
```

---

# Cognitive State Compiler

The Cognitive State Compiler is a small model responsible for interpreting interactions.

It does not generate dialogue.

Its task is:

> Given the current character state, the user's message, and the character's previous expression, determine how the internal state should evolve.

Mathematically:

```
f(
    current_state,
    user_message,
    previous_character_message
)
    ->
    delta_state
```

Not:

```
f(message) -> response
```

The compiler interprets the interaction.
The conversation model produces the words.

---

# Why Include the Previous Character Message?

A character is not only affected by what others say.

A character is also affected by what it has **done and expressed previously**.

The previous message is an action performed by the character.

Example:

Current state:

```yaml
personality:
  proud: high
  distrustful: high

relationship:
  user:
    trust: medium
```

User:

```
"I'm sorry for lying to you."
```

Previous character message:

```
"I don't care about your excuses. People like you always betray others."
```

The compiler should understand that the character has already:

* expressed hostility;
* created emotional distance;
* reinforced a defensive posture.

Output:

```yaml
operations:

- relationship.defensiveness++

- emotion.anger.stabilize

- memory.reinforce(
    previous_conflict
)

- reflection.start(
    "possible_overreaction"
)
```

Without the previous character message, the system only sees the user's apology.

With it, the system sees a conversation between two evolving agents.

---

# Internal State vs Expressed State

A character can feel one thing and express another.

Example:

```yaml
internal_state:

emotion:
  anger: high

beliefs:
  user_is_unfair: true


expressed_state:

tone:
  calm

strategy:
  avoid_conflict
```

The internal state represents the character.

The expressed state represents the behavior shown to others.

The previous character message is the bridge between both.

It allows the system to understand:

* what the character felt;
* what the character chose to show;
* what consequences this expression created.

---

# Character Speech as a Cognitive Event

A spoken sentence is not only output.

It can create new internal constraints.

Example:

Character says:

```
"I will never forgive you."
```

This creates a conversational commitment:

```yaml
commitments:

- id: statement_42

  type:
    emotional_claim

  content:
    "I will never forgive you"

  strength:
    medium
```

Later:

User:

```
"But you helped me yesterday."
```

The compiler can detect tension:

```yaml
operations:

- commitment.review(statement_42)

- self_consistency.pressure++

- belief.update(
    "I never forgive people"
)

- reflection.start
```

This allows characters to evolve through their own actions, not only through external events.

---

# Why Use State Transitions Instead of Absolute Values?

Avoid:

```yaml
trust: 0.73
anger: 0.24
```

because numerical values are difficult to annotate and interpret.

Prefer:

```yaml
trust:
  increases

anger:
  decreases

uncertainty:
  increases
```

The actual numerical interpretation belongs to the state engine.

Example:

```rust
trust.increase(SMALL);
```

could internally become:

```
+0.02
```

or:

```
relationship_factor * event_weight
```

without retraining the model.

---

# Cognitive Intermediate Representation (Cognitive IR)

The compiler outputs a small set of primitive operations.

Example:

```yaml
operations:

- relationship.trust++

- emotion.anger--

- memory.reinforce(event_42)

- attention.focus(user)

- reflection.start

- commitment.review(statement_12)
```

The Cognitive IR acts like an intermediate representation in a compiler.

Similar concept:

```
Source code
     |
     v
LLVM IR
     |
     v
Machine code
```

CogStateIR:

```
Conversation
     |
     v
Cognitive IR
     |
     v
Character behavior
```

---

# Character State Engine

The persistent identity exists outside the LLM.

Example:

```
character/

├── core/
│   ├── identity
│   ├── personality
│   └── values
│
├── cognition/
│   ├── beliefs
│   ├── goals
│   ├── attention
│   ├── self_model
│   └── commitments
│
├── memory/
│   ├── episodic
│   ├── semantic
│   ├── procedural
│   └── memory_index
│
├── social/
│   └── relationships
│
├── affect/
│   ├── emotions
│   └── moods
│
├── behavior/
│   ├── habits
│   └── expression_style
│
└── history/
    └── state_transitions
```

The database is the persistent identity.

The LLM is only the language renderer.

---

# Dataset Creation

The main challenge is not model training.

It is creating the right dataset.

Avoid:

* literary scenes;
* complete roleplay conversations;
* generated stories.

These introduce strong stylistic and cultural biases.

The dataset should teach interpretation, not writing.

Use small interaction fragments.

Example:

```
Character information:

- personality:
    distrustful
    proud

- relationship:
    user trust = medium

- current state:
    anger = high


Previous character message:

"I don't believe your excuses."


User:

"You are right, I should have told you earlier."
```

Target:

```yaml
operations:

- emotion.anger.decrease

- relationship.trust.increase

- memory.reinforce(
    honesty_issue
)

- reflection.start

- expression:
    maintain_defensive_tone
```

---

# Annotation Principles

Do not annotate:

* exact emotions;
* exact numerical values;
* hidden chain-of-thought;
* complete internal monologues.

Annotate:

* direction of change;
* affected systems;
* important events;
* behavioral consequences.

Examples:

```
trust increases slightly

anger decreases

old memory activated

relationship becomes uncertain

character becomes defensive

previous statement requires reconsideration
```

---

# Possible Model Architecture

## Cognitive State Compiler

Size:

```
2B-4B parameters
```

Role:

* interpret interactions;
* detect conflicts;
* update internal state;
* produce Cognitive IR.

No dialogue generation.

---

## Conversation Model

Size:

```
8B-14B+
```

Role:

Generate natural language from:

* current character state;
* relevant memories;
* cognitive operations;
* personality constraints;
* communication style.

Input:

```
character_state
+
cognitive_ir
+
conversation_context
```

Output:

```
character_message
```

---

## Memory Consolidator

Optional asynchronous model:

```
Conversation
      |
      v
Memory Consolidator
      |
      v
Long-term database
```

Responsibilities:

* merge memories;
* remove irrelevant information;
* reinforce important events;
* update relationships;
* detect recurring patterns.

---

# Full Cognitive Loop

The complete architecture becomes:

```
User message
      |
      v
Cognitive State Compiler
      |
      v
State transition
      |
      v
Character State Engine
      |
      v
Conversation Model
      |
      v
Character response
      |
      v
Self-observation
      |
      v
Future state updates
```

The character learns not only from what happens to it.

It also learns from what it chooses to do.

---

# Tooling

This repository includes a Rust CLI for dataset validation:

- `cargo run -- validate <input.yaml> <output.yaml>` — validate a single example pair.
- `cargo run -- validate-all <directory>` — validate all pairs under a directory.

See `DATASET_CREATION_GUIDE.md` for the dataset format and complete reference.

---

# Key Principle

The character is not the prompt.

The character is not the LLM.

The character is a dynamic state evolving through interactions.

The LLM is only the component that converts this evolving internal state into human-readable language.

CogStateIR explores how to create persistent, coherent, adaptive character illusions without claiming to reproduce consciousness.
