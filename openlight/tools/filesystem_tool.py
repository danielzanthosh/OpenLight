"""Filesystem inspection and editing tools."""

import os
from pathlib import Path
from typing import Any, Dict, List, Optional


class FileReadTool:
    name = "read_file"
    description = "Read lines from a file on disk with optional start and end line ranges (1-indexed)."

    def get_tool_schema(self) -> Dict[str, Any]:
        return {
            "name": self.name,
            "description": self.description,
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Absolute or relative path to the file."},
                    "start_line": {"type": "integer", "description": "1-indexed starting line number."},
                    "end_line": {"type": "integer", "description": "1-indexed ending line number."},
                },
                "required": ["path"],
            },
        }

    def execute(self, **kwargs) -> Dict[str, Any]:
        path_str = kwargs.get("path", "")
        start_line = kwargs.get("start_line")
        end_line = kwargs.get("end_line")

        try:
            path = Path(path_str).resolve()
            if not path.is_file():
                return {"success": False, "output": f"File does not exist: {path_str}"}

            with open(path, "r", encoding="utf-8", errors="replace") as f:
                lines = f.readlines()

            total_lines = len(lines)
            s = max(1, start_line) if start_line else 1
            e = min(total_lines, end_line) if end_line else total_lines

            subset = lines[s - 1 : e]
            formatted = "".join(f"{idx + s}: {line}" for idx, line in enumerate(subset))
            return {
                "success": True,
                "output": f"File: {path} (Lines {s}-{e} of {total_lines})\n{formatted}",
            }
        except Exception as ex:
            return {"success": False, "output": f"Error reading file {path_str}: {str(ex)}"}


class FileWriteTool:
    name = "write_file"
    description = "Create or overwrite a file on disk."

    def get_tool_schema(self) -> Dict[str, Any]:
        return {
            "name": self.name,
            "description": self.description,
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "File path to create or write."},
                    "content": {"type": "string", "description": "Content to write into the file."},
                    "overwrite": {"type": "boolean", "description": "Whether to overwrite if file already exists."},
                },
                "required": ["path", "content"],
            },
        }

    def execute(self, **kwargs) -> Dict[str, Any]:
        path_str = kwargs.get("path", "")
        content = kwargs.get("content", "")
        overwrite = kwargs.get("overwrite", True)

        try:
            path = Path(path_str).resolve()
            if path.exists() and not overwrite:
                return {"success": False, "output": f"File already exists and overwrite=False: {path_str}"}

            path.parent.mkdir(parents=True, exist_ok=True)
            with open(path, "w", encoding="utf-8") as f:
                f.write(content)

            return {"success": True, "output": f"Successfully wrote {len(content)} characters to {path}"}
        except Exception as ex:
            return {"success": False, "output": f"Error writing file {path_str}: {str(ex)}"}


class FilePatchTool:
    name = "patch_file"
    description = "Replace an exact chunk of text in an existing file with new content."

    def get_tool_schema(self) -> Dict[str, Any]:
        return {
            "name": self.name,
            "description": self.description,
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Path to file to patch."},
                    "target_content": {"type": "string", "description": "Exact text chunk in the file to replace."},
                    "replacement_content": {"type": "string", "description": "New text chunk to put in its place."},
                },
                "required": ["path", "target_content", "replacement_content"],
            },
        }

    def execute(self, **kwargs) -> Dict[str, Any]:
        path_str = kwargs.get("path", "")
        target = kwargs.get("target_content", "")
        replacement = kwargs.get("replacement_content", "")

        try:
            path = Path(path_str).resolve()
            if not path.is_file():
                return {"success": False, "output": f"File not found: {path_str}"}

            with open(path, "r", encoding="utf-8") as f:
                content = f.read()

            if target not in content:
                return {
                    "success": False,
                    "output": f"Target content not found in {path_str}. Verify exact whitespace and lines.",
                }

            count = content.count(target)
            if count > 1:
                return {
                    "success": False,
                    "output": f"Target content appears {count} times in {path_str}. Provide more surrounding context to be unique.",
                }

            new_content = content.replace(target, replacement, 1)
            with open(path, "w", encoding="utf-8") as f:
                f.write(new_content)

            return {"success": True, "output": f"Successfully patched {path}"}
        except Exception as ex:
            return {"success": False, "output": f"Error patching file {path_str}: {str(ex)}"}


class SearchFilesTool:
    name = "search_files"
    description = "Search for a text pattern inside files or search filenames."

    def get_tool_schema(self) -> Dict[str, Any]:
        return {
            "name": self.name,
            "description": self.description,
            "parameters": {
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "Text substring or regex pattern to search."},
                    "path": {"type": "string", "description": "Directory or file path to search within."},
                    "file_pattern": {"type": "string", "description": "Glob pattern (e.g. '*.py', '*.ts')."},
                },
                "required": ["query"],
            },
        }

    def execute(self, **kwargs) -> Dict[str, Any]:
        query = kwargs.get("query", "")
        path_str = kwargs.get("path", ".")
        file_pat = kwargs.get("file_pattern", "*")

        matches: List[str] = []
        try:
            base_path = Path(path_str).resolve()
            for p in base_path.rglob(file_pat):
                if p.is_file() and not any(part.startswith(".") for part in p.parts):
                    try:
                        with open(p, "r", encoding="utf-8", errors="ignore") as f:
                            for idx, line in enumerate(f, 1):
                                if query in line:
                                    matches.append(f"{p}:{idx}: {line.strip()[:150]}")
                                    if len(matches) >= 50:
                                        break
                    except Exception:
                        pass
                if len(matches) >= 50:
                    break

            if not matches:
                return {"success": True, "output": f"No matches found for query '{query}' in {path_str}"}

            return {"success": True, "output": "\n".join(matches)}
        except Exception as ex:
            return {"success": False, "output": f"Error searching files: {str(ex)}"}
