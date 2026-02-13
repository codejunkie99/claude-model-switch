# claude-model-switch

Local API proxy for seamless Claude Code model provider switching.

Switch between model providers (Anthropic, GLM, MiniMax, custom) instantly — no Claude Code restart needed.

## Install

```bash
cargo install claude-model-switch
```

## Quick Start

```bash
# First-time setup
claude-model-switch init

# Add your provider credentials
claude-model-switch setup glm --api-key sk-your-key-here

# Start the proxy
claude-model-switch start

# Switch providers (instant, no restart)
claude-model-switch use glm
claude-model-switch use claude
claude-model-switch use minimax
```

## Built-in Providers

| Provider | Models |
|----------|--------|
| claude | Default Anthropic models (passthrough) |
| glm | GLM-4.5-air / GLM-4.7 |
| glm-flash | GLM-4.7-FlashX (all tiers) |
| glm-5 | GLM-5 flagship (GLM-5-code / GLM-5) |
| minimax | MiniMax M2 / M2.5 |
| minimax-fast | MiniMax M2.5-Lightning |

## Custom Providers

```bash
claude-model-switch add my-provider \
  --base-url https://api.example.com/v1 \
  --haiku model-small \
  --sonnet model-medium \
  --opus model-large

claude-model-switch setup my-provider --api-key sk-xxx
```

## How It Works

The tool runs a lightweight HTTP proxy on `localhost:4000`. Claude Code sends requests to the proxy, which:

1. Rewrites model names (e.g., `claude-sonnet-4` → `glm-4.7`)
2. Sets the correct auth headers for the active provider
3. Forwards the request to the provider's API
4. Returns the response to Claude Code

Switching providers changes where the proxy routes requests — no Claude Code restart needed.

## Commands

| Command | Description |
|---------|-------------|
| `init` | First-time setup (configures Claude Code) |
| `start [--port N]` | Start proxy (default: port 4000) |
| `stop` | Stop proxy |
| `use <provider>` | Switch active provider |
| `setup <provider>` | Register credentials |
| `add <name>` | Add custom provider |
| `remove <name>` | Remove a provider |
| `list` | List providers |
| `status` | Show current config |

## License

MIT
