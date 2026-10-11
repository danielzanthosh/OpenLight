"""OpenRouter and Ollama provider implementations."""

from typing import Optional
from openlight.providers.openai_provider import OpenAIProvider


class OpenRouterProvider(OpenAIProvider):
    """OpenRouter provider using OpenAI-compatible client."""

    def __init__(
        self,
        model_name: str = "anthropic/claude-3.7-sonnet",
        api_key: Optional[str] = None,
        base_url: str = "https://openrouter.ai/api/v1",
        **kwargs,
    ):
        super().__init__(
            model_name=model_name,
            api_key=api_key,
            base_url=base_url,
            **kwargs,
        )


class OllamaProvider(OpenAIProvider):
    """Local Ollama provider using OpenAI-compatible client."""

    def __init__(
        self,
        model_name: str = "llama3.2-vision",
        base_url: str = "http://localhost:11434/v1",
        api_key: str = "ollama",
        **kwargs,
    ):
        super().__init__(
            model_name=model_name,
            api_key=api_key,
            base_url=base_url,
            **kwargs,
        )
