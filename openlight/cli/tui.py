"""Rich Terminal UI formatting and callbacks for OpenLight."""

import json
from typing import Any, Dict
from rich.console import Console
from rich.markdown import Markdown
from rich.panel import Panel
from rich.table import Table
from rich.text import Text

console = Console()


def print_banner(version: str, provider: str, model: str):
    """Print the OpenLight welcome banner."""
    title = Text("🌟 OpenLight", style="bold cyan")
    title.append(f" v{version}", style="dim")
    subtitle = f"Hermes-Style Autonomous Agent & Computer-Using Agent (CUA)\nProvider: [bold yellow]{provider}[/bold yellow] | Model: [bold green]{model}[/bold green]"

    panel = Panel(
        subtitle,
        title=title,
        border_style="cyan",
        padding=(0, 2),
    )
    console.print(panel)


def on_tool_start(tool_name: str, args: Dict[str, Any]):
    """Display tool invocation indicator."""
    short_args = json.dumps(args)
    if len(short_args) > 120:
        short_args = short_args[:117] + "..."
    console.print(f"[bold yellow]⚙ Executing Tool:[/bold yellow] [bold white]{tool_name}[/bold white] [dim]{short_args}[/dim]")


def on_tool_finish(tool_name: str, result: Dict[str, Any]):
    """Display tool result summary."""
    success = result.get("success", True)
    status_style = "bold green" if success else "bold red"
    status_icon = "✓" if success else "✗"

    output = str(result.get("output", "")).strip()
    preview = output if len(output) <= 200 else output[:197] + "..."
    console.print(f"[{status_style}]{status_icon} Tool {tool_name}:[/{status_style}] [dim]{preview}[/dim]")

    if result.get("image_base64"):
        console.print("[dim cyan]📸 Captured screenshot attached to context.[/dim cyan]")


def print_assistant_message(content: str):
    """Print markdown formatted assistant response."""
    if not content:
        return
    console.print("\n[bold cyan]OpenLight:[/bold cyan]")
    console.print(Markdown(content))
    console.print()
