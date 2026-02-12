use serde::{Deserialize, Serialize};

use crate::error::KataraError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedSkill {
    pub file_path: String,
    pub metadata: SkillMetadata,
    pub prompt_template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub inputs: Vec<SkillInput>,
    #[serde(default)]
    pub outputs: Vec<SkillOutput>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInput {
    pub name: String,
    #[serde(default)]
    pub label: String,
    #[serde(rename = "type", default = "default_text")]
    pub input_type: String,
    #[serde(default)]
    pub required: bool,
    pub default: Option<serde_json::Value>,
    pub options: Option<Vec<String>>,
    pub placeholder: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillOutput {
    pub name: String,
    #[serde(default)]
    pub label: String,
    #[serde(rename = "type", default = "default_text")]
    pub output_type: String,
}

fn default_true() -> bool {
    true
}

fn default_text() -> String {
    "text".into()
}

/// Parse a skill markdown file with YAML frontmatter.
///
/// Expected format:
/// ```markdown
/// ---
/// name: My Skill
/// description: Does something
/// inputs:
///   - name: input1
///     type: text
/// outputs:
///   - name: output1
///     type: markdown
/// ---
///
/// Prompt template content here...
/// ```
pub fn parse_skill(content: &str, file_path: &str) -> Result<ParsedSkill, KataraError> {
    // Split on --- frontmatter delimiters
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return Err(KataraError::Skill(format!(
            "No YAML frontmatter found in {}",
            file_path
        )));
    }

    // Find the closing ---
    let after_first = &trimmed[3..];
    let end_idx = after_first
        .find("\n---")
        .ok_or_else(|| KataraError::Skill(format!("Unclosed frontmatter in {}", file_path)))?;

    let yaml_str = &after_first[..end_idx];
    let prompt_template = after_first[end_idx + 4..].trim().to_string();

    let metadata: SkillMetadata =
        serde_yaml::from_str(yaml_str).map_err(|e| KataraError::Skill(e.to_string()))?;

    Ok(ParsedSkill {
        file_path: file_path.to_string(),
        metadata,
        prompt_template,
    })
}
