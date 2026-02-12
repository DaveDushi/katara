use crate::error::KataraError;
use crate::skills::manager as skill_mgr;
use crate::skills::parser::ParsedSkill;

#[tauri::command]
pub async fn list_skills(skills_dir: Option<String>) -> Result<Vec<ParsedSkill>, KataraError> {
    let dir = skills_dir.unwrap_or_else(|| {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".claude")
            .join("skills")
            .display()
            .to_string()
    });
    skill_mgr::list_skills(&dir)
}

#[tauri::command]
pub async fn read_skill(path: String) -> Result<ParsedSkill, KataraError> {
    skill_mgr::read_skill(&path)
}

#[tauri::command]
pub async fn write_skill(path: String, content: String) -> Result<(), KataraError> {
    skill_mgr::write_skill(&path, &content)
}

#[tauri::command]
pub async fn delete_skill(path: String) -> Result<(), KataraError> {
    skill_mgr::delete_skill(&path)
}
