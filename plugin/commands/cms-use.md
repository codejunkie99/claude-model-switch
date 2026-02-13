---
name: cms-use
description: Switch to a different model provider
argument: "[provider]"
---

# Switch Model Provider

Switch Claude Code to use a different model provider instantly.

## Steps

1. Run `claude-model-switch list` to get available providers.

2. If no provider argument was given, present the available providers using AskUserQuestion and let the user pick one.

3. Run:
   ```bash
   claude-model-switch use <provider>
   ```

4. Run `claude-model-switch status` and show the user what provider and models are now active.
