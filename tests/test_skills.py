"""Tests for skill loading and management."""

from pathlib import Path
from openlight.agent.skills import SkillsManager, SkillTool


def test_create_and_discover_skill(tmp_path: Path):
    mgr = SkillsManager(search_dirs=[tmp_path])

    # Create skill
    skill_file = tmp_path / "web_browser" / "SKILL.md"
    skill_file.parent.mkdir(parents=True)
    skill_file.write_text(
        """---
name: Web Automation
description: Automates browser workflows using Chromium
---

# Web Automation
Instructions for using browser.
""",
        encoding="utf-8",
    )

    skills = mgr.discover_skills()
    assert len(skills) == 1
    assert skills[0].name == "Web Automation"
    assert "Automates browser" in skills[0].description


def test_skill_tool(tmp_path: Path):
    mgr = SkillsManager(search_dirs=[tmp_path])
    tool = SkillTool(manager=mgr)

    # Empty list
    res = tool.execute(action="list")
    assert res["success"] is True

    # Create via tool
    c_res = tool.execute(
        action="create",
        name="Excel Helper",
        description="Tips for operating spreadsheets",
        instructions="Always select cell before editing.",
    )
    assert c_res["success"] is True

    # View
    v_res = tool.execute(action="view", name="Excel Helper")
    assert v_res["success"] is True
    assert "select cell" in v_res["output"]
