"""Tests for terminal execution tool."""

from openlight.tools.terminal_tool import TerminalTool


def test_terminal_simple_command():
    tool = TerminalTool()
    res = tool.execute(command="Write-Output 'HelloOpenLight'")
    assert res["success"] is True
    assert "HelloOpenLight" in res["output"]


def test_terminal_exit_code():
    tool = TerminalTool()
    res = tool.execute(command="exit 0")
    assert res["success"] is True
    assert res["exit_code"] == 0


def test_terminal_timeout():
    tool = TerminalTool()
    # Test timeout protection
    res = tool.execute(command="Start-Sleep -Seconds 3", timeout_seconds=1)
    assert res["success"] is False
    assert "timed out" in res["output"].lower()
