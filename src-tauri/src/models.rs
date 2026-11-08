use serde::{Deserialize, Serialize};

pub const PRODUCTION_STAGES: [(i32, &str, &str, &str); 15] = [
    (1, "Story Foundation", "Storytelling", "01_Story"),
    (2, "Script Development", "Screenwriting", "01_Story"),
    (3, "Voice Design", "Sound Design", "02_Audio"),
    (4, "Visual Direction", "World Building", "03_Visual_Style"),
    (5, "Character Design", "Character Design", "04_Characters"),
    (6, "Character Rigging", "Character Rigging", "05_Rigging"),
    (7, "Animation Library", "Animation", "06_Animation"),
    (8, "Environment Creation", "Environment Design", "07_Environments"),
    (9, "Storyboarding", "Storytelling", "08_Storyboards"),
    (10, "Scene Animation", "Animation", "09_Scenes"),
    (11, "Audio Production", "Sound Design", "02_Audio"),
    (12, "Video Editing", "Editing", "10_Editing"),
    (13, "Final Rendering", "Production Management", "11_Render"),
    (14, "Quality Assurance", "Production Management", "00_Management"),
    (15, "YouTube Publication", "Production Management", "12_YouTube"),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub id: String,
    pub display_name: String,
    pub studio_name: String,
    pub current_level: i32,
    pub total_xp: i32,
    pub active_title: String,
    pub active_theme: String,
    pub avatar_badge: String,
    pub daily_xp_custom_tasks: i32,
    pub last_custom_xp_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Campaign {
    pub id: String,
    pub title: String,
    pub synopsis: String,
    pub genre: String,
    pub visual_style: String,
    pub world_info: String,
    pub target_runtime_minutes: i32,
    pub status: String,
    pub project_folder_path: String,
    pub progress_percent: f64,
    pub is_sample: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Episode {
    pub id: String,
    pub campaign_id: String,
    pub episode_number: i32,
    pub title: String,
    pub premise: String,
    pub narrative_structure: String,
    pub target_duration_sec: i32,
    pub current_stage: i32,
    pub roadmap_approved: bool,
    pub status: String,
    pub youtube_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub campaign_id: String,
    pub episode_id: String,
    pub stage_number: i32,
    pub stage_name: String,
    pub title: String,
    pub description: String,
    pub objective: String,
    pub required_input: String,
    pub ai_instructions: String,
    pub human_responsibilities: String,
    pub required_software: String,
    pub expected_output: String,
    pub estimated_effort_mins: i32,
    pub actual_effort_mins: i32,
    pub definition_of_done: String,
    pub completion_requirement_type: String,
    pub completion_artifact_value: String,
    pub difficulty: String,
    pub xp_reward: i32,
    pub skill_category: String,
    pub status: String,
    pub is_daily_quest: bool,
    pub due_date: Option<String>,
    pub notes: String,
    pub attached_files_json: String,
    pub dependencies: Vec<String>,
    pub completed_at: Option<String>,
    pub approved_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XpAwardResult {
    pub awarded: bool,
    pub xp_delta: i32,
    pub new_total_xp: i32,
    pub old_level: i32,
    pub new_level: i32,
    pub new_title: String,
    pub unlocked_achievements: Vec<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreakRecord {
    pub id: String,
    pub user_id: String,
    pub current_streak: i32,
    pub best_streak: i32,
    pub streak_freezes_available: i32,
    pub weekly_rest_day: i32,
    pub vacation_mode: bool,
    pub vacation_until: Option<String>,
    pub last_productive_date: Option<String>,
    pub history_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderConfig {
    pub id: String,
    pub title: String,
    pub reminder_type: String,
    pub task_id: Option<String>,
    pub schedule_time: String,
    pub days_of_week_json: String,
    pub specific_date: Option<String>,
    pub enabled: bool,
    pub snoozed_until: Option<String>,
    pub windows_task_name: String,
    pub last_triggered_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryPlannerInput {
    pub campaign_id: String,
    pub episode_id: String,
    pub story_title: String,
    pub short_premise: String,
    pub genre: Option<String>,
    pub desired_runtime_mins: i32,
    pub creative_instructions: Option<String>,
    pub target_stage_limit: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedRoadmap {
    pub story_analysis: String,
    pub improved_premise: String,
    pub narrative_structure: String,
    pub generated_tasks: Vec<Task>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpDispatchRequest {
    pub connection_id: String,
    pub task_id: String,
    pub action_type: String,
    pub human_approved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatorRank {
    pub level: i32,
    pub rank_name: String,
    pub min_published_videos: i32,
    pub badge_icon: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatorProgress {
    pub user_id: String,
    pub rank_level: i32,
    pub published_videos_total: i32,
    pub shorts_count: i32,
    pub long_form_count: i32,
    pub subscriber_count: Option<i64>,
    pub total_channel_views: Option<i64>,
    pub total_watch_minutes: Option<i64>,
    pub last_recalculated_at: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Achievement {
    pub id: String,
    pub code: String,
    pub title: String,
    pub description: String,
    pub category: String,
    pub xp_bonus: i32,
    pub badge_svg_key: String,
    pub cosmetic_reward: Option<String>,
    pub unlocked: bool,
    pub unlocked_at: Option<String>,
    pub progress_current: i32,
    pub progress_target: i32,
    pub condition_type: String,
    pub condition_value: i32,
    pub condition_scope: String,
    pub progress_available: bool,
    pub is_hidden: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AchievementUnlock {
    pub achievement_id: String,
    pub code: String,
    pub title: String,
    pub xp_bonus: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AchievementNotification {
    pub id: String,
    pub achievement_id: Option<String>,
    pub kind: String,
    pub title: String,
    pub body: String,
    pub items_json: String,
    pub presented_at: Option<String>,
    pub acknowledged_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetStreakPrefsInput {
    pub weekly_rest_day: i32,
    pub vacation_mode: bool,
    pub vacation_until: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcknowledgeNotificationsInput {
    pub notification_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingItem {
    pub key: String,
    pub value: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub identifier: String,
    pub data_dir: String,
    pub db_path: String,
    pub schema_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TaskCounts {
    pub total: i32,
    pub locked: i32,
    pub available: i32,
    pub in_progress: i32,
    pub review: i32,
    pub approved: i32,
    pub completed: i32,
    pub blocked: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YoutubeStatus {
    pub connected: bool,
    pub channel_title: Option<String>,
    pub last_sync_at: Option<String>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityDay {
    pub date: String,
    pub xp: i32,
    pub tasks_completed: i32,
    pub focus_minutes: i32,
    pub active: bool,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapState {
    pub profile: UserProfile,
    pub creator_progress: CreatorProgress,
    pub creator_rank: CreatorRank,
    pub creator_ranks: Vec<CreatorRank>,
    pub settings: Vec<SettingItem>,
    pub active_campaign: Option<Campaign>,
    pub active_episode: Option<Episode>,
    pub task_counts: TaskCounts,
    pub episode_progress_percent: f64,
    pub streak: StreakRecord,
    pub achievements: Vec<Achievement>,
    pub deadlines: Vec<Task>,
    pub weekly_activity: Vec<ActivityDay>,
    pub youtube: YoutubeStatus,
    pub app_info: AppInfo,
    pub today: String,
    pub onboarding_completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThresholdUpdate {
    pub level: i32,
    pub min_published_videos: i32,
}
