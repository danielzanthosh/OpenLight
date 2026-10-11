"""Configuration management for OpenLight."""

import json
import os
from pathlib import Path
from typing import Optional
from pydantic import BaseModel, Field

DEFAULT_DATA_DIR = Path.home() / ".openlight"
DEFAULT_CONFIG_PATH = DEFAULT_DATA_DIR / "config.json"
DEFAULT_SKILLS_DIR = DEFAULT_DATA_DIR / "skills"
DEFAULT_MEMORY_PATH = DEFAULT_DATA_DIR / "memory.json"


class OpenLightConfig(BaseModel):
    """Configuration settings for OpenLight agent and providers."""

    # Provider and Model selection
    default_provider: str = Field(
        default="gemini",
        description="Default provider: 'gemini', 'openai', 'anthropic', 'openrouter', or 'ollama'",
    )
    openai_model: str = Field(default="gpt-4o")
    anthropic_model: str = Field(default="claude-3-7-sonnet-20250219")
    gemini_model: str = Field(default="gemini-2.5-pro")
    openrouter_model: str = Field(default="anthropic/claude-3.7-sonnet")
    ollama_model: str = Field(default="llama3.2-vision")
    ollama_base_url: str = Field(default="http://localhost:11434/v1")

    # API Keys (loaded from env or config file)
    openai_api_key: Optional[str] = None
    anthropic_api_key: Optional[str] = None
    gemini_api_key: Optional[str] = None
    openrouter_api_key: Optional[str] = None

    # Computer Use / Screen Safety Settings
    failsafe_enabled: bool = Field(
        default=True,
        description="Whether moving mouse to screen corners triggers immediate abort",
    )
    mouse_move_duration: float = Field(default=0.15, description="Duration in seconds for mouse movements")
    typing_interval: float = Field(default=0.015, description="Interval in seconds between keystrokes")
    confirm_destructive_actions: bool = Field(default=False)
    screenshot_max_dimension: int = Field(default=1280, description="Max width/height for screenshots sent to LLM")
    default_monitor_index: int = Field(default=1, description="Monitor index to capture (1 = primary)")

    # Agent loop limits
    max_turns: int = Field(default=30, description="Maximum agent execution turns per task")
    working_directory: str = Field(default_factory=os.getcwd)

    @classmethod
    def load(cls) -> "OpenLightConfig":
        """Load configuration from environment variables and config file."""
        data = {}
        if DEFAULT_CONFIG_PATH.exists():
            try:
                with open(DEFAULT_CONFIG_PATH, "r", encoding="utf-8") as f:
                    data = json.load(f)
            except Exception:
                pass

        # Environment variable overrides
        openai_key = os.getenv("OPENAI_API_KEY", data.get("openai_api_key"))
        anthropic_key = os.getenv("ANTHROPIC_API_KEY", data.get("anthropic_api_key"))
        gemini_key = os.getenv("GEMINI_API_KEY", data.get("gemini_api_key"))
        openrouter_key = os.getenv("OPENROUTER_API_KEY", data.get("openrouter_api_key"))

        config = cls(
            **data,
            openai_api_key=openai_key,
            anthropic_api_key=anthropic_key,
            gemini_api_key=gemini_key,
            openrouter_api_key=openrouter_key,
        )
        return config

    def save(self):
        """Save configuration to ~/.openlight/config.json."""
        DEFAULT_DATA_DIR.mkdir(parents=True, exist_ok=True)
        with open(DEFAULT_CONFIG_PATH, "w", encoding="utf-8") as f:
            json.dump(self.model_dump(), f, indent=2)
