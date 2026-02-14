# claude-model-switch

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Release](https://img.shields.io/github/v/release/codejunkie99/claude-model-switch)](https://github.com/codejunkie99/claude-model-switch/releases/latest)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux-lightgrey)](https://github.com/codejunkie99/claude-model-switch/releases/latest)

Use **any model** with Claude Code. Add your own providers and API keys, switch instantly, and run multi-model tmux orchestration — no restart needed.

A lightweight local proxy that sits between Claude Code and any OpenAI-compatible or Anthropic-compatible API.

## Install

### Prebuilt binaries (recommended)

**One-liner:**

```bash
curl -fsSL https://raw.githubusercontent.com/codejunkie99/claude-model-switch/main/install.sh | sh
```

**Or download directly:**

| Platform | Download |
|----------|----------|
| macOS Apple Silicon | [claude-model-switch-aarch64-apple-darwin.tar.gz](https://github.com/codejunkie99/claude-model-switch/releases/latest/download/claude-model-switch-aarch64-apple-darwin.tar.gz) |
| macOS Intel | [claude-model-switch-x86_64-apple-darwin.tar.gz](https://github.com/codejunkie99/claude-model-switch/releases/latest/download/claude-model-switch-x86_64-apple-darwin.tar.gz) |
| Linux x64 | [claude-model-switch-x86_64-unknown-linux-gnu.tar.gz](https://github.com/codejunkie99/claude-model-switch/releases/latest/download/claude-model-switch-x86_64-unknown-linux-gnu.tar.gz) |
| Linux ARM64 | [claude-model-switch-aarch64-unknown-linux-gnu.tar.gz](https://github.com/codejunkie99/claude-model-switch/releases/latest/download/claude-model-switch-aarch64-unknown-linux-gnu.tar.gz) |

Extract and move to your PATH:

```bash
tar xzf claude-model-switch-*.tar.gz
mv claude-model-switch ~/.local/bin/
```

### Cargo

```bash
cargo install claude-model-switch
```

### From source

```bash
git clone https://github.com/codejunkie99/claude-model-switch.git
cd claude-model-switch
cargo install --path .
```

## Quick Start

```bash
# 1. Point Claude Code at the local proxy
claude-model-switch init

# 2. One command: add provider + save key (preset base URL)
claude-model-switch add glm sk-xxx

# 3. Start the proxy
claude-model-switch start

# 4. Switch to it
claude-model-switch use glm

# 5. Switch back anytime
claude-model-switch use claude
```

That's it. Claude Code now routes through your chosen provider.

## Add Any Provider

Add any API endpoint that speaks the Anthropic or OpenAI messages format:

```bash
claude-model-switch add <name> \
  <api-url> \
  <api-key>

# or set credentials later:
claude-model-switch setup <name> --api-key <your-key>
```

This creates a **passthrough provider**: whatever model ID Claude sends is forwarded as-is.

Built-in provider presets let users skip `--base-url`:

```bash
claude-model-switch add glm sk-xxx
claude-model-switch add openrouter sk-or-xxx
claude-model-switch add minimax xxx
```

If a provider already exists, you can update only the key and it reuses the saved base URL:

```bash
# First time
claude-model-switch add acme https://api.acme.ai/v1 sk-old

# Later key rotation (base URL reused automatically)
claude-model-switch add acme sk-new
```

Flag-based syntax still works if you prefer it:

```bash
claude-model-switch add <name> --base-url <api-url> --api-key <your-key>
```

If you want Claude-tier rewriting, pass all three mapping flags:

```bash
claude-model-switch add <name> \
  --base-url <api-url> \
  --haiku <fast-model> \
  --sonnet <balanced-model> \
  --opus <best-model>
```

Claude Code uses three model tiers internally. Tier mapping lets you map each tier to whatever model your provider offers:

- **haiku** — fast/cheap tier (used for quick tasks)
- **sonnet** — balanced tier (used for most coding)
- **opus** — best tier (used for complex reasoning)

### Examples

```bash
# GLM via Z.ai
claude-model-switch add glm \
  --base-url https://open.z.ai/api/paas/v4 \
  --haiku glm-4.5-air \
  --sonnet glm-4.7 \
  --opus glm-5
claude-model-switch setup glm --api-key sk-xxx

# MiniMax
claude-model-switch add minimax \
  --base-url https://api.minimax.io/anthropic/v1 \
  --haiku MiniMax-M2 \
  --sonnet MiniMax-M2.5 \
  --opus MiniMax-M2.5
claude-model-switch setup minimax --api-key xxx

# OpenRouter
claude-model-switch add openrouter \
  --base-url https://openrouter.ai/api/v1 \
  --haiku google/gemini-2.5-flash \
  --sonnet anthropic/claude-sonnet-4 \
  --opus deepseek/deepseek-r1
claude-model-switch setup openrouter --api-key sk-or-xxx

# Any custom endpoint
claude-model-switch add my-llm \
  --base-url https://api.example.com/v1 \
  --haiku small-model \
  --sonnet medium-model \
  --opus large-model
claude-model-switch setup my-llm --api-key xxx
```

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

## Multi-Model Orchestration

Run multiple Claude Code instances in tmux, each using a different provider. Useful for parallel workstreams where different models have different strengths.

### Start a session

```bash
# Trio: 3 panes using your first 3 configured providers
claude-model-switch orchestrate start --preset trio

# Duo: 2 panes using your first 2 configured providers
claude-model-switch orchestrate start --preset duo

# Attach to see all panes
tmux attach -t cms-swarm
```

Presets automatically assign roles to your configured providers (sorted alphabetically):

- **trio** — requires 3+ providers: planner (1st/sonnet), coder (2nd/opus), reviewer (3rd/sonnet)
- **duo** — requires 2+ providers: planner (1st/sonnet), coder (2nd/opus)

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
claude-model-switch orchestrate switch coder openrouter --model sonnet

# Tear it down
claude-model-switch orchestrate stop
```

## How It Works

```
Claude Code
    |
    v
http://localhost:4000/v1  (local proxy)
    |
    +-- Rewrites model name (claude-sonnet-4 -> your-model)
    +-- Sets auth headers for the active provider
    |
    v
https://api.your-provider.com/v1  (wherever you point it)
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
| `add <name> [<base-url>] [<api-key>] [--haiku <m> --sonnet <m> --opus <m>]` | Add/update provider (for presets, `add <name> <api-key>` works) |
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
  "active": "openrouter",
  "providers": {
    "claude": {
      "base_url": "https://api.anthropic.com"
    },
    "openrouter": {
      "base_url": "https://openrouter.ai/api/v1",
      "api_key": "sk-or-xxx",
      "models": {
        "haiku": "google/gemini-2.5-flash",
        "sonnet": "anthropic/claude-sonnet-4",
        "opus": "deepseek/deepseek-r1"
      }
    }
  }
}
```

## Troubleshooting

### `claude-model-switch: command not found`

The binary isn't on your PATH. Either:
- Add `~/.local/bin` to your PATH: `export PATH="$HOME/.local/bin:$PATH"` (add to `~/.bashrc` or `~/.zshrc`)
- Or move the binary somewhere already on your PATH: `mv claude-model-switch /usr/local/bin/`

### Proxy won't start / port already in use

```bash
# Check if something is using port 4000
lsof -i :4000

# Stop any existing proxy
claude-model-switch stop

# Start on a different port
claude-model-switch start --port 4001
```

If you change the port, also update `~/.claude/settings.json` to match:
```json
{ "env": { "ANTHROPIC_BASE_URL": "http://localhost:4001/v1" } }
```

### Claude Code not routing through the proxy

1. Make sure `init` was run: `claude-model-switch init`
2. Check `~/.claude/settings.json` has `ANTHROPIC_BASE_URL` set to `http://localhost:4000/v1`
3. Make sure the proxy is running: `claude-model-switch status`
4. Restart Claude Code after running `init` for the first time

### Provider returns errors

- Verify your API key: `claude-model-switch status` shows the active provider config
- Check that the base URL is correct (some providers need `/v1` at the end, some don't)
- Make sure the model names match what your provider expects exactly

### Orchestration: `tmux: command not found`

Install tmux:
- macOS: `brew install tmux`
- Ubuntu/Debian: `sudo apt install tmux`
- Fedora: `sudo dnf install tmux`

### Stale proxy after crash

If the proxy crashed but the PID file remains:
```bash
rm ~/.claude/model-switch-proxy.pid
claude-model-switch start
```

## Contributing

Contributions welcome. Here's how:

1. Fork the repo
2. Create a branch: `git checkout -b my-feature`
3. Make your changes
4. Run tests: `cargo test`
5. Commit and push
6. Open a PR

### Building from source

```bash
git clone https://github.com/codejunkie99/claude-model-switch.git
cd claude-model-switch
cargo build
cargo test
```

### Cross-compiling

The project uses `rustls` (no OpenSSL dependency), so cross-compilation works with [cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild):

```bash
cargo install cargo-zigbuild
cargo zigbuild --release --target x86_64-unknown-linux-gnu
cargo zigbuild --release --target aarch64-unknown-linux-gnu
```

## Requirements

- macOS or Linux
- `tmux` (only for orchestration)

## License

MIT
