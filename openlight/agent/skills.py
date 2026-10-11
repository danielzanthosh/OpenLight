"""Skill management and learning loop for OpenLight (Hermes-style)."""

import os
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, List, Optional
from openlight.config import DEFAULT_SKILLS_DIR


@dataclass
class Skill:
    name: str
    description: str
    content: str
    path: Path


class SkillsManager:
    """Discovers, loads, and saves agent skills formatted as markdown files with frontmatter."""

    def __init__(self, search_dirs: Optional[List[Path]] = None):
        self.search_dirs = search_dirs or [
            Path("./skills"),
            DEFAULT_SKILLS_DIR,
        ]
        self.skills_dir = self.search_dirs[0] if self.search_dirs else DEFAULT_SKILLS_DIR

    def discover_skills(self) -> List[Skill]:
        """Scan directories for SKILL.md or *.skill.md files."""
        skills: List[Skill] = []
        seen = set()

        for directory in self.search_dirs:
            if not directory.exists():
                continue
            for path in directory.rglob("*.md"):
                if path.name.lower() in ("skill.md", "skills.md") or path.name.endswith(".skill.md"):
                    skill = self._parse_skill_file(path)
                    if skill and skill.name not in seen:
                        skills.append(skill)
                        seen.add(skill.name)

        return skills

    def _parse_skill_file(self, path: Path) -> Optional[Skill]:
        try:
            with open(path, "r", encoding="utf-8") as f:
                raw = f.read()

            name = path.stem
            desc = ""
            body = raw

            if raw.startswith("---"):
                parts = raw.split("---", 2)
                if len(parts) >= 3:
                    frontmatter = parts[1]
                    body = parts[2].strip()
                    for line in frontmatter.splitlines():
                        if line.startswith("name:"):
                            name = line.split("name:", 1)[1].strip().strip('"\'')
                        elif line.startswith("description:"):
                            desc = line.split("description:", 1)[1].strip().strip('"\'')

            return Skill(name=name, description=desc, content=body, path=path)
        except Exception:
            return None

    def create_skill(self, name: str, description: str, instructions: str) -> str:
        """Create a new learned skill in user's skills directory."""
        self.skills_dir.mkdir(parents=True, exist_ok=True)
        slug = "".join(c if c.isalnum() or c in "-_" else "_" for c in name.lower())
        skill_dir = self.skills_dir / slug
        skill_dir.mkdir(parents=True, exist_ok=True)
        skill_file = skill_dir / "SKILL.md"

        content = f"""---
name: {name}
description: "{description}"
---

# {name}

{instructions}
"""
        with open(skill_file, "w", encoding="utf-8") as f:
            f.write(content)

        return f"Successfully created skill '{name}' at {skill_file}"


class SkillTool:
    """Tool allowing the agent to discover skills, view instructions, or create new skills."""

    name = "skill_manage"
    description = "List available agent skills, view detailed skill instructions, or create new learned skills."

    def __init__(self, manager: Optional[SkillsManager] = None):
        self.manager = manager or SkillsManager()

    def get_tool_schema(self) -> Dict[str, Any]:
        return {
            "name": self.name,
            "description": self.description,
            "parameters": {
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["list", "view", "create"],
                        "description": "Skill operation.",
                    },
                    "name": {"type": "string", "description": "Skill name for view or create."},
                    "description": {"type": "string", "description": "Short description for create."},
                    "instructions": {"type": "string", "description": "Markdown instructions for create."},
                },
                "required": ["action"],
            },
        }

    def execute(self, **kwargs) -> Dict[str, Any]:
        action = kwargs.get("action", "list")
        if action == "list":
            skills = self.manager.discover_skills()
            if not skills:
                return {"success": True, "output": "No specialized skills currently loaded."}
            lines = [f"- {s.name}: {s.description}" for s in skills]
            return {"success": True, "output": "Available Skills:\n" + "\n".join(lines)}

        elif action == "view":
            name = kwargs.get("name", "")
            skills = self.manager.discover_skills()
            for s in skills:
                if s.name.lower() == name.lower():
                    return {"success": True, "output": f"=== Skill: {s.name} ===\n{s.content}"}
            return {"success": False, "output": f"Skill '{name}' not found."}

        elif action == "create":
            name = kwargs.get("name", "")
            desc = kwargs.get("description", "")
            inst = kwargs.get("instructions", "")
            if not name or not inst:
                return {"success": False, "output": "Both 'name' and 'instructions' required."}
            res = self.manager.create_skill(name, desc, inst)
            return {"success": True, "output": res}

        return {"success": False, "output": f"Unknown skill action: {action}"}
