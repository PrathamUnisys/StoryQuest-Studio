use crate::credentials::retrieve_api_key;
use crate::models::{GeneratedRoadmap, StoryPlannerInput, Task, PRODUCTION_STAGES};

pub const ROADMAP_JSON_SCHEMA_HINT: &str = r#"Respond with ONLY valid JSON (no markdown fences) matching:
{"story_analysis":"...","improved_premise":"...","narrative_structure":"...","stages":[{"stage_number":1,"title":"...","description":"...","objective":"...","ai_instructions":"...","human_responsibilities":"...","expected_output":"...","estimated_effort_mins":30,"difficulty":"medium","xp_reward":25}]}
Include exactly the requested number of stages, numbered 1..N in order."#;

/// Generates a structured, progressive animated production roadmap from a short story premise.
/// Tries the active AI provider first (OpenAI / Anthropic / OpenAI-compatible / local Ollama
/// via JSON mode); on any failure or missing key, falls back to the deterministic offline
/// studio template so the app works 100% offline. `source` reports which path produced it.
pub async fn generate_roadmap(
    input: &StoryPlannerInput,
    provider: Option<&crate::models::AiProviderConfig>,
) -> GeneratedRoadmap {
    if let Some(cfg) = provider {
        if let Ok(roadmap) = call_provider(input, cfg).await {
            return roadmap;
        }
    }
    let mut roadmap = build_progressive_production_roadmap(input);
    roadmap.source = "template".to_string();
    roadmap
}

fn build_prompt(input: &StoryPlannerInput) -> (String, String) {
    let genre = input.genre.clone().unwrap_or_else(|| "Dark Fantasy Animation".to_string());
    let stage_limit = input.target_stage_limit.unwrap_or(15).clamp(1, 15);
    let system = format!(
        "You are StoryQuest Studio's production planner for solo animated YouTube creators. \
         Plan a {stage_limit}-stage production roadmap for '{title}' ({genre}, ~{runtime} min). \
         Stages must follow the canonical 15-stage pipeline in order (Story Foundation, Script Development, \
         Voice Design, Visual Direction, Character Design, Character Rigging, Animation Library, Environment \
         Creation, Storyboarding, Scene Animation, Audio Production, Video Editing, Final Rendering, \
         Quality Assurance, YouTube Publication). Keep each stage scoped for a solo creator.",
        stage_limit = stage_limit,
        title = input.story_title,
        genre = genre,
        runtime = input.desired_runtime_mins,
    );
    let user = format!(
        "Story title: {}\nPremise: {}\nCreative instructions: {}\n\n{}",
        input.story_title,
        input.short_premise,
        input.creative_instructions.clone().unwrap_or_default(),
        ROADMAP_JSON_SCHEMA_HINT,
    );
    (system, user)
}

fn strip_code_fences(text: &str) -> String {
    let trimmed = text.trim();
    if let Some(rest) = trimmed.strip_prefix("```") {
        let rest = rest.strip_prefix("json").unwrap_or(rest);
        return rest.strip_suffix("```").unwrap_or(rest).trim().to_string();
    }
    trimmed.to_string()
}

