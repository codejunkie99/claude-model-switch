---
name: cms-status
description: Show current model provider and proxy status
---

# Model Switch Status

Show the user the current state of their model switching setup.

## Steps

1. Run `claude-model-switch status` and display the output.

2. Check if the proxy is running by looking for the PID file at `~/.claude/model-switch-proxy.pid` and checking if the process is alive.

3. Present a clear summary:
   - Active provider
   - Model mappings (haiku/sonnet/opus)
   - Proxy status (running/stopped, port)
