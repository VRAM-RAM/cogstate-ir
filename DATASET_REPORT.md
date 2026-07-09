# CogStateIR Dataset Coverage Report

**Date:** 2026-07-09
**Scope:** 150 examples (`example_01` through `example_150`)

---

## Executive Summary

The dataset at `data/` contains **150 examples**, each with an `input.yaml` (character state + conversation fragment) and `output.yaml` (target state changes). It is a first-pass annotation effort that shows strong coverage of **interpersonal emotional dynamics** but has significant gaps in **context/setting variety**, **genre diversity**, **primal/physiological drives**, and **speaker role variety**. There are also **7 duplicate example pairs** that inflate the effective unique count to ~136.

---

## 1. Coverage Across Cognitive Dimensions

### 1.1 Personality Traits (Character Identity)

**Strength:** 257 unique personality traits across 150 examples — excellent lexical diversity.

**Well-covered archetypes:**
- **Proud** (17 examples), **Loyal** (9), **Guarded** (8), **Patient** (8), **Calm** (8), **Stubborn** (7), **Protective** (7), **Insecure** (7)
- The full trait list shows good nuance (e.g. *grieving*, *traumatized*, *vindictive*, *melancholic*, *longing*)

**Partially covered (1-2 occurrences):**
- **Primal/physiological:** `hungry`, `tired`, `exhausted` — only in current_state, never as personality
- **Intellectual:** `intellectual` (1), `analytical` (3), `logical` (1), `existential` (1)
- **Spiritual/transcendental:** `spiritual` (2), `wise` (5 — okay coverage)
- **Playful/joyful:** `funny` (1), `cheerful` (4), `witty` (1) — notably sparse
- **Deceitful/manipulative:** `deceitful` (1), `manipulative` (2), `charming` (2), `calculating` (3) — somewhat thin

**Gap — Primal urges:** The README architecture mentions *primal urges* as a cognitive dimension, but the dataset contains almost no examples rooted in hunger, thirst, exhaustion, cold, heat, pain, lust (in a non-romantic sense), or survival instincts. The only exception is `example_51` (hunger/exhaustion/wariness in a post-apocalyptic setting).

**Recommendations:**
- Add examples with **primal survival states**: poisoned, freezing, starving, injured
- Add more **playful/humorous** personality configurations
- Add **low-status/servile** character types (currently 0 examples with traits like *subservient*, *cowering*, *obsequious*)
- Add **authority figures** beyond mentor/teacher (e.g. *commander*, *ruler*, *judge* with actual positional power dynamics)

### 1.2 Current State / Affective Tags

**Strength:** 93 unique emotion/state keys in the input, covering most basic emotions.

**Well-covered:** `calm` (27), `anger` (22), `focus` (16), `determination` (11), `fear` (10), `curiosity` (10), `hope` (10)

**Gaps — underrepresented emotions:**
- **Disgust:** appears only 1 time as input state (example_13)
- **Shame:** appears only 3 times
- **Boredom:** appears only 2 times
- **Awe/wonder:** appears only 1 time (example_45)
- **Triumph/pride (positive):** `pride` as state appears 7 times but mostly in the mentor/achievement duplicate cluster (examples 100/149, 146/97, 147/98)
- **No physiological states in input:** No `pain`, `fever`, `intoxication`, `dizziness`, `nausea`
- **No sensory-affective states:** No `beauty`, `awe`, `sublime`, `terror` (as distinct from fear)

**Recommendation:** Add examples with disgust-driven interactions, shame/embarrassment as primary state, awe/wonder in non-childlike contexts, and physiologically-driven states (pain, illness, intoxication).

### 1.3 Relationship Dimensions

**Critical imbalance:** `trust` appears in **112 out of 150 examples** as a relationship key. The entire dataset is trust-centric.

Relationship key distribution:

| Key | Count | Percentage |
|---|---|---|
| `trust` | 112 | 75% |
| `closeness` | 41 | 27% |
| `respect` | 35 | 23% |
| `friendliness` / `friendness` | 20 | 13% |
| `familiarity` | 9 | 6% |
| `rivalry` | 7 | 5% |
| `suspicion` | 7 | 5% |
| `attraction` | 6 | 4% |