fn hydrate_tasks(
    input: &StoryPlannerInput,
    stages_json: &serde_json::Value,
) -> Result<Vec<Task>, String> {
    let genre = input.genre.clone().unwrap_or_else(|| "Dark Fantasy Animation".to_string());
    let stage_limit = input.target_stage_limit.unwrap_or(15).clamp(1, 15);
    let by_number: std::collections::HashMap<i32, serde_json::Value> = stages_json
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|s| {
                    s.get("stage_number")
                        .and_then(|v| v.as_i64())
                        .map(|n| (n as i32, s.clone()))
                })
                .collect()
        })
        .unwrap_or_default();

    let mut tasks = Vec::new();
    let mut prev_task_id: Option<String> = None;

    for (stage_num, stage_name, skill_cat, folder) in PRODUCTION_STAGES.iter() {
        if *stage_num > stage_limit {
            break;
        }
        let task_id = format!("task_{}_s{:02}", input.episode_id, stage_num);
        let deps = prev_task_id.iter().cloned().collect::<Vec<_>>();
        let status = if *stage_num == 1 { "available" } else { "locked" };
        let model_stage = by_number.get(stage_num);
        let text = |key: &str, fallback: String| -> String {
            model_stage
                .and_then(|s| s.get(key))
                .and_then(|v| v.as_str())
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.to_string())
                .unwrap_or(fallback)
        };
        let int = |key: &str, fallback: i32| -> i32 {
            model_stage
                .and_then(|s| s.get(key))
                .and_then(|v| v.as_i64())
                .map(|n| n as i32)
                .filter(|n| *n > 0)
                .unwrap_or(fallback)
        };

        tasks.push(Task {
            id: task_id.clone(),
            campaign_id: input.campaign_id.clone(),
            episode_id: input.episode_id.clone(),
            stage_number: *stage_num,
            stage_name: stage_name.to_string(),
            title: text(
                "title",
                format!("Stage {:02}: {} — {}", stage_num, stage_name, input.story_title),
            ),
            description: text(
                "description",
                format!(
                    "Execute the {} deliverables for '{}' and store output in {}/.",
                    stage_name, input.story_title, folder
                ),
            ),
            objective: text(
                "objective",
                format!("Complete verified deliverable for {} stage.", stage_name),
            ),
            required_input: if *stage_num == 1 {
                "Initial story premise and campaign bible".to_string()
            } else {
                format!("Approved output from Stage {:02}", stage_num - 1)
            },
            ai_instructions: text(
                "ai_instructions",
                format!(
                    "Assist the creator with {} for '{}' ({}) while preserving visual & narrative continuity.",
                    stage_name, input.story_title, genre
                ),
            ),
            human_responsibilities: text(
                "human_responsibilities",
                format!(
                    "Review creative direction, finalize {} asset in {}, and approve milestone quality.",
                    stage_name, folder
                ),
            ),
            required_software: match *stage_num {
                1 | 2 | 14 => "StoryQuest Workshop / Markdown".to_string(),
                3 | 11 => "Audacity / Reaper / Voice Studio".to_string(),
                4 | 5 | 8 | 9 => "Krita / Photoshop / Blender Storypencil".to_string(),
                6 | 7 | 10 => "Moho Pro 14 / Blender 4.2".to_string(),
                12 | 13 => "DaVinci Resolve / Blender Video Sequence Editor".to_string(),
                _ => "YouTube Studio / Browser".to_string(),
            },
            expected_output: text(
                "expected_output",
                format!("{}/stage_{:02}_deliverable", folder, stage_num),
            ),
            estimated_effort_mins: int("estimated_effort_mins", if *stage_num <= 4 { 30 } else { 45 }),
            actual_effort_mins: 0,
            definition_of_done: format!(
                "Deliverable saved in {} and verified against continuity checklist.",
                folder
            ),
            completion_requirement_type: if *stage_num == 2 || *stage_num == 5 || *stage_num == 10 {
                "artifact".to_string()
            } else {
                "confirmation".to_string()
            },
            completion_artifact_value: String::new(),
            difficulty: text(
                "difficulty",
                if *stage_num == 10 || *stage_num == 13 || *stage_num == 15 {
                    "large".to_string()
                } else {
                    "medium".to_string()
                },
            ),
            xp_reward: int(
                "xp_reward",
                if *stage_num == 10 || *stage_num == 13 || *stage_num == 15 { 50 } else { 25 },
            ),
            skill_category: skill_cat.to_string(),
            status: status.to_string(),
            is_daily_quest: *stage_num <= 3,
            due_date: None,
            notes: String::new(),
            attached_files_json: "[]".to_string(),
            dependencies: deps,
            completed_at: None,
            approved_at: None,
        });

        prev_task_id = Some(task_id);
    }
    Ok(tasks)
}

fn parse_model_roadmap(input: &StoryPlannerInput, raw_text: &str) -> Result<GeneratedRoadmap, String> {
    let json: serde_json::Value =
        serde_json::from_str(&strip_code_fences(raw_text)).map_err(|e| format!("Model did not return valid JSON: {e}"))?;
    let story_analysis = json
        .get("story_analysis")
        .and_then(|v| v.as_str())
        .unwrap_or("AI-generated story analysis.").to_string();
    let improved_premise = json
        .get("improved_premise")
        .and_then(|v| v.as_str())
        .unwrap_or(&input.short_premise).to_string();
    let narrative_structure = json
        .get("narrative_structure")
        .and_then(|v| v.as_str())
        .unwrap_or("AI-generated narrative structure.").to_string();
    let stages = json.get("stages").ok_or_else(|| "Model JSON missing 'stages' array".to_string())?;
    let generated_tasks = hydrate_tasks(input, stages)?;
    if generated_tasks.is_empty() {
        return Err("Model returned no usable stages".to_string());
    }
    Ok(GeneratedRoadmap {
        story_analysis,
        improved_premise,
        narrative_structure,
        generated_tasks,
        source: "ai".to_string(),
    })
}

