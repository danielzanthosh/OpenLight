"""Core OpenLight Agent perception-reasoning-action loop."""

from typing import Any, Callable, Dict, List, Optional
from openlight.agent.memory import MemoryManager, MemoryTool
from openlight.agent.skills import SkillsManager, SkillTool
from openlight.agent.subagent import SubagentRunner, SubagentTool
from openlight.config import OpenLightConfig
from openlight.providers import BaseLLMProvider, LLMMessage, ToolCall, get_provider
from openlight.tools.tool_registry import ToolRegistry


HERMES_SYSTEM_PROMPT = """You are OpenLight, an autonomous AI agent with real-time screen control (Computer-Using Agent) and full developer tools.

## Core Capabilities
1. **Desktop & Screen Control (CUA)**: You can see the user's screen and interact with their desktop using the `computer_use` tool.
   - Flow: **Capture first** -> examine screenshot and detected UI element indices (#N) -> **Actuate** (click element index or coordinates, type text, press hotkeys) -> **Verify** outcome.
   - For UI elements detected by accessibility inspection, ALWAYS prefer targeting `element=N` over raw pixel coordinates for maximum precision.
   - You can perform complex desktop workflows: opening applications, filling forms, clicking buttons, automating browser tasks.
2. **Terminal & Developer Automation**: You have a PowerShell/Bash `terminal` tool to run commands, run tests, install packages, and manage files.
3. **Filesystem Tools**: You can read (`read_file`), write (`write_file`), patch (`patch_file`), and search (`search_files`) files.
4. **Persistent Memory & Skills**: You can persist facts, user preferences, and learned workflows across sessions using `memory` and `skill_manage`.
5. **Subagents**: You can spawn isolated workers for parallel or deep tasks via `delegate_task`.

## Guidelines
- Always verify the outcome of actions. After clicking a button or running a command, verify the new state before declaring completion.
- When controlling the screen, proceed methodically step-by-step.
- Keep your thoughts concise, focused, and goal-driven.
"""


class OpenLightAgent:
    """The central agent orchestrator."""

    def __init__(
        self,
        config: Optional[OpenLightConfig] = None,
        provider: Optional[BaseLLMProvider] = None,
        tool_registry: Optional[ToolRegistry] = None,
        on_text_chunk: Optional[Callable[[str], None]] = None,
        on_tool_start: Optional[Callable[[str, Dict[str, Any]], None]] = None,
        on_tool_finish: Optional[Callable[[str, Dict[str, Any]], None]] = None,
    ):
        self.config = config or OpenLightConfig.load()
        self.provider = provider or get_provider(config=self.config)
        self.tool_registry = tool_registry or ToolRegistry.create_default(
            include_computer_use=True,
            monitor_index=self.config.default_monitor_index,
            cwd=self.config.working_directory,
        )

        # Hermes memory and skills
        self.memory_manager = MemoryManager()
        self.skills_manager = SkillsManager()
        self.tool_registry.register(MemoryTool(self.memory_manager))
        self.tool_registry.register(SkillTool(self.skills_manager))

        # Subagent support
        self.subagent_runner = SubagentRunner(agent_factory=self._create_subagent_worker)
        self.tool_registry.register(SubagentTool(self.subagent_runner))

        # Event callbacks
        self.on_text_chunk = on_text_chunk
        self.on_tool_start = on_tool_start
        self.on_tool_finish = on_tool_finish

        self.messages: List[LLMMessage] = []

    def _create_subagent_worker(self) -> "OpenLightAgent":
        """Instantiate a child worker agent with its own context."""
        worker_registry = ToolRegistry.create_default(
            include_computer_use=False,  # Subagents focus on research/code
            cwd=self.config.working_directory,
        )
        return OpenLightAgent(
            config=self.config,
            provider=self.provider,
            tool_registry=worker_registry,
        )

    def _build_system_prompt(self) -> str:
        prompt_parts = [HERMES_SYSTEM_PROMPT]

        # Inject persistent memories if any
        mem_block = self.memory_manager.format_for_prompt()
        if mem_block:
            prompt_parts.append(mem_block)

        # Inject discovered skills if any
        skills = self.skills_manager.discover_skills()
        if skills:
            skill_lines = ["\n--- Available Skills ---"]
            for s in skills:
                skill_lines.append(f"- **{s.name}**: {s.description}")
            prompt_parts.append("\n".join(skill_lines))

        return "\n\n".join(prompt_parts)

    def run_task(self, prompt: str) -> str:
        """Run agent execution loop for a user task prompt."""
        self.messages.append(LLMMessage(role="user", content=prompt))
        system_instruction = self._build_system_prompt()
        tools_schema = self.tool_registry.list_schemas()

        final_response_text = ""
        turns = 0

        while turns < self.config.max_turns:
            turns += 1

            # Call LLM provider
            resp = self.provider.generate_turn(
                messages=self.messages,
                tools_schema=tools_schema,
                system_instruction=system_instruction,
            )

            if resp.content:
                final_response_text = resp.content
                if self.on_text_chunk:
                    self.on_text_chunk(resp.content)

            # Record assistant message
            self.messages.append(
                LLMMessage(
                    role="assistant",
                    content=resp.content,
                    tool_calls=resp.tool_calls if resp.tool_calls else None,
                )
            )

            # If no tool calls, model finished its response
            if not resp.tool_calls:
                break

            # Execute each requested tool call
            for tc in resp.tool_calls:
                if self.on_tool_start:
                    self.on_tool_start(tc.name, tc.arguments)

                result = self.tool_registry.execute_tool(tc.name, tc.arguments)

                if self.on_tool_finish:
                    self.on_tool_finish(tc.name, result)

                output_text = str(result.get("output", ""))
                image_b64 = result.get("image_base64")

                # Append tool result back to message history
                self.messages.append(
                    LLMMessage(
                        role="tool",
                        content=output_text,
                        tool_call_id=tc.id,
                        image_base64=image_b64,
                    )
                )

        return final_response_text