For comparison, `obedience` appears once, `hatred` once, `obligation` once, `possessiveness` once, `reliance` twice.

**Missing relationship dimensions:**
- **Power/dependency:** no `power_dynamic`, `dependence`, `authority`
- **Professional:** no `mentor`, `student`, `colleague` — these are implied but never explicit
- **Family:** no `parent`, `child`, `sibling` relationship keys
- **Competitive:** `rivalry` (7) and `competition` only appear in sports/athletic contexts

**Recommendation:** Diversify relationship keys beyond `trust`. Create examples where trust is irrelevant — e.g., master-servant, guard-prisoner, doctor-patient, parent-child dynamics.

### 1.4 Beliefs (Values & Taboos)

**Strength:** 153 unique belief identifiers show rich conceptual variety.

**Well-represented beliefs:** `people_can_change` (4), `user_cares_about_me` (4), `hard_work_pays_off` (3)

**Interesting belief themes present:**
- Epistemic: `evidence_cannot_be_fought`, `truth_comes_from_all_places`, `empire_fell_from_weak_borders`
- Moral: `honesty_matters`, `justice_must_be_served`, `promises_matter`, `integrity_is_everything`
- Relational: `user_is_a_threat`, `user_will_stay`, `i_am_not_alone`, `some_people_are_safe`
- Self: `i_am_a_monster`, `i_am_untouchable`, `i_deserve_happiness`

**Gap — belief identifiers lack a naming convention standard:**
- Some use snake_case (`user_cares_about_me`, `hard_work_pays_off`)
- Some use spaces would break kebab-case (e.g. `empire_fell_from_weak_borders`)
- Some are oddly specific (`empire_fell_from_weak_borders`, `user_never_supports_me`)
- There's no consistent namespace (e.g. `belief.self.i_am_alone` vs. `i_am_alone`)

**Gap — taboo/violation beliefs:** Few examples involve moral violations — only 2-3 about secrets/betrayal, none about theft, violence, cruelty, or ethical dilemmas.

**Recommendation:** Create examples for taboo transgressions (character witnesses something morally wrong, is asked to do something unethical). Standardize belief naming (e.g. `self.efficacy`, `user.benevolence`, `world.justice`).

### 1.5 Scene/Context

**Critical gap:** **0 examples have any `context`, `setting`, or `scene` field.** The dataset currently has no explicit scene/context key in the schema. All setting is implied through dialogue alone.

Genre analysis (inferred from dialogue):

| Genre | Count | Percentage |
|---|---|---|
| interpersonal/conflict | 76 | 51% |
| romantic | 15 | 10% |
| workplace | 10 | 7% |
| artistic | 8 | 5% |
| social/domestic | 8 | 5% |
| fantasy/medieval | 7 | 5% |
| service/casual | 7 | 5% |
| action/thriller | 6 | 4% |

