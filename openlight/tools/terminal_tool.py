"""Terminal and Shell execution tool."""

import subprocess
import sys
from typing import Any, Dict, Optional


class TerminalTool:
    """Executes commands in PowerShell (Windows) or Bash/sh (Unix)."""

    name = "terminal"
    description = "Execute shell / terminal commands (PowerShell on Windows, Bash on Linux/macOS)."

    def __init__(self, default_cwd: Optional[str] = None):
        self.default_cwd = default_cwd

    def get_tool_schema(self) -> Dict[str, Any]:
        return {
            "name": self.name,
            "description": self.description,
            "parameters": {
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The command string to execute in the shell.",
                    },
                    "timeout_seconds": {
                        "type": "integer",
                        "description": "Maximum seconds to wait before terminating the command (default: 60).",
                    },
                    "cwd": {
                        "type": "string",
                        "description": "Working directory for the command execution.",
                    },
                },
                "required": ["command"],
            },
        }

    def execute(self, **kwargs) -> Dict[str, Any]:
        command = kwargs.get("command", "")
        timeout = kwargs.get("timeout_seconds", 60)
        cwd = kwargs.get("cwd") or self.default_cwd

        if not command:
            return {"success": False, "output": "No command provided."}

        try:
            if sys.platform == "win32":
                args = ["powershell", "-NoProfile", "-NonInteractive", "-Command", command]
            else:
                args = ["/bin/bash", "-c", command]

            res = subprocess.run(
                args,
                cwd=cwd,
                capture_output=True,
                text=True,
                timeout=timeout,
                encoding="utf-8",
                errors="replace",
            )

            output = ""
            if res.stdout:
                output += res.stdout
            if res.stderr:
                if output:
                    output += "\n--- STDERR ---\n"
                output += res.stderr

            if not output.strip():
                output = f"(Command exited with code {res.returncode} with no output)"

            return {
                "success": res.returncode == 0,
                "exit_code": res.returncode,
                "output": output[:30000],  # Cap output to avoid context overflow
            }
        except subprocess.TimeoutExpired:
            return {
                "success": False,
                "exit_code": -1,
                "output": f"Command timed out after {timeout} seconds.",
            }
        except Exception as e:
            return {
                "success": False,
                "exit_code": -1,
                "output": f"Execution error: {str(e)}",
            }
