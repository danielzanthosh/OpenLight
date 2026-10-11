"""Subagent delegation engine for running isolated background tasks."""

from typing import Any, Callable, Dict, Optional


class SubagentRunner:
    """Spawns an isolated subagent worker for parallel or delegated tasks."""

    def __init__(self, agent_factory: Callable[[], Any]):
        self.agent_factory = agent_factory

    def run_subagent(self, task: str, role: str = "Assistant Worker") -> str:
        """Run subagent with its own fresh context and return output."""
        worker = self.agent_factory()
        prompt = f"[DELEGATED SUBTASK - Role: {role}]\nYour task is:\n{task}\nComplete this task and return a clear, concise summary of your findings and results."
        return worker.run_task(prompt)


class SubagentTool:
    """Tool allowing the agent to delegate subtasks to an isolated worker agent."""

    name = "delegate_task"
    description = "Delegate a focused task or research question to a separate subagent to run without cluttering the main conversation."

    def __init__(self, runner: SubagentRunner):
        self.runner = runner

    def get_tool_schema(self) -> Dict[str, Any]:
        return {
            "name": self.name,
            "description": self.description,
            "parameters": {
                "type": "object",
                "properties": {
                    "task": {
                        "type": "string",
                        "description": "Specific, actionable task description for the subagent.",
                    },
                    "role": {
                        "type": "string",
                        "description": "Role/persona for the worker (e.g. 'Code Researcher', 'Data Analyst').",
                    },
                },
                "required": ["task"],
            },
        }

    def execute(self, **kwargs) -> Dict[str, Any]:
        task = kwargs.get("task", "")
        role = kwargs.get("role", "Worker")
        if not task:
            return {"success": False, "output": "Subtask description is required."}

        try:
            summary = self.runner.run_subagent(task=task, role=role)
            return {"success": True, "output": f"[Subagent '{role}' Completed]\n{summary}"}
        except Exception as ex:
            return {"success": False, "output": f"Error running subagent: {str(ex)}"}
