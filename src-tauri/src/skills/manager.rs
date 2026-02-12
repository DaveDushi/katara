use std::path::PathBuf;

use crate::error::KataraError;
use crate::skills::parser::{parse_skill, ParsedSkill};

/// Discover all skill files in a directory (recursive glob for *.md).
pub fn list_skills(skills_dir: &str) -> Result<Vec<ParsedSkill>, KataraError> {
    let pattern = format!("{}/**/*.md", skills_dir);
    let mut skills = Vec::new();

    for entry in glob::glob(&pattern).map_err(|e| KataraError::Skill(e.to_string()))? {
        if let Ok(path) = entry {
            let content = std::fs::read_to_string(&path).map_err(KataraError::Io)?;
            // Only include files that have valid frontmatter
            match parse_skill(&content, &path.display().to_string()) {
                Ok(skill) => skills.push(skill),
                Err(_) => continue, // Skip non-skill markdown files
            }
        }
    }

    Ok(skills)
}

/// Read and parse a single skill file.
pub fn read_skill(path: &str) -> Result<ParsedSkill, KataraError> {
    let content = std::fs::read_to_string(path).map_err(KataraError::Io)?;
    parse_skill(&content, path)
}

/// Write skill content to a file (creates parent dirs if needed).
pub fn write_skill(path: &str, content: &str) -> Result<(), KataraError> {
    // Validate the content parses correctly before writing
    let _ = parse_skill(content, path)?;

    let path_buf = PathBuf::from(path);
    if let Some(parent) = path_buf.parent() {
        std::fs::create_dir_all(parent).map_err(KataraError::Io)?;
    }
    std::fs::write(path, content).map_err(KataraError::Io)?;
    Ok(())
}

/// Delete a skill file.
pub fn delete_skill(path: &str) -> Result<(), KataraError> {
    std::fs::remove_file(path).map_err(KataraError::Io)?;
    Ok(())
}
