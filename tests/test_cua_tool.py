"""Tests for computer_use tool schema and execution."""

from openlight.computer_use.cua_tool import ComputerUseTool


def test_cua_tool_schema():
    tool = ComputerUseTool()
    schema = tool.get_tool_schema()

    assert schema["name"] == "computer_use"
    assert "parameters" in schema
    assert "action" in schema["parameters"]["properties"]
    actions = schema["parameters"]["properties"]["action"]["enum"]
    assert "capture" in actions
    assert "click" in actions
    assert "type" in actions
    assert "hotkey" in actions


def test_cua_tool_capture():
    tool = ComputerUseTool()
    res = tool.execute(action="capture")

    assert res["success"] is True
    assert "Captured screen" in res["output"]
    assert res["image_base64"] is not None
    assert len(res["image_base64"]) > 100


def test_cua_tool_wait():
    tool = ComputerUseTool()
    res = tool.execute(action="wait", seconds=0.01)
    assert res["success"] is True
    assert "Waited 0.01s" in res["output"]
