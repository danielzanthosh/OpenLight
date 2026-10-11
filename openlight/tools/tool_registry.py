"""Tool registry managing tool definitions and execution."""

from typing import Any, Callable, Dict, List, Optional
from openlight.computer_use.cua_tool import ComputerUseTool
from openlight.tools.filesystem_tool import (
    FilePatchTool,
    FileReadTool,
    FileWriteTool,
    SearchFilesTool,
)
from openlight.tools.terminal_tool import TerminalTool


class ToolRegistry:
    """Central registry for discovering and executing agent tools."""

    def __init__(self):
        self._tools: Dict[str, Any] = {}

    def register(self, tool: Any):
        """Register a tool instance."""
        name = getattr(tool, "name", None)
        if not name:
            raise ValueError("Tool must have a 'name' attribute.")
        self._tools[name] = tool

    def get_tool(self, name: str) -> Optional[Any]:
        return self._tools.get(name)

    def list_schemas(self) -> List[Dict[str, Any]]:
        """Return function calling schemas for all registered tools."""
        schemas = []
        for tool in self._tools.values():
            if hasattr(tool, "get_tool_schema"):
                schemas.append(tool.get_tool_schema())
        return schemas

    def execute_tool(self, name: str, arguments: Dict[str, Any]) -> Dict[str, Any]:
        """Execute a tool by name with arguments."""
        tool = self.get_tool(name)
        if not tool:
            return {"success": False, "output": f"Tool '{name}' not found in registry."}

        try:
            if hasattr(tool, "execute"):
                return tool.execute(**arguments)
            else:
                return {"success": False, "output": f"Tool '{name}' has no execute method."}
        except Exception as ex:
            return {"success": False, "output": f"Error executing tool '{name}': {str(ex)}"}

    @classmethod
    def create_default(
        cls,
        include_computer_use: bool = True,
        monitor_index: int = 1,
        cwd: Optional[str] = None,
    ) -> "ToolRegistry":
        """Create a registry loaded with default developer and computer-use tools."""
        registry = cls()
        registry.register(TerminalTool(default_cwd=cwd))
        registry.register(FileReadTool())
        registry.register(FileWriteTool())
        registry.register(FilePatchTool())
        registry.register(SearchFilesTool())

        if include_computer_use:
            registry.register(ComputerUseTool(monitor_index=monitor_index))

        return registry
