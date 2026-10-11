"""Base abstractions for LLM providers."""

from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional


@dataclass
class ToolCall:
    id: str
    name: str
    arguments: Dict[str, Any]


@dataclass
class LLMMessage:
    role: str  # 'system', 'user', 'assistant', 'tool'
    content: str
    tool_calls: Optional[List[ToolCall]] = None
    tool_call_id: Optional[str] = None
    image_base64: Optional[str] = None  # Attached image for vision models


@dataclass
class LLMResponse:
    content: str
    tool_calls: List[ToolCall] = field(default_factory=list)
    finish_reason: Optional[str] = None
    usage: Optional[Dict[str, int]] = None


class BaseLLMProvider(ABC):
    """Abstract interface for all LLM providers in OpenLight."""

    def __init__(self, model_name: str, api_key: Optional[str] = None, **kwargs):
        self.model_name = model_name
        self.api_key = api_key
        self.extra_config = kwargs

    @abstractmethod
    def generate_turn(
        self,
        messages: List[LLMMessage],
        tools_schema: Optional[List[Dict[str, Any]]] = None,
        system_instruction: Optional[str] = None,
    ) -> LLMResponse:
        """Execute a single model turn given messages and available tool schemas."""
        pass
