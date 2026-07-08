use std::path::{Path, PathBuf};

use crate::spec::{Input, Target};

#[derive(Debug)]
#[allow(dead_code)]
pub struct Example {
    pub input: Input,
    pub target: Target,
    pub input_path: String,
    pub output_path: String,
}

pub fn load_example(input_path: &Path, output_path: &Path) -> anyhow::Result<Example> {
    let input_content = std::fs::read_to_string(input_path)?;
    let output_content = std::fs::read_to_string(output_path)?;

    let input: Input = serde_yaml::from_str(&input_content)?;
    let target: Target = serde_yaml::from_str(&output_content)?;

    Ok(Example {
        input,
        target,
        input_path: input_path.display().to_string(),
        output_path: output_path.display().to_string(),
    })
}

pub fn validate_example(example: &Example) -> Vec<String> {
    let mut errors = Vec::new();

    let input_errors = example.input.validate();
    for err in input_errors {
        errors.push(format!("input: {}", err));
    }

    errors
}

// ── validate-all ─────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct ValidationResult {
    pub input_path: String,
    pub output_path: String,
    pub errors: Vec<String>,
}

impl ValidationResult {
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

pub fn collect_pairs(dir: &Path, pairs: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let input = path.join("input.yaml");
        let output = path.join("output.yaml");
        if input.exists() && output.exists() {
            pairs.push(path);
        } else {
            collect_pairs(&path, pairs);
        }
    }
}

pub fn validate_all(dir: &Path) -> Vec<ValidationResult> {
    let mut pairs = Vec::new();
    collect_pairs(dir, &mut pairs);

    let mut results = Vec::new();
    for dir in &pairs {
        let input_path = dir.join("input.yaml");
        let output_path = dir.join("output.yaml");

        let result = match load_example(&input_path, &output_path) {
            Ok(example) => {
                let errors = validate_example(&example);
                ValidationResult {
                    input_path: example.input_path,
                    output_path: example.output_path,
                    errors,
                }
            }
            Err(e) => ValidationResult {
                input_path: input_path.display().to_string(),
                output_path: output_path.display().to_string(),
                errors: vec![format!("{}", e)],
            },
        };
        results.push(result);
    }
    results
}
