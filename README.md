# OpenLight 🌟

> **High-Performance Autonomous AI Agent in Rust with Desktop Screen Control (CUA), Hermes Reasoning Loop, and Multi-Provider LLMs.**

OpenLight connects multimodal AI models directly to your operating system, written in **pure Rust** for ultra-low latency and minimal footprint.

---

## 🚀 Key Capabilities

1. **Desktop Screen & OS Control (Computer-Using Agent)**:
   - High-performance multi-monitor screen capture via `xcap` and `image` (JPEG base64 encoded).
   - Virtual mouse & keyboard actuation via native Windows `SendInput`: click (single, double, right), move, drag, scroll, type, and hotkeys.
   - Set-of-Marks visual coordinate grounding.
   - Built-in hardware fail-safe: slam mouse into top-left corner to immediately abort execution.
2. **Hermes-Style Autonomous Loop**:
   - Tokio-driven asynchronous perception-reasoning-action loop.
   - PowerShell / Bash execution with timeout protection (`terminal`).
   - File manipulation: `read_file`, `write_file`, `patch_file`, `search_files`.
   - Persistent memory across sessions stored in `~/.openlight/memory.json`.
   - Skill discovery and creation system (`SKILL.md` with YAML frontmatter).
   - Subagent task delegation.
3. **Multi-Provider LLMs**:
   - **OpenAI**: GPT-4o, o3, CUA endpoints.
   - **Anthropic**: Claude 3.5 Sonnet / Claude 3.7 Sonnet (Computer Use).
   - **Google Gemini**: Gemini 2.5 Pro / Flash with high-res vision & function calling.
   - **OpenRouter & Ollama**: Local open-weights models.

---

## ⚡ Quick Start

### Build & Run

```powershell
# Build release binary (openlight.exe)
cargo build --release

# Run interactive assistant
.\openlight.exe

# Run one-shot task
.\openlight.exe -p "Capture the screen and summarize the active window"

# Select provider & model
.\openlight.exe --provider gemini --model gemini-2.5-pro
.\openlight.exe --provider anthropic --model claude-3-7-sonnet-20250219
.\openlight.exe --provider openai --model gpt-4o

# Configure API keys
.\openlight.exe config --set-key gemini_api_key --value "YOUR_KEY"
```

---

## 🧪 Running Tests

```powershell
cargo test
```

All 9 integration and unit tests cover:
- Filesystem inspection, creation, and exact chunk patching
- PowerShell terminal subprocess execution and timeout safety
- Screen capture, monitor enumeration, and coordinate scaling
- Persistent memory store, recall, and deletion
- Skill discovery and authoring
