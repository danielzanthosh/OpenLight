"""Tests for memory store and tool."""

from pathlib import Path
from openlight.agent.memory import MemoryManager, MemoryTool


def test_memory_crud(tmp_path: Path):
    mem_file = tmp_path / "test_memory.json"
    mgr = MemoryManager(memory_path=mem_file)

    mgr.store("favorite_editor", "VS Code")
    mgr.store("user_timezone", "UTC+5:30")

    # Recall
    results = mgr.recall("editor")
    assert len(results) == 1
    assert results[0]["key"] == "favorite_editor"
    assert results[0]["value"] == "VS Code"

    # List
    all_items = mgr.list_all()
    assert len(all_items) == 2

    # Delete
    mgr.delete("favorite_editor")
    assert "favorite_editor" not in mgr.list_all()


def test_memory_tool(tmp_path: Path):
    mem_file = tmp_path / "test_memory.json"
    mgr = MemoryManager(memory_path=mem_file)
    tool = MemoryTool(memory_manager=mgr)

    # Store via tool
    res = tool.execute(action="store", key="os", value="Windows 11")
    assert res["success"] is True

    # Recall via tool
    rec = tool.execute(action="recall", query="Windows")
    assert rec["success"] is True
    assert "Windows 11" in rec["output"]
