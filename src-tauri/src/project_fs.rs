use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const PROJECT_SUBFOLDERS: [&str; 13] = [
    "00_Management",
    "01_Story",
    "02_Audio",
    "03_Visual_Style",
    "04_Characters",
    "05_Rigging",
    "06_Animation",
    "07_Environments",
    "08_Storyboards",
    "09_Scenes",
    "10_Editing",
    "11_Render",
    "12_YouTube",
];

pub fn sanitize_folder_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .collect()
}

pub fn initialize_campaign_directories(base_projects_dir: &Path, campaign_folder_name: &str) -> Result<PathBuf, String> {
    let safe_name = sanitize_folder_name(campaign_folder_name);
    let campaign_path = base_projects_dir.join(safe_name);
    fs::create_dir_all(&campaign_path)
        .map_err(|e| format!("Failed to create campaign directory {:?}: {}", campaign_path, e))?;

    for subfolder in PROJECT_SUBFOLDERS.iter() {
        let sub_path = campaign_path.join(subfolder);
        fs::create_dir_all(&sub_path)
            .map_err(|e| format!("Failed to create subfolder {:?}: {}", sub_path, e))?;
    }

    let manifest_path = campaign_path.join("00_Management").join("project_manifest.json");
    if !manifest_path.exists() {
        let initial_manifest = serde_json::json!({
            "application": "StoryQuest Studio",
            "schema_version": "1.0.0",
            "folder_structure": PROJECT_SUBFOLDERS,
            "created_at": chrono::Utc::now().to_rfc3339()
        });
        let _ = fs::write(
            &manifest_path,
            serde_json::to_string_pretty(&initial_manifest).unwrap_or_default(),
        );
    }

    Ok(campaign_path)
}

pub fn open_folder_in_explorer(folder_path: &Path) -> Result<(), String> {
    if !folder_path.exists() {
        return Err(format!("Folder does not exist: {:?}", folder_path));
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer.exe")
            .arg(folder_path)
            .spawn()
            .map_err(|e| format!("Failed to launch Windows Explorer: {}", e))?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = Command::new("xdg-open").arg(folder_path).spawn();
    }
    Ok(())
}
