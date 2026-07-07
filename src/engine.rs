use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::spec::{Magnitude, MemoryAction, ReflectionAction, StateChanges};

// ── Character State ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub strength: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub emotions: HashMap<String, f32>,
    pub relationships: HashMap<String, HashMap<String, f32>>,
    pub beliefs: HashMap<String, f32>,
    pub memory: Vec<MemoryEntry>,
    pub reflection_pending: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterState {
    pub personality: Vec<String>,
    pub emotions: HashMap<String, f32>,
    pub relationships: HashMap<String, HashMap<String, f32>>,
    pub beliefs: HashMap<String, f32>,
    pub memory: Vec<MemoryEntry>,
    pub reflection_pending: bool,
    pub history: Vec<StateSnapshot>,
}

impl CharacterState {
    pub fn new(personality: Vec<String>) -> Self {
        CharacterState {
            personality,
            emotions: HashMap::new(),
            relationships: HashMap::new(),
            beliefs: HashMap::new(),
            memory: Vec::new(),
            reflection_pending: false,
            history: Vec::new(),
        }
    }

    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(json)?)
    }
}

// ── IR Operations ────────────────────────────────────────────────────────

pub enum IrOp {
    Emotion {
        name: String,
        magnitude: Magnitude,
    },
    Relationship {
        target: String,
        trait_name: String,
        magnitude: Magnitude,
    },
    Belief {
        identifier: String,
        magnitude: Magnitude,
    },
    ReinforceMemory {
        entry_id: String,
    },
    StartReflection,
}

// ── Conversion from dataset types ────────────────────────────────────────

pub fn ops_from_state_changes(changes: &StateChanges) -> Vec<IrOp> {
    let mut ops = Vec::new();

    if let Some(emotions) = &changes.emotion {
        for (name, magnitude) in emotions {
            ops.push(IrOp::Emotion {
                name: name.clone(),
                magnitude: *magnitude,
            });
        }
    }

    if let Some(traits) = &changes.relationship {
        for (trait_name, magnitude) in traits {
            ops.push(IrOp::Relationship {
                target: "user".to_string(),
                trait_name: trait_name.clone(),
                magnitude: *magnitude,
            });
        }
    }

    if let Some(beliefs) = &changes.belief {
        for (identifier, magnitude) in beliefs {
            ops.push(IrOp::Belief {
                identifier: identifier.clone(),
                magnitude: *magnitude,
            });
        }
    }

    if let Some(memory) = &changes.memory {
        match memory {
            MemoryAction::ReinforcePreviousConflict => {
                ops.push(IrOp::ReinforceMemory {
                    entry_id: "previous_conflict".to_string(),
                });
            }
        }
    }

    if let Some(reflection) = &changes.reflection {
        match reflection {
            ReflectionAction::Required => {
                ops.push(IrOp::StartReflection);
            }
        }
    }

    ops
}

// ── Delta constants ──────────────────────────────────────────────────────

fn magnitude_delta(m: Magnitude) -> f32 {
    match m {
        Magnitude::DecreasesALot => -0.20,
        Magnitude::Decreases => -0.10,
        Magnitude::DecreasesALittle => -0.04,
        Magnitude::IncreasesALittle => 0.04,
        Magnitude::Increases => 0.10,
        Magnitude::IncreasesALot => 0.20,
    }
}

fn clamp(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

// ── Engine ───────────────────────────────────────────────────────────────

pub struct Engine;

impl Engine {
    pub fn apply_state(state: &CharacterState, ops: &[IrOp]) -> CharacterState {
        let mut new_state = state.clone();

        let snapshot = StateSnapshot {
            emotions: state.emotions.clone(),
            relationships: state.relationships.clone(),
            beliefs: state.beliefs.clone(),
            memory: state.memory.clone(),
            reflection_pending: state.reflection_pending,
        };

        for op in ops {
            match op {
                IrOp::Emotion { name, magnitude } => {
                    let entry = new_state.emotions.entry(name.clone()).or_insert(0.5);
                    *entry = clamp(*entry + magnitude_delta(*magnitude));
                }
                IrOp::Relationship {
                    target,
                    trait_name,
                    magnitude,
                } => {
                    let traits = new_state
                        .relationships
                        .entry(target.clone())
                        .or_default();
                    let entry = traits.entry(trait_name.clone()).or_insert(0.5);
                    *entry = clamp(*entry + magnitude_delta(*magnitude));
                }
                IrOp::Belief { identifier, magnitude } => {
                    let entry = new_state
                        .beliefs
                        .entry(identifier.clone())
                        .or_insert(0.5);
                    *entry = clamp(*entry + magnitude_delta(*magnitude));
                }
                IrOp::ReinforceMemory { entry_id } => {
                    if let Some(entry) = new_state.memory.iter_mut().find(|e| e.id == *entry_id) {
                        entry.strength = clamp(entry.strength + 0.10);
                    } else {
                        new_state.memory.push(MemoryEntry {
                            id: entry_id.clone(),
                            strength: 0.4,
                        });
                    }
                }
                IrOp::StartReflection => {
                    new_state.reflection_pending = true;
                }
            }
        }

        new_state.history.push(snapshot);
        new_state
    }
}
