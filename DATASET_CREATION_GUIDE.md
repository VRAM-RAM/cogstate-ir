# Dataset Creation Guide

Each training example is a pair of YAML files: one describing the **input** context, and one describing the **target** state changes the compiler should produce.

---

## File structure

```
data/
├── example_01/
│   ├── input.yaml
│   └── output.yaml
├── example_02/
│   ├── input.yaml
│   └── output.yaml
└── ...
```

Number examples `01`, `02`, … for easy ordering. The names don't matter to the tooling — any directory with `input.yaml` + `output.yaml` is valid.

---

## Input format

### Full schema

```yaml
character:
  personality:
    - <trait>
    - <trait>
    …

relationship:
  <target>:
    <trait>: <level>

current_state:
  <emotion>: <intensity>
  …

previous_character_message: "<spoken line>"

user_message: "<what the user says>"
```

### Fields

| Field | Required | Description |
|---|---|---|
| `character.personality` | yes | List of personality traits. At least one. |
| `relationship.<target>.<trait>` | yes | Relationship state with a named target (currently always `user`). The trait is e.g. `trust`, `defensiveness`. The value is a qualitative level (`low`, `medium`, `high`, etc.). |
| `current_state.<emotion>` | yes | Current emotional state. The key is an emotion name, the value is an intensity level (`low`, `medium`, `high`, etc.). At least one emotion. |
| `previous_character_message` | no | What the character most recently said (or thought). Helps the compiler understand the character's expressed stance. Omit when the conversation starts or the character hasn't spoken yet. |
| `user_message` | yes | The user's latest line of dialogue. |

### Example

```yaml
character:
  personality:
    - distrustful
    - proud
    - sarcastic

relationship:
  user:
    trust: medium

current_state:
  anger: high

previous_character_message:
  "I don't believe your excuses."

user_message:
  "I'm sorry for lying to you."
```

### Notes

- Personality traits, emotion names, relationship traits, and intensity levels are **free-form**. Use whatever labels feel right for the character.
- The validator only checks that the fields exist and are non-empty. It does not restrict your vocabulary here — that's your creative space.
- `previous_character_message` is optional. Omit it when the conversation starts or when the character hasn't spoken yet.

---

## Output format

### Full schema

```yaml
state_changes:
  emotion:
    <name>: <magnitude>
    …
  relationship:
    <trait>: <magnitude>
    …
  belief:
    <identifier>: <magnitude>
    …
  memory: <action>
  reflection: <action>
```

All keys under `state_changes` are optional. Include only the subsystems that actually change.

### Valid values

| Path | Required | Valid values |
|---|---|---|
| `emotion.<name>` | no | `increases_a_lot`, `increases`, `increases_a_little`, `decreases_a_little`, `decreases`, `decreases_a_lot` |
| `relationship.<trait>` | no | same magnitude scale |
| `belief.<identifier>` | no | same magnitude scale |
| `memory` | no | `reinforce_previous_conflict` |
| `reflection` | no | `required` |

### Magnitude scale

| Value | Meaning |
|---|---|
| `increases_a_lot` | Strong positive shift |
| `increases` | Moderate positive shift |
| `increases_a_little` | Small positive shift |
| `decreases_a_little` | Small negative shift |
| `decreases` | Moderate negative shift |
| `decreases_a_lot` | Strong negative shift |

### Example

```yaml
state_changes:
  relationship:
    trust: increases_a_little
  emotion:
    anger: decreases
  belief:
    people_can_change: increases_a_little
  memory: reinforce_previous_conflict
  reflection: required
```

### Notes

- **Relationship target is implicit.** Since all examples currently involve a single character interacting with `user`, the output only names the trait (e.g. `trust`), not the target.
- **Belief identifiers** are kebab-case labels (`people_can_change`, `user_is_unfair`), not free-form text. Keep them short and consistent across examples.
- **Memory** currently has one action: `reinforce_previous_conflict`. Use it when the user's message **explicitly references or echoes a prior conflict** in the conversation — the character should recall that past event. Do *not* use it for generic tension or vague unease; only when a specific past conflict is directly invoked by the dialogue. Annotate it alongside the emotion/relationship changes it relates to.
- **Reflection** is a flag. Include `reflection: required` when the character needs internal deliberation (e.g. after an intense emotional event). Omit it for casual conversation.

---

## IR vocabulary quick reference

```
emotion.<name>       → increases_a_lot | increases | increases_a_little
                        | decreases_a_little | decreases | decreases_a_lot

relationship.<trait> → increases_a_lot | increases | increases_a_little
                        | decreases_a_little | decreases | decreases_a_lot

belief.<identifier>  → increases_a_lot | increases | increases_a_little
                        | decreases_a_little | decreases | decreases_a_lot

memory               → reinforce_previous_conflict

reflection           → required
```

---

## Annotation principles (from README)

**Do not annotate:**
- exact emotions
- exact numerical values
- hidden chain-of-thought
- complete internal monologues

**Annotate:**
- direction of change
- affected systems
- important events
- behavioral consequences

**Style tips:**
- Use **small interaction fragments**, not full scenes or stories.
- Focus on the **interpretation** of the exchange, not the dialogue quality.
- Literary scenes and complete roleplay conversations introduce stylistic bias — avoid them.
- If uncertain about a magnitude, prefer the middle of the scale (`increases` / `decreases`) over extremes.

---

## Validation

### Single pair

```
cargo run -- validate data/example_NN/input.yaml data/example_NN/output.yaml
```

**Success:**
```
✓ data/example_01/input.yaml + data/example_01/output.yaml: valid
```

**Error** (serde catches typos and unknown values automatically):
```
Error: state_changes.emotion.anger: unknown variant `decreeses`,
expected one of `increases_a_lot`, `increases`, `increases_a_little`,
`decreases_a_little`, `decreases`, `decreases_a_lot`
```

### Batch

```
cargo run -- validate-all data/
```

Prints one line per pair (✓ or ✗) with a summary:
```
Results: 10 passed, 2 failed
```

Exits with code 1 if any pair fails.

---

## Example walkthrough

### Input

```yaml
character:
  personality:
    - distrustful
    - proud

relationship:
  user:
    trust: medium

current_state:
  anger: high

previous_character_message:
  "I don't believe your excuses."

user_message:
  "You are right, I should have told you earlier."
```

What the compiler sees: a distrustful, proud character who is angry, has medium trust in the user, and just expressed disbelief. The user admits fault.

### Output

```yaml
state_changes:
  emotion:
    anger: decreases              # apology reduces anger
  relationship:
    trust: increases              # honesty improves trust
  belief:
    user_can_be_honest: increases # patterns shift
  memory: reinforce_previous_conflict
  reflection: required            # apology warrants internal review
```

### Why these choices

| Change | Rationale |
|---|---|
| `anger.decreases` | An apology, even overdue, lowers tension |
| `trust.increases` | User acknowledged fault → trust partially restored |
| `user_can_be_honest.increases` | Belief about the user shifts positively |
| `memory.reinforce_previous_conflict` | The apology references the earlier conflict |
| `reflection.required` | This is a charged moment — character needs to process it internally |
