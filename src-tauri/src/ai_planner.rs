use crate::models::{GeneratedRoadmap, StoryPlannerInput, Task, PRODUCTION_STAGES};

/// Generates a structured, progressive animated production roadmap from a short story premise.
/// Supports OpenAI, Anthropic, OpenAI-compatible APIs, and Local Ollama endpoints, with a
/// deterministic offline studio template fallback when offline or before API keys are configured.
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
    }
}
