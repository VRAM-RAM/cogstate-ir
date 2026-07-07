use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ── Input format ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Input {
    pub character: CharacterSection,
    pub relationship: BTreeMap<String, BTreeMap<String, String>>,
    pub current_state: BTreeMap<String, String>,
    pub previous_character_message: Option<String>,
    pub user_message: String,
}

#[derive(Debug, Deserialize)]
pub struct CharacterSection {
    pub personality: Vec<String>,
}

// ── Output format (target IR) ─────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
pub struct Target {
    pub state_changes: StateChanges,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StateChanges {
    pub emotion: Option<BTreeMap<String, Magnitude>>,
    pub relationship: Option<BTreeMap<String, Magnitude>>,
    pub belief: Option<BTreeMap<String, Magnitude>>,
    pub memory: Option<MemoryAction>,
    pub reflection: Option<ReflectionAction>,
}

// ── IR vocabulary ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Magnitude {
    IncreasesALot,
    Increases,
    IncreasesALittle,
    DecreasesALittle,
    Decreases,
    DecreasesALot,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryAction {
    ReinforcePreviousConflict,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReflectionAction {
    Required,
}

// ── Validation ────────────────────────────────────────────────────────────

impl Input {
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.character.personality.is_empty() {
            errors.push("character.personality must have at least one trait".into());
        }
        if self.relationship.is_empty() {
            errors.push("relationship must specify at least one target".into());
        }
        if self.current_state.is_empty() {
            errors.push("current_state must not be empty".into());
        }
        if self.user_message.is_empty() {
            errors.push("user_message must not be empty".into());
        }
        errors
    }
}
