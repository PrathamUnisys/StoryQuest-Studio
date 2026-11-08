# StoryQuest Studio — Provider-Agnostic AI Configuration Guide

## Security Architecture
- **Zero Hardcoded Keys**: No API keys are ever stored in source code, environment files, or the SQLite `storyquest.db` database.
- **Windows Credential Manager**: All API secrets are stored in the OS vault via the Rust `keyring` crate (`service = "com.storyquest.studio.credentials"`).
- **100% Offline Core**: Task tracking, the 15-stage Quest Map, Focus Mode timers, XP/Streaks/Achievements, Reminders, and Deterministic Studio Roadmap Templates work completely offline without any API key.

## Supported Providers
1. **OpenAI API**: Base URL `https://api.openai.com/v1`, Default Model `gpt-4o`, Credential Key `StoryQuest_OpenAI_API_Key`.
2. **Anthropic Claude API**: Base URL `https://api.anthropic.com/v1`, Default Model `claude-3-5-sonnet-latest`, Credential Key `StoryQuest_Anthropic_API_Key`.
3. **OpenAI-Compatible Endpoints (OpenRouter, Groq, Together, DeepSeek)**: Configurable Base URL and Model ID.
4. **Local Model Endpoints (Ollama / LM Studio)**: Base URL `http://127.0.0.1:11434/v1` (Ollama) or `http://127.0.0.1:1234/v1` (LM Studio) — runs 100% locally on your GPU/CPU with zero cloud cost.
