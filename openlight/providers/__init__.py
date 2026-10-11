"""LLM Providers factory and exports for OpenLight."""

from typing import Optional
from openlight.config import OpenLightConfig
from openlight.providers.anthropic_provider import AnthropicProvider
from openlight.providers.base import BaseLLMProvider, LLMMessage, LLMResponse, ToolCall
from openlight.providers.gemini_provider import GeminiProvider
from openlight.providers.openai_provider import OpenAIProvider
from openlight.providers.openrouter_provider import OllamaProvider, OpenRouterProvider


def get_provider(
    provider_name: Optional[str] = None,
    model_override: Optional[str] = None,
    config: Optional[OpenLightConfig] = None,
) -> BaseLLMProvider:
    """Factory function to instantiate the requested LLM provider."""
    cfg = config or OpenLightConfig.load()
    name = (provider_name or cfg.default_provider).lower()

    if name == "gemini":
        model = model_override or cfg.gemini_model
        return GeminiProvider(model_name=model, api_key=cfg.gemini_api_key)
    elif name == "anthropic":
        model = model_override or cfg.anthropic_model
        return AnthropicProvider(model_name=model, api_key=cfg.anthropic_api_key)
    elif name == "openai":
        model = model_override or cfg.openai_model
        return OpenAIProvider(model_name=model, api_key=cfg.openai_api_key)
    elif name == "openrouter":
        model = model_override or cfg.openrouter_model
        return OpenRouterProvider(model_name=model, api_key=cfg.openrouter_api_key)
    elif name == "ollama":
        model = model_override or cfg.ollama_model
        return OllamaProvider(model_name=model, base_url=cfg.ollama_base_url)
    else:
        raise ValueError(f"Unsupported provider: '{name}'. Supported: gemini, openai, anthropic, openrouter, ollama.")


__all__ = [
    "BaseLLMProvider",
    "LLMMessage",
    "LLMResponse",
    "ToolCall",
    "GeminiProvider",
    "AnthropicProvider",
    "OpenAIProvider",
    "OpenRouterProvider",
    "OllamaProvider",
    "get_provider",
]