async fn call_openai_compatible(
    input: &StoryPlannerInput,
    cfg: &crate::models::AiProviderConfig,
    system: &str,
    user: &str,
) -> Result<GeneratedRoadmap, String> {
    let api_key = retrieve_api_key(&cfg.credential_key_name)?
        .filter(|k| !k.trim().is_empty())
        .unwrap_or_else(|| "not-needed-local".to_string());
    let url = format!("{}/chat/completions", cfg.base_url.trim_end_matches('/'));
    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .bearer_auth(&api_key)
        .json(&serde_json::json!({
            "model": cfg.default_model,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": user},
            ],
            "temperature": cfg.temperature,
            "max_tokens": cfg.max_tokens,
            "response_format": {"type": "json_object"},
        }))
        .send()
        .await
        .map_err(|e| format!("Provider request failed: {e}"))?;
    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Provider HTTP {status}: {text}"));
    }
    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Invalid provider response: {e}"))?;
    let content = body
        .get("choices")
        .and_then(|c| c.as_array())
        .and_then(|a| a.first())
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .ok_or_else(|| "Provider response missing message content".to_string())?;
    parse_model_roadmap(input, content)
}

async fn call_anthropic(
    input: &StoryPlannerInput,
    cfg: &crate::models::AiProviderConfig,
    system: &str,
    user: &str,
) -> Result<GeneratedRoadmap, String> {
    let api_key = retrieve_api_key(&cfg.credential_key_name)?
        .filter(|k| !k.trim().is_empty())
        .ok_or_else(|| "Anthropic API key is not configured".to_string())?;
    let url = format!("{}/messages", cfg.base_url.trim_end_matches('/'));
    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&serde_json::json!({
            "model": cfg.default_model,
            "max_tokens": cfg.max_tokens,
            "system": system,
            "messages": [{"role": "user", "content": user}],
            "temperature": cfg.temperature,
        }))
        .send()
        .await
        .map_err(|e| format!("Anthropic request failed: {e}"))?;
    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Anthropic HTTP {status}: {text}"));
    }
    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Invalid Anthropic response: {e}"))?;
    let content = body
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|a| a.iter().find(|b| b.get("type").and_then(|v| v.as_str()) == Some("text")))
        .and_then(|b| b.get("text"))
        .and_then(|t| t.as_str())
        .ok_or_else(|| "Anthropic response missing text content".to_string())?;
    parse_model_roadmap(input, content)
}

async fn call_provider(
    input: &StoryPlannerInput,
    cfg: &crate::models::AiProviderConfig,
) -> Result<GeneratedRoadmap, String> {
    let (system, user) = build_prompt(input);
    match cfg.provider_name.as_str() {
        "anthropic" => call_anthropic(input, cfg, &system, &user).await,
        _ => call_openai_compatible(input, cfg, &system, &user).await,
    }
}

