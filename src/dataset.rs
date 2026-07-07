use std::path::Path;

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
