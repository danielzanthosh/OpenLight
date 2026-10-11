"""OpenAI and OpenAI-compatible provider."""

import json
from typing import Any, Dict, List, Optional
from openai import OpenAI
from openlight.providers.base import BaseLLMProvider, LLMMessage, LLMResponse, ToolCall


class OpenAIProvider(BaseLLMProvider):
    """Provider for OpenAI (GPT-4o, o3, etc.) and OpenAI-compatible endpoints."""

    def __init__(
        self,
        model_name: str = "gpt-4o",
        api_key: Optional[str] = None,
        base_url: Optional[str] = None,
        **kwargs,
    ):
        super().__init__(model_name=model_name, api_key=api_key, **kwargs)
        self.client = OpenAI(api_key=api_key, base_url=base_url)

    def generate_turn(
        self,
        messages: List[LLMMessage],
        tools_schema: Optional[List[Dict[str, Any]]] = None,
        system_instruction: Optional[str] = None,
    ) -> LLMResponse:
        formatted_messages = []

        if system_instruction:
            formatted_messages.append({"role": "system", "content": system_instruction})

        for msg in messages:
            if msg.role == "tool":
                formatted_messages.append({
                    "role": "tool",
                    "tool_call_id": msg.tool_call_id,
                    "content": msg.content,
                })
            elif msg.role == "assistant":
                m: Dict[str, Any] = {"role": "assistant", "content": msg.content or ""}
                if msg.tool_calls:
                    m["tool_calls"] = [
                        {
                            "id": tc.id,
                            "type": "function",
                            "function": {
                                "name": tc.name,
                                "arguments": json.dumps(tc.arguments),
                            },
                        }
                        for tc in msg.tool_calls
                    ]
                formatted_messages.append(m)
            elif msg.role == "user":
                if msg.image_base64:
                    # Vision content with image
                    content_parts = []
                    if msg.content:
                        content_parts.append({"type": "text", "text": msg.content})
                    content_parts.append({
                        "type": "image_url",
                        "image_url": {
                            "url": f"data:image/jpeg;base64,{msg.image_base64}",
                            "detail": "high",
                        },
                    })
                    formatted_messages.append({"role": "user", "content": content_parts})
                else:
                    formatted_messages.append({"role": "user", "content": msg.content})

        tools_param = None
        if tools_schema:
            tools_param = [
                {
                    "type": "function",
                    "function": schema,
                }
                for schema in tools_schema
            ]

        kwargs: Dict[str, Any] = {
            "model": self.model_name,
            "messages": formatted_messages,
        }
        if tools_param:
            kwargs["tools"] = tools_param
            kwargs["tool_choice"] = "auto"

        response = self.client.chat.completions.create(**kwargs)
        choice = response.choices[0]
        msg = choice.message

        tool_calls: List[ToolCall] = []
        if msg.tool_calls:
            for tc in msg.tool_calls:
                try:
                    args = json.loads(tc.function.arguments) if tc.function.arguments else {}
                except Exception:
                    args = {}
                tool_calls.append(
                    ToolCall(
                        id=tc.id,
                        name=tc.function.name,
                        arguments=args,
                    )
                )

        return LLMResponse(
            content=msg.content or "",
            tool_calls=tool_calls,
            finish_reason=choice.finish_reason,
            usage=response.usage.model_dump() if response.usage else None,
        )
