# StoryQuest Studio — MCP & External Animation Software Guide

## Philosophy: Safe, Honest, Auditable Automation
StoryQuest Studio provides a modular integration layer (`src-tauri/src/mcp_integration.rs`) designed for solo animators using:
- **Blender 4.2+** (`Blender MCP`)
- **Moho Pro 14** (`Moho Pro 14 MCP`)
- **Rigby2D** (`Rigby2D MCP`)
- **Claude Code CLI** & **OpenCode Agent**

### Security Guarantees
1. **No Arbitrary Shell Execution**: The frontend cannot execute arbitrary shell commands. Only pre-configured tool IDs and verified local project subfolders can be invoked.
2. **Explicit Human Approval Gate**: Any action that modifies files or invokes an external agent with write permissions requires explicit user approval.
3. **Honest Status Reporting**: If an executable path or MCP server is not installed or offline, StoryQuest Studio exports the structured task packet (`00_Management/task_export_<id>.json`) and logs `exported_only` in `agent_execution_logs`—it never falsely reports that an external automation ran when it did not.
