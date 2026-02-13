---
name: cms-remove
description: Remove a model provider
argument: "[name]"
---

# Remove Provider

Remove a configured model provider.

## Steps

1. If no name argument given, run `claude-model-switch list` and ask the user which provider to remove using AskUserQuestion.

2. Confirm with the user that they want to remove this provider.

3. Run:
   ```bash
   claude-model-switch remove <name>
   ```

4. Show confirmation.
