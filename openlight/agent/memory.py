"""Persistent memory manager for OpenLight agent (Hermes-style)."""

import json
from pathlib import Path
from typing import Any, Dict, List, Optional
from openlight.config import DEFAULT_MEMORY_PATH


class MemoryManager:
    """Stores and retrieves long-term agent memories across sessions."""

    def __init__(self, memory_path: Optional[Path] = None):
        self.memory_path = memory_path or DEFAULT_MEMORY_PATH
        self._cache: Dict[str, str] = {}
        self._load()

    def _load(self):
        if self.memory_path.exists():
            try:
                with open(self.memory_path, "r", encoding="utf-8") as f:
                    self._cache = json.load(f)
            except Exception:
                self._cache = {}

    def _save(self):
        self.memory_path.parent.mkdir(parents=True, exist_ok=True)
        with open(self.memory_path, "w", encoding="utf-8") as f:
            json.dump(self._cache, f, indent=2, ensure_ascii=False)

    def store(self, key: str, value: str) -> str:
        """Store or update a memory item."""
        self._cache[key] = value
        self._save()
        return f"Stored memory item '{key}'"

    def recall(self, query: str) -> List[Dict[str, str]]:
        """Search memory for relevant keys or values."""
        query_lower = query.lower()
        results = []
        for k, v in self._cache.items():
            if query_lower in k.lower() or query_lower in v.lower():
                results.append({"key": k, "value": v})
        return results

    def list_all(self) -> Dict[str, str]:
        """Return all memories."""
        return dict(self._cache)

    def delete(self, key: str) -> str:
        """Delete a memory item."""
        if key in self._cache:
            del self._cache[key]
            self._save()
            return f"Deleted memory item '{key}'"
        return f"Key '{key}' not found in memory"

    def format_for_prompt(self) -> str:
        """Format persistent memories for injection into system prompt."""
        if not self._cache:
            return ""
        lines = ["--- Persistent Memories ---"]
        for k, v in self._cache.items():
            lines.append(f"- {k}: {v}")
        return "\n".join(lines)


class MemoryTool:
    """Tool allowing the agent to remember and recall user preferences and knowledge."""

    name = "memory"
    description = "Store, recall, list, or delete long-term persistent memories across conversations."

    def __init__(self, memory_manager: Optional[MemoryManager] = None):
        self.manager = memory_manager or MemoryManager()

    def get_tool_schema(self) -> Dict[str, Any]:
        return {
            "name": self.name,
            "description": self.description,
            "parameters": {
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["store", "recall", "list", "delete"],
                        "description": "Memory operation to perform.",
                    },
                    "key": {"type": "string", "description": "Key identifier for the memory item."},
                    "value": {"type": "string", "description": "Information to store in memory."},
                    "query": {"type": "string", "description": "Search query for recall action."},
                },
                "required": ["action"],
            },
        }

    def execute(self, **kwargs) -> Dict[str, Any]:
        action = kwargs.get("action", "list")
        if action == "store":
            key = kwargs.get("key", "")
            val = kwargs.get("value", "")
            if not key or not val:
                return {"success": False, "output": "Both 'key' and 'value' are required to store."}
            res = self.manager.store(key, val)
            return {"success": True, "output": res}

        elif action == "recall":
            q = kwargs.get("query", "")
            items = self.manager.recall(q)
            if not items:
                return {"success": True, "output": f"No memories found matching '{q}'."}
            formatted = "\n".join(f"- {item['key']}: {item['value']}" for item in items)
            return {"success": True, "output": f"Found memories:\n{formatted}"}

        elif action == "delete":
            key = kwargs.get("key", "")
            res = self.manager.delete(key)
            return {"success": True, "output": res}

        elif action == "list":
            all_mems = self.manager.list_all()
            if not all_mems:
                return {"success": True, "output": "Memory store is currently empty."}
            formatted = "\n".join(f"- {k}: {v}" for k, v in all_mems.items())
            return {"success": True, "output": f"All memories:\n{formatted}"}

        return {"success": False, "output": f"Unknown memory action: {action}"}
