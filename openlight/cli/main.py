"""CLI Entrypoint for OpenLight."""

import argparse
import sys
from prompt_toolkit import PromptSession
from prompt_toolkit.history import InMemoryHistory
from openlight import __version__
from openlight.agent.agent import OpenLightAgent
from openlight.cli.tui import (
    console,
    on_tool_finish,
    on_tool_start,
    print_assistant_message,
    print_banner,
)
from openlight.config import OpenLightConfig
from openlight.providers import get_provider


def handle_config_cmd(args):
    """View or update OpenLight configuration."""
    cfg = OpenLightConfig.load()
    if args.set_key and args.value:
        setattr(cfg, args.set_key, args.value)
        cfg.save()
        console.print(f"[bold green]Successfully updated {args.set_key}[/bold green]")
    else:
        console.print("[bold cyan]Current OpenLight Configuration:[/bold cyan]")
        for k, v in cfg.model_dump().items():
            # Mask API keys
            display_val = "********" if "key" in k and v else v
            console.print(f"  [bold white]{k}:[/bold white] [dim]{display_val}[/dim]")


def main():
    parser = argparse.ArgumentParser(
        prog="openlight",
        description="OpenLight: Lightweight Autonomous Agent with Screen Control (CUA) and Hermes Loop",
    )
    parser.add_argument("-p", "--prompt", type=str, help="Single-shot task prompt to execute")
    parser.add_argument("--provider", type=str, help="LLM provider: gemini, openai, anthropic, openrouter, ollama")
    parser.add_argument("--model", type=str, help="Specific model name override")
    parser.add_argument("--monitor", type=int, default=1, help="Monitor index to control (1 = primary)")
    parser.add_argument("--version", action="version", version=f"OpenLight {__version__}")

    subparsers = parser.add_subparsers(dest="subcommand")
    config_parser = subparsers.add_parser("config", help="Manage configuration and API keys")
    config_parser.add_argument("--set-key", type=str, help="Configuration key to update (e.g. gemini_api_key)")
    config_parser.add_argument("--value", type=str, help="Value to set")

    args = parser.parse_args()

    if args.subcommand == "config":
        handle_config_cmd(args)
        return

    cfg = OpenLightConfig.load()
    if args.provider:
        cfg.default_provider = args.provider
    if args.monitor:
        cfg.default_monitor_index = args.monitor

    try:
        provider = get_provider(
            provider_name=args.provider,
            model_override=args.model,
            config=cfg,
        )
    except Exception as e:
        console.print(f"[bold red]Provider Error:[/bold red] {e}")
        console.print("[dim]Configure API keys in environment or run 'openlight config --set-key <key> --value <val>'[/dim]")
        sys.exit(1)

    agent = OpenLightAgent(
        config=cfg,
        provider=provider,
        on_tool_start=on_tool_start,
        on_tool_finish=on_tool_finish,
    )

    print_banner(
        version=__version__,
        provider=provider.__class__.__name__.replace("Provider", ""),
        model=provider.model_name,
    )

    # Single-shot mode
    if args.prompt:
        console.print(f"[bold green]Goal:[/bold green] {args.prompt}\n")
        try:
            res = agent.run_task(args.prompt)
            print_assistant_message(res)
        except KeyboardInterrupt:
            console.print("\n[bold yellow]Agent interrupted by user.[/bold yellow]")
        return

    # Interactive REPL mode
    session = PromptSession(history=InMemoryHistory())
    console.print("[dim]Type your message or task below. Type 'exit' or 'quit' to close.[/dim]\n")

    while True:
        try:
            user_input = session.prompt("openlight > ").strip()
            if not user_input:
                continue
            if user_input.lower() in ("exit", "quit", "q"):
                console.print("[dim]Goodbye![/dim]")
                break

            res = agent.run_task(user_input)
            print_assistant_message(res)

        except (KeyboardInterrupt, EOFError):
            console.print("\n[dim]Session closed.[/dim]")
            break
        except Exception as ex:
            console.print(f"[bold red]Error:[/bold red] {ex}")


if __name__ == "__main__":
    main()