**Missing contexts entirely:**
- **Historical/war:** 0 examples
- **Horror/supernatural (as primary):** 0 (curse in ex_68, ghost in ex_48 are close but not horror)
- **Post-apocalyptic/scarcity:** only example_51
- **Cyberpunk/tech-noir:** 0
- **Children/coming-of-age:** 0
- **Nature/wilderness/survival:** only implied in ex_7 (tunnel/dungeon)
- **Medical/hospital:** 1 (example_140 — doctor's prognosis)
- **Legal/courtroom:** 0
- **Prison/captivity:** 0

**Recommendation:** Either add an explicit `context` field to the input schema, or intentionally create examples covering at least 5 more distinct genres/settings. The current 51% in "interpersonal/conflict" is dangerously homogeneous.

---

## 2. Coverage Across Speaker Roles

### 2.1 Narrator Presence

**Action markers:** Only **33 examples** (22%) use asterisk `*action*` markers in either character or user messages. The remaining 78% are pure dialogue.

The dataset lacks examples where:
- A **third-person narrator** describes events (all examples are dyadic user/character)
- The **character thinks** (internal monologue) — only `previous_character_message` captures what the character said, not what they thought
- Multiple **non-user characters** are present (every example is exactly 1 character + 1 user)

### 2.2 Character Role Diversity

Every example has exactly one named entity (`character`) interacting with `user`. There are:
- No examples with the character as a **group/collective**
- No examples with the user as a **non-human entity** (AI, animal, object)
- No examples where the **character is the narrator** telling a story
- No **multi-party** conversations (3+ participants)

**Recommendation:** Add examples with:
- An internal monologue variant (character's own thoughts as `previous_character_message`)
- Third-party references (e.g., "she said that about you")
- Collective characters ("we", the council, the team)

---

## 3. Coverage Across Interaction Types

Interaction type distribution:

| Type | Count |
|---|---|
| other/mixed | 46 |
| disagreement/defiance | 34 |
| gratitude/appreciation | 15 |
| apology/reconciliation | 12 |
| ultimatum/threat | 8 |
| confrontation/questioning | 8 |
| understanding/acknowledgment | 5 |
| trust/belief challenge | 5 |
| agreement/acceptance | 5 |
| confession/affection | 3 |
| greeting | 3 |
| praise/celebration | 3 |
| request for support | 2 |
| farewell/goodbye | 1 |

**Well-covered:** disagreement/defiance (34), apologies (12), gratitude (15)

**Poorly covered:**
- **Farewells/goodbyes** — only 1 example (example_03), and it's a "goodnight" which barely counts
- **Requests for support** — only 2 (help me / protect me)
- **Greetings** — only 3, all of which are also service encounters
- **Confessions of affection** — only 3, all romantic, none platonic
- **Humor/joking/teasing** — only 1-2 examples with playful banter

**Missing entirely:**
- Negotiation/bargaining
- Teaching/instruction where the character learns from user
- Deception/lying by the character (not just being lied to)
- Comforting/soothing by the user (character is upset, user comforts)
- Celebration/shared joy
- Character admitting fault/apologizing to user
- Character asking a question (character initiates new topic)

---

## 4. Gaps, Inconsistencies, and Unbalanced Areas

### 4.1 Critical: 7 Duplicate Example Pairs

The following examples are exact duplicates (same input and output):

| Pair | Topic |
|---|---|
| example_101 = example_150 | Romantic confession |
| example_98 = example_147 | Achievement/proud mentor |
| example_95 = example_145 | Concern about trip |
| example_100 = example_149 | Presentation success |
| example_94 = example_144 | River/peace reflection |
| example_97 = example_146 | Promotion recommendation |
| example_99 = example_148 | Gift of scarf |

These 7 duplicates reduce the effective unique dataset size from 150 to **136 examples**. The duplicates are all in the 90-150 range, suggesting a copy-paste issue during dataset expansion.

### 4.2 YAML Format Inconsistency

The dataset uses two different YAML styles:
- **Examples 1-100 (mostly):** Proper inline YAML with quoted strings on single lines (`previous_character_message: "text..."`)
- **Examples 102-110:** Block-scalar style without quotes, using indentation for multi-line strings:

```yaml
previous_character_message: Oh, so you two looked pretty close at the party. Should
  I be worried?
```

This is valid YAML but inconsistent. Lines are folded at arbitrary positions (around 70 chars) which could affect tokenization and model training.

### 4.3 `{{{User}}}` Placeholder Inconsistency

Only **8 examples** use the `{{{User}}}` template placeholder. The remaining 142 examples hard-code "you", "him", "her", or nothing. This means the dataset is not consistently template-ized for multi-character support.

### 4.4 Magnitude Distribution Skew

In output annotations, the system supports 6 magnitudes. Analysis of actual usage shows a preference for mid-range values:
- `increases` and `decreases` are the most common
- `increases_a_lot` and `decreases_a_lot` are used sparingly (mostly in high-stakes examples like ex_38, ex_50, ex_58)
- `increases_a_little` and `decreases_a_little` are frequent but seem to be used inconsistently — sometimes for the same type of change as `increases` but with less confidence

There is no clear threshold guidance for when to use `increases` vs `increases_a_little`.

### 4.5 Memory and Reflection Flag Usage

- **Memory:** Only 30 examples (20%) use `memory: reinforce_previous_conflict`. This seems low given that 76 examples are interpersonal conflicts. Many conflict examples that *should* reference past events do not have this flag.
- **Reflection:** 94 examples (63%) have `reflection: required`. This seems high — potentially overused for any moderate emotional event. The guide says "omit it for casual conversation", but many "disagreement" examples include it.

### 4.6 No Negative Examples

There are **no negative examples** — examples where the correct output is *not* what a naive model would predict. For instance:
- User apologizes sincerely but character is paranoid → trust *decreases* (the dataset does have ex_25 where user offers help but suspicion increases — this is good, but rare)
- The model could benefit from more counter-intuitive examples

### 4.7 No Multi-Turn Examples

Every example is a **single exchange** (one character message + one user message). There are no 2-turn or 3-turn conversational fragments. This means the compiler never learns from an extended context.

---

## 5. Quality of Annotations

### 5.1 Strengths

1. **Emotional logic is mostly sound.** Example: `example_38` — character is caught lying → `calm: decreases_a_lot`, `confidence: decreases_a_lot`, `lying_is_safe: decreases_a_lot`. The emotional cascade is well-reasoned.

2. **Beliefs are well-chosen.** Example: `example_49` — mercenary is told his daughter is in danger → `family_comes_first: increases_a_lot`. This is exactly the right belief to surface.

3. **Contrast pairs exist.** E.g., `example_01` (distrustful + proud + sarcastic, user apologizes → trust increases) vs `example_13` (cynical + observant, user lies → trust decreases). This helps the model learn personality-conditioned responses.

4. **Physiological state integration** in `example_51` (hunger + exhaustion + wariness) is well-executed and should be a template for more.

### 5.2 Weaknesses

1. **Inconsistent magnitude calibration.** Compare:
   - `example_08` (intellectual, confronted with evidence): `certainty: decreases_a_lot` — the strongest magnitude for a factual error
   - `example_38` (liar caught red-handed): `calm: decreases_a_lot`, `confidence: decreases_a_lot` — also strongest magnitude

   Both use `decreases_a_lot` but example_38 is a much more severe emotional event. The same magnitude applied to both dilutes the scale.

2. **Belief naming is ad-hoc.** The same belief concept appears under different names:
   - `people_can_change` (examples 01, 33) and `people_dont_really_change` (131) and `people_change` (25)
   - `user_is_a_threat` (107, 138) — same name used consistently (good!)
   - But `user_is_deep` (22) is too specific to be reusable

3. **Some outputs are thin.** `example_03` output has only `emotion: ashamed: increases` (no relationship change for a social goodbye interaction). `example_106` output has only 3 change entries for a charged conflict exchange.

4. **Overuse of `reflection: required`.** About 94 examples use it. Many casual interactions (example_80 — surprise party) probably don't warrant "required" reflection.

5. **Memory flag is underused.** Only 30 examples mark memory reinforcement, even though most conflict examples reference past issues. The guideline says "only when a specific past conflict is directly invoked" — but many examples have the user explicitly referencing past actions.

### 5.3 Specific Annotation Quality Issues

| Example | Issue |
|---|---|
| `example_03` | Output only has `ashamed: increases`, no relationship change for a social interaction |
| `example_28` | Character wants adventure, user says no → output does not include `disappointment: increases` for the character's own disappointment (user is disappointed, not character) — wait, it does? Let me recheck: The output shows `disappointment: increases` — but is the user disappointed or the character? The YAML doesn't specify whose disappointment. This is a schema ambiguity. |
| `example_117` | `no_changes` for a meeting scheduling interaction, but the character is "calm, steady, neutral" — this is correct but also the least interesting example possible |
| `example_119` | Missing `previous_character_message` for a service interaction — inconsistent with other service examples |

---

## 6. Specific Recommendations

### 6.1 Immediate Fixes

1. **Remove the 7 duplicate pairs** (101/150, 98/147, 95/145, 100/149, 94/144, 97/146, 99/148) — keep the lower-numbered one and either replace or delete the duplicate.

2. **Standardize YAML formatting** across all examples — prefer consistent inline quoting style.

3. **Add `{{{User}}}` placeholder** to all examples for template consistency, or define a policy for when to use it vs hard-coded "you".

### 6.2 Schema Improvements

4. **Add an optional `context` field** to the input schema:
   ```yaml
   context:
     setting: "fantasy_tavern" | "sci-fi_lab" | "modern_office" | ...
     time_of_day: "morning" | "night"
     social_context: "private" | "public" | "formal"
   ```

5. **Add explicit relationship target** to the output schema (currently implicit):
   ```yaml
   relationship:
     user:
       trust: increases
   ```
   instead of:
   ```yaml
   relationship:
     trust: increases
   ```

6. **Normalize belief naming** with a namespace convention:
   ```yaml
   belief:
     self.efficacy: increases
     user.benevolence: decreases
     world.justice: increases
   ```

### 6.3 Content Additions (Priority Order)

| Priority | Addition | Rationale |
|---|---|---|
| 1 | Non-trust relationship examples | 75% of examples use trust; add authority, rivalry-without-trust, professional-respect, familial-duty |
| 2 | Primal state examples | Hunger, pain, cold, exhaustion, intoxication, illness — the README mentions these but the dataset doesn't cover them |
| 3 | Multi-turn interactions | 2-3 exchange fragments to test context accumulation |
| 4 | Counter-intuitive examples | Where the "obvious" emotional response is wrong (paranoid character distrusts genuine apology) |
| 5 | Taboo/moral violation examples | Character must choose between values, witnesses wrongdoing, experiences moral injury |
| 6 | Comedy/humor examples | Playful teasing, witty banter, joke-telling, mock-insults — almost none exist |
| 7 | Farewell/separation examples | Only 1 exists; important for closure dynamics |
| 8 | Character-as-initiator examples | Currently user always initiates; add examples where character message starts a new topic |
| 9 | Non-dyadic conversations | References to third parties, group dynamics, "they said" |
| 10 | More genre variety | At least: horror, war, courtroom, wilderness/survival, children's perspective |

### 6.4 Annotation Quality

11. **Calibrate magnitude usage.** Provide clearer guidelines: "Use `decreases_a_lot` only when the character's worldview is shaken, not for routine disappointment."

12. **Increase memory flag usage.** For the 76 interpersonal conflict examples, at least 40-50 should have `memory: reinforce_previous_conflict`.

13. **Decrease reflection overuse.** Reserve `reflection: required` for genuinely charged moments (betrayal, confession, ultimatum) — should be more like 40-50 examples, not 94.

14. **Add minimal effective outputs.** Ensure every output has at least 3-4 change entries. Single-entry outputs (example_03, example_19) don't provide enough signal.

---

## 7. Summary Statistics

| Metric | Value |
|---|---|
| Total examples | 150 |
| Unique examples | 136 (7 duplicate pairs) |
| Unique personality traits | 257 |
| Unique emotions (input) | 93 |
| Unique emotions (output) | 93 |
| Unique beliefs | 153 |
| Unique relationship keys | 49 |
| Genres represented | ~14 |
| With action markers | 33 (22%) |
| With `{{{User}}}` placeholder | 8 (5%) |
| No previous_character_message | 4 (3%) |
| No changes output | 13 (9%) |
| With memory flag | 30 (20%) |
| With reflection flag | 94 (63%) |
| Average dialogue length | 232 chars |

**Overall assessment:** The dataset is a solid first draft for teaching an LLM the basics of emotional state transitions in dyadic interpersonal conflict. It needs more **primal drives**, **setting diversity**, **non-trust relationships**, and **genre breadth** before it can train a generalizable cognitive state compiler. The annotation quality is good for a first pass but has magnitude calibration issues and inconsistent memory/reflection flag usage.
