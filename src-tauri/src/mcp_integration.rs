use std::path::Path;
use std::process::Command;
use rusqlite::{params, Connection};
use uuid::Uuid;
use crate::models::{McpDispatchRequest, Task};

/// Exports structured Task JSON for external DCC tools (Blender MCP, Moho MCP, Rigby2D)
/// or coding agents (Claude Code, OpenCode) and logs every operation honestly.
pub fn execute_safe_integration_action(
    conn: &Connection,
    req: &McpDispatchRequest,
    task: &Task,
    project_folder: &Path,
) -> Result<String, String> {
    let (conn_name, exec_path, is_configured, req_approval, status): (String, String, i32, i32, String) = conn
        .query_row(
            "SELECT name, executable_path, is_configured, requires_approval_for_writes, status
             FROM agent_connections WHERE id = ?1",
            params![&req.connection_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
        )
        .map_err(|e| format!("Integration connection not found: {}", e))?;

    if req_approval != 0 && !req.human_approved {
        return Err(format!(
            "Explicit human approval is required before dispatching action '{}' to {}.",
            req.action_type, conn_name
        ));
    }

    let payload = serde_json::json!({
        "storyquest_version": "1.0.0",
        "mcp_protocol": "2024-11-05",
        "target_integration": conn_name,
        "action": req.action_type,
        "task": {
            "id": task.id,
            "stage_number": task.stage_number,
            "stage_name": task.stage_name,
            "title": task.title,
            "objective": task.objective,
            "ai_instructions": task.ai_instructions,
            "human_responsibilities": task.human_responsibilities,
            "required_software": task.required_software,
            "expected_output": task.expected_output
        }
    });

    let payload_str = serde_json::to_string_pretty(&payload).unwrap_or_default();
    let export_path = project_folder
        .join("00_Management")
        .join(format!("task_export_{}.json", task.id));

    if let Some(parent) = export_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&export_path, &payload_str);

    let (exec_status, log_msg) = if req.action_type == "export_json" || is_configured == 0 {
        (
            "exported_only",
            format!(
                "Exported structured task JSON to {:?}. ({} status: {})",
                export_path, conn_name, status
            ),
        )
    } else if req.action_type == "launch_software" && !exec_path.trim().is_empty() {
        let path_obj = Path::new(&exec_path);
        if path_obj.exists() {
            Command::new(path_obj)
                .spawn()
                .map_err(|e| format!("Failed to start {}: {}", conn_name, e))?;
            ("dispatched", format!("Launched {} with task packet {:?}", conn_name, export_path))
        } else {
            (
                "exported_only",
                format!(
                    "Executable '{}' not found on disk; exported structured task packet to {:?} without falsely reporting execution.",
                    exec_path, export_path
                ),
            )
        }
    } else {
        (
            "exported_only",
            format!("Prepared MCP instruction packet for {} at {:?}", conn_name, export_path),
        )
    };

    let log_id = format!("log_{}", Uuid::new_v4());
    conn.execute(
        "INSERT INTO agent_execution_logs (id, agent_connection_id, task_id, action_type, command_payload_json, human_approved, execution_status, output_log)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            log_id,
            &req.connection_id,
            &task.id,
            &req.action_type,
            &payload_str,
            if req.human_approved { 1 } else { 0 },
            exec_status,
            &log_msg
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(log_msg)
}
