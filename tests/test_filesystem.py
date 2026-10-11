"""Tests for filesystem tools."""

from pathlib import Path
from openlight.tools.filesystem_tool import (
    FilePatchTool,
    FileReadTool,
    FileWriteTool,
    SearchFilesTool,
)


def test_write_and_read_file(tmp_path: Path):
    test_file = tmp_path / "hello.txt"
    write_tool = FileWriteTool()
    read_tool = FileReadTool()

    # Write file
    w_res = write_tool.execute(path=str(test_file), content="Line 1\nLine 2\nLine 3\n")
    assert w_res["success"] is True

    # Read all lines
    r_res = read_tool.execute(path=str(test_file))
    assert r_res["success"] is True
    assert "Line 1" in r_res["output"]
    assert "Line 3" in r_res["output"]

    # Read range
    r_subset = read_tool.execute(path=str(test_file), start_line=2, end_line=2)
    assert r_subset["success"] is True
    assert "2: Line 2" in r_subset["output"]
    assert "Line 1" not in r_subset["output"]


def test_patch_file(tmp_path: Path):
    test_file = tmp_path / "code.py"
    write_tool = FileWriteTool()
    patch_tool = FilePatchTool()

    write_tool.execute(path=str(test_file), content="def foo():\n    return 42\n")

    p_res = patch_tool.execute(
        path=str(test_file),
        target_content="return 42",
        replacement_content="return 100",
    )
    assert p_res["success"] is True

    # Verify content
    read_tool = FileReadTool()
    r_res = read_tool.execute(path=str(test_file))
    assert "return 100" in r_res["output"]


def test_search_files(tmp_path: Path):
    sub = tmp_path / "src"
    sub.mkdir()
    (sub / "a.py").write_text("def find_me(): pass", encoding="utf-8")
    (sub / "b.txt").write_text("nothing here", encoding="utf-8")

    search_tool = SearchFilesTool()
    res = search_tool.execute(query="find_me", path=str(tmp_path))
    assert res["success"] is True
    assert "find_me" in res["output"]
    assert "a.py" in res["output"]
