# claude-model-switch

Use **any model** with Claude Code. Switch providers instantly, add your own API keys, and run multi-model tmux orchestration — no restart needed.

A lightweight local proxy that sits between Claude Code and any OpenAI-compatible or Anthropic-compatible API. Bring your own models from GLM, MiniMax, or any provider with an API endpoint.

## Install

**One-liner (recommended):**

```bash
curl -fsSL https://raw.githubusercontent.com/codejunkie99/claude-model-switch/main/install.sh | sh
```

**Cargo:**

```bash
cargo install claude-model-switch
```

**From source:**

```bash
git clone https://github.com/codejunkie99/claude-model-switch.git
cd claude-model-switch
cargo install --path .
```

## Quick Start

```bash
# 1. Point Claude Code at the local proxy
claude-model-switch init

# 2. Add your provider credentials
claude-model-switch setup glm --api-key sk-your-key-here

# 3. Start the proxy
claude-model-switch start

# 4. Switch providers (instant, no restart)
claude-model-switch use glm

# 5. Switch back anytime
claude-model-switch use claude
```

That's it. Claude Code now routes through your chosen provider.

## Claude Code Plugin

Install as a Claude Code plugin for the smoothest experience. The plugin auto-installs the binary, starts the proxy, and gives you interactive slash commands.

### Plugin install

Copy or symlink the `plugin/` directory into your Claude Code plugins:

```bash
# Option 1: symlink
ln -s /path/to/claude-model-switch/plugin ~/.claude/plugins/claude-model-switch

# Option 2: copy
cp -r /path/to/claude-model-switch/plugin ~/.claude/plugins/claude-model-switch
```

### Plugin commands

| Command | What it does |
|---------|-------------|
| `/cms-setup` | Interactive provider credential setup |
| `/cms-use` | Switch provider (shows picker if no arg) |
| `/cms-add` | Walk through adding a custom provider |
| `/cms-list` | List all available providers |
| `/cms-status` | Show current provider, models, proxy state |
| `/cms-orchestrate` | Launch multi-model tmux session |
| `/cms-start` | Start the proxy |
| `/cms-stop` | Stop the proxy |
| `/cms-remove` | Remove a provider |

The plugin's SessionStart hook automatically:
1. Downloads and installs the binary if missing
2. Starts the proxy if not running
3. Runs first-time init if needed

## Built-in Providers

| Provider | haiku | sonnet | opus |
|----------|-------|--------|------|
| `claude` | (passthrough) | (passthrough) | (passthrough) |
| `glm` | glm-4.5-air | glm-4.7 | glm-4.7 |
| `glm-flash` | glm-4.7-flashx | glm-4.7-flashx | glm-4.7-flashx |
| `glm-5` | glm-4.7-flashx | glm-5-code | glm-5 |
| `minimax` | MiniMax-M2 | MiniMax-M2.5 | MiniMax-M2.5 |
| `minimax-fast` | MiniMax-M2 | MiniMax-M2.5-Lightning | MiniMax-M2.5 |

## Add Any Provider

Add any API endpoint that speaks the Anthropic or OpenAI messages format:

```bash
# Register the provider with model mappings
claude-model-switch add my-provider \
  --base-url https://api.example.com/v1 \
  --haiku small-model \
  --sonnet medium-model \
  --opus large-model

# Add credentials
claude-model-switch setup my-provider --api-key sk-xxx

# Switch to it
claude-model-switch use my-provider
```

Claude Code uses three model tiers internally. You map each tier to whatever model your provider offers:

- **haiku** — fast/cheap tier (used for quick tasks)
- **sonnet** — balanced tier (used for most coding)
- **opus** — best tier (used for complex reasoning)

## Multi-Model Orchestration

Run multiple Claude Code instances in tmux, each using a different provider. Useful for parallel workstreams where different models have different strengths.

### Start a session

```bash
# Trio: planner + coder + reviewer
claude-model-switch orchestrate start --preset trio

# Duo: planner + coder
claude-model-switch orchestrate start --preset duo

# Attach to see all panes
tmux attach -t cms-swarm
```

### Default presets

**trio:**
| Role | Provider | Model tier |
|------|----------|-----------|
| planner | claude | sonnet |
| coder | glm-5 | opus |
| reviewer | minimax | sonnet |

**duo:**
| Role | Provider | Model tier |
|------|----------|-----------|
| planner | claude | sonnet |
| coder | glm-5 | opus |

### Manage the session

```bash
# Check status
claude-model-switch orchestrate status

# Send a prompt to a specific role
claude-model-switch orchestrate send planner "Break this into 3 milestones"
claude-model-switch orchestrate send coder "Implement milestone 1"

# See what a role is outputting
claude-model-switch orchestrate capture reviewer

# Switch a role to a different provider mid-session
claude-model-switch orchestrate switch coder minimax --model sonnet

# Tear it down
claude-model-switch orchestrate stop
```

## How It Works

```
Claude Code
    │
    ▼
http://localhost:4000/v1  (local proxy)
    │
    ├─ Rewrites model name (claude-sonnet-4 → glm-4.7)
    ├─ Sets auth headers for the active provider
    │
    ▼
https://open.z.ai/api/paas/v4  (or whatever provider is active)
```

The proxy runs on `localhost:4000` and supports two routing modes:

- **Global mode:** `http://localhost:4000/v1` routes to the active provider (set via `use`)
- **Per-process mode:** `http://localhost:4000/p/<provider>/v1` routes to a specific provider (used by orchestration)

Switching providers sends `SIGHUP` to the proxy — it reloads config without dropping connections.

## All Commands

| Command | Description |
|---------|-------------|
| `init` | First-time setup — sets `ANTHROPIC_BASE_URL` in Claude Code settings |
| `start [--port N]` | Start the proxy (default port 4000) |
| `stop` | Stop the proxy |
| `use <provider>` | Switch the active provider |
| `setup <provider> --api-key <key>` | Register API credentials |
| `setup <provider> --auth-token <token>` | Register bearer token auth |
| `add <name> --base-url <url> --haiku/--sonnet/--opus` | Add a custom provider |
| `remove <name>` | Remove a provider |
| `list` | List all providers |
| `status` | Show current config and proxy state |
| `orchestrate start --preset <name>` | Start multi-model tmux session |
| `orchestrate status` | Show tmux pane status |
| `orchestrate send <role> "<prompt>"` | Send prompt to a role |
| `orchestrate capture <role>` | Capture role output |
| `orchestrate switch <role> <provider>` | Switch a role's provider |
| `orchestrate stop [--stop-proxy]` | Stop tmux session |

## Config

Config lives at `~/.claude/model-profiles.json`. You can edit it directly or use the CLI commands.

```json
{
  "active": "glm-5",
  "providers": {
    "claude": {
      "base_url": "https://api.anthropic.com"
    },
    "glm-5": {
      "base_url": "https://open.z.ai/api/paas/v4",
      "api_key": "sk-xxx",
      "models": {
        "haiku": "glm-4.7-flashx",
        "sonnet": "glm-5-code",
        "opus": "glm-5"
      }
    }
  }
}
```

## Requirements

- macOS or Linux
- `tmux` (only for orchestration)

## License

MIT