/// Deterministic offline studio template (always available, no key, no network).
pub fn build_progressive_production_roadmap(input: &StoryPlannerInput) -> GeneratedRoadmap {
    let genre = input.genre.clone().unwrap_or_else(|| "Dark Fantasy Animation".to_string());
    let story_analysis = format!(
        "Arc & Pacing Analysis for '{}' ({}, ~{} min target): Strong central hook with clear visual silhouette potential. Recommended 3-act micro-structure (Hook 0:00-1:15, Escalation & Trial 1:15-6:00, Climax & Resolution 6:00-{}:00) to keep solo animation scope achievable.",
        input.story_title, genre, input.desired_runtime_mins, input.desired_runtime_mins
    );
    let improved_premise = format!(
        "{} — Structured for solo production using reusable 2.5D character rigs, modular multi-plane environments, and rhythm-locked animatics.",
        input.short_premise.trim()
    );
    let narrative_structure = "Act I: The Inciting Relic (Beat 1-3) -> Act II: Crossing the Obsidian Threshold (Beat 4-9) -> Act III: Resonance & Cliffhanger Tag (Beat 10-12)".to_string();

    let stage_limit = input.target_stage_limit.unwrap_or(15).clamp(1, 15);
    let mut tasks = Vec::new();
    let mut prev_task_id: Option<String> = None;

    for (stage_num, stage_name, skill_cat, folder) in PRODUCTION_STAGES.iter() {
        if *stage_num > stage_limit {
            break;
        }
        let task_id = format!("task_{}_s{:02}", input.episode_id, stage_num);
        let deps = prev_task_id.iter().cloned().collect::<Vec<_>>();
        let status = if *stage_num == 1 { "available" } else { "locked" };

        tasks.push(Task {
            id: task_id.clone(),
            campaign_id: input.campaign_id.clone(),
            episode_id: input.episode_id.clone(),
            stage_number: *stage_num,
            stage_name: stage_name.to_string(),
            title: format!("Stage {:02}: {} — {}", stage_num, stage_name, input.story_title),
            description: format!(
                "Execute the {} deliverables for '{}' and store output in {}/.",
                stage_name, input.story_title, folder
            ),
            objective: format!("Complete verified deliverable for {} stage.", stage_name),
            required_input: if *stage_num == 1 {
                "Initial story premise and campaign bible".to_string()
            } else {
                format!("Approved output from Stage {:02}", stage_num - 1)
            },
            ai_instructions: format!(
                "Assist the creator with {} for '{}' ({}) while preserving visual & narrative continuity.",
                stage_name, input.story_title, genre
            ),
            human_responsibilities: format!(
                "Review creative direction, finalize {} asset in {}, and approve milestone quality.",
                stage_name, folder
            ),
            required_software: match *stage_num {
                1 | 2 | 14 => "StoryQuest Workshop / Markdown".to_string(),
                3 | 11 => "Audacity / Reaper / Voice Studio".to_string(),
                4 | 5 | 8 | 9 => "Krita / Photoshop / Blender Storypencil".to_string(),
                6 | 7 | 10 => "Moho Pro 14 / Blender 4.2".to_string(),
                12 | 13 => "DaVinci Resolve / Blender Video Sequence Editor".to_string(),
                _ => "YouTube Studio / Browser".to_string(),
            },
            expected_output: format!("{}/stage_{:02}_deliverable", folder, stage_num),
            estimated_effort_mins: if *stage_num <= 4 { 30 } else { 45 },
            actual_effort_mins: 0,
            definition_of_done: format!("Deliverable saved in {} and verified against continuity checklist.", folder),
            completion_requirement_type: if *stage_num == 2 || *stage_num == 5 || *stage_num == 10 {
                "artifact".to_string()
            } else {
                "confirmation".to_string()
            },
            completion_artifact_value: String::new(),
            difficulty: if *stage_num == 10 || *stage_num == 13 || *stage_num == 15 {
                "large".to_string()
            } else {
                "medium".to_string()
            },
            xp_reward: if *stage_num == 10 || *stage_num == 13 || *stage_num == 15 { 50 } else { 25 },
            skill_category: skill_cat.to_string(),
            status: status.to_string(),
            is_daily_quest: *stage_num <= 3,
            due_date: None,
            notes: String::new(),
            attached_files_json: "[]".to_string(),
            dependencies: deps,
            completed_at: None,
            approved_at: None,
        });

        prev_task_id = Some(task_id);
    }

    GeneratedRoadmap {
        story_analysis,
        improved_premise,
        narrative_structure,
        generated_tasks: tasks,
        source: "template".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::StoryPlannerInput;

    fn input(stage_limit: Option<i32>) -> StoryPlannerInput {
        StoryPlannerInput {
            campaign_id: "camp_01".to_string(),
            episode_id: "ep_01".to_string(),
            story_title: "Test Story".to_string(),
            short_premise: "A relic awakens.".to_string(),
            genre: Some("Dark Fantasy".to_string()),
            desired_runtime_mins: 8,
            creative_instructions: None,
            target_stage_limit: stage_limit,
        }
    }

    #[test]
    fn strips_code_fences() {
        assert_eq!(strip_code_fences("```json\n{\"a\":1}\n```"), "{\"a\":1}");
        assert_eq!(strip_code_fences("  {\"a\":1}  "), "{\"a\":1}");
    }

    #[test]
    fn parses_model_json_and_hydrates_stages() {
        let raw = r#"{"story_analysis":"s","improved_premise":"p","narrative_structure":"n","stages":[{"stage_number":1,"title":"AI Title","description":"d","objective":"o","ai_instructions":"ai","human_responsibilities":"h","expected_output":"out","estimated_effort_mins":60,"difficulty":"small","xp_reward":10}]}"#;
        let roadmap = parse_model_roadmap(&input(Some(3)), raw).unwrap();
        assert_eq!(roadmap.source, "ai");
        assert_eq!(roadmap.generated_tasks.len(), 3);
        assert_eq!(roadmap.generated_tasks[0].title, "AI Title");
        assert_eq!(roadmap.generated_tasks[0].stage_name, "Story Foundation");
        assert_eq!(roadmap.generated_tasks[1].status, "locked");
        assert_eq!(roadmap.generated_tasks[0].dependencies.len(), 0);
        assert_eq!(roadmap.generated_tasks[1].dependencies.len(), 1);
    }

    #[test]
    fn rejects_invalid_model_json() {
        assert!(parse_model_roadmap(&input(None), "not json at all").is_err());
        assert!(parse_model_roadmap(&input(None), r#"{"foo":1}"#).is_err());
    }

    #[test]
    fn template_fallback_has_source_template() {
        let roadmap = build_progressive_production_roadmap(&input(Some(2)));
        assert_eq!(roadmap.source, "template");
        assert_eq!(roadmap.generated_tasks.len(), 2);
    }
}
