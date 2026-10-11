"""Anthropic Claude provider supporting tool calling and vision."""

import json
from typing import Any, Dict, List, Optional
from anthropic import Anthropic
from openlight.providers.base import BaseLLMProvider, LLMMessage, LLMResponse, ToolCall


class AnthropicProvider(BaseLLMProvider):
    """Provider for Anthropic Claude (Claude 3.5 Sonnet, 3.7 Sonnet)."""

    def __init__(
        self,
        model_name: str = "claude-3-7-sonnet-20250219",
        api_key: Optional[str] = None,
        **kwargs,
    ):
        super().__init__(model_name=model_name, api_key=api_key, **kwargs)
        self.client = Anthropic(api_key=api_key)

    def generate_turn(
        self,
        messages: List[LLMMessage],
        tools_schema: Optional[List[Dict[str, Any]]] = None,
        system_instruction: Optional[str] = None,
    ) -> LLMResponse:
        formatted_messages = []

        for msg in messages:
            if msg.role == "tool":
                formatted_messages.append({
                    "role": "user",
                    "content": [
                        {
                            "type": "tool_result",
                            "tool_use_id": msg.tool_call_id,
                            "content": msg.content,
                        }
                    ],
                })
            elif msg.role == "assistant":
                content_blocks = []
                if msg.content:
                    content_blocks.append({"type": "text", "text": msg.content})
                if msg.tool_calls:
                    for tc in msg.tool_calls:
                        content_blocks.append({
                            "type": "tool_use",
                            "id": tc.id,
                            "name": tc.name,
                            "input": tc.arguments,
                        })
                formatted_messages.append({"role": "assistant", "content": content_blocks})
            elif msg.role == "user":
                content_blocks = []
                if msg.content:
                    content_blocks.append({"type": "text", "text": msg.content})
                if msg.image_base64:
                    content_blocks.append({
                        "type": "image",
                        "source": {
                            "type": "base64",
                            "media_type": "image/jpeg",
                            "data": msg.image_base64,
                        },
                    })
                formatted_messages.append({"role": "user", "content": content_blocks})

        # Format tools for Anthropic (name, description, input_schema)
        anthropic_tools = []
        if tools_schema:
            for s in tools_schema:
                anthropic_tools.append({
                    "name": s["name"],
                    "description": s.get("description", ""),
                    "input_schema": s.get("parameters", {"type": "object", "properties": {}}),
                })

        kwargs: Dict[str, Any] = {
            "model": self.model_name,
            "messages": formatted_messages,
            "max_tokens": 4096,
        }
        if system_instruction:
            kwargs["system"] = system_instruction
        if anthropic_tools:
            kwargs["tools"] = anthropic_tools

        response = self.client.messages.create(**kwargs)

        text_content = ""
        tool_calls: List[ToolCall] = []

        for block in response.content:
            if block.type == "text":
                text_content += block.text
            elif block.type == "tool_use":
                tool_calls.append(
                    ToolCall(
                        id=block.id,
                        name=block.name,
                        arguments=block.input if isinstance(block.input, dict) else {},
                    )
                )

        return LLMResponse(
            content=text_content,
            tool_calls=tool_calls,
            finish_reason=response.stop_reason,
            usage={"input_tokens": response.usage.input_tokens, "output_tokens": response.usage.output_tokens},
        )
