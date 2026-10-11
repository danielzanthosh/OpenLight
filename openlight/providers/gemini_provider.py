"""Google Gemini provider using official google-genai SDK."""

import base64
from typing import Any, Dict, List, Optional
from openlight.providers.base import BaseLLMProvider, LLMMessage, LLMResponse, ToolCall


class GeminiProvider(BaseLLMProvider):
    """Provider for Google Gemini models (Gemini 2.5 Pro/Flash, Gemini 3)."""

    def __init__(
        self,
        model_name: str = "gemini-2.5-pro",
        api_key: Optional[str] = None,
        **kwargs,
    ):
        super().__init__(model_name=model_name, api_key=api_key, **kwargs)
        from google import genai
        self.client = genai.Client(api_key=api_key)

    def generate_turn(
        self,
        messages: List[LLMMessage],
        tools_schema: Optional[List[Dict[str, Any]]] = None,
        system_instruction: Optional[str] = None,
    ) -> LLMResponse:
        from google.genai import types

        contents = []

        for msg in messages:
            role = "model" if msg.role == "assistant" else "user"
            parts = []

            if msg.role == "tool":
                # Function response
                parts.append(
                    types.Part.from_function_response(
                        name=msg.tool_call_id or "function",
                        response={"result": msg.content},
                    )
                )
                contents.append(types.Content(role="user", parts=parts))
                continue

            if msg.content:
                parts.append(types.Part.from_text(text=msg.content))

            if msg.image_base64:
                img_bytes = base64.b64decode(msg.image_base64)
                parts.append(
                    types.Part.from_bytes(data=img_bytes, mime_type="image/jpeg")
                )

            if msg.tool_calls:
                for tc in msg.tool_calls:
                    parts.append(
                        types.Part.from_function_call(
                            name=tc.name,
                            args=tc.arguments,
                        )
                    )

            if parts:
                contents.append(types.Content(role=role, parts=parts))

        # Build tools config
        gemini_tools = None
        if tools_schema:
            function_declarations = []
            for s in tools_schema:
                function_declarations.append(
                    types.FunctionDeclaration(
                        name=s["name"],
                        description=s.get("description", ""),
                        parameters=s.get("parameters"),
                    )
                )
            gemini_tools = [types.Tool(function_declarations=function_declarations)]

        config = types.GenerateContentConfig(
            system_instruction=system_instruction,
            tools=gemini_tools,
        )

        response = self.client.models.generate_content(
            model=self.model_name,
            contents=contents,
            config=config,
        )

        text_content = ""
        tool_calls: List[ToolCall] = []

        if response.candidates:
            cand = response.candidates[0]
            if cand.content and cand.content.parts:
                for part in cand.content.parts:
                    if part.text:
                        text_content += part.text
                    if part.function_call:
                        fc = part.function_call
                        tool_calls.append(
                            ToolCall(
                                id=fc.name,
                                name=fc.name,
                                arguments=dict(fc.args) if fc.args else {},
                            )
                        )

        return LLMResponse(
            content=text_content,
            tool_calls=tool_calls,
            finish_reason="stop",
        )
