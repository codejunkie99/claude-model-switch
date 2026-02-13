---
name: cms-setup
description: Set up a model provider with API credentials
argument: "[provider]"
---

# Set Up a Model Provider

Help the user register API credentials for a model provider.

## Steps

1. If no provider argument was given, run `claude-model-switch list` to show available providers, then ask the user which provider they want to set up using AskUserQuestion.

2. Ask the user for their API key using AskUserQuestion:
   - Question: "Paste your API key for <provider>"
   - Options are not applicable here — use an open-ended question by providing a dummy option and letting the user type via "Other"

3. Run the setup command:
   ```bash
   claude-model-switch setup <provider> --api-key <key>
   ```

4. If the user provided an auth token instead of an API key, use `--auth-token` instead.

5. After setup, ask if they want to switch to this provider now. If yes, run:
   ```bash
   claude-model-switch use <provider>
   ```

6. Show confirmation of what was configured.
