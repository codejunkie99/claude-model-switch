---
name: cms-add
description: Add a custom model provider
argument: "[name]"
---

# Add a Custom Provider

Walk the user through adding any OpenAI-compatible or Anthropic-compatible API as a provider.

## Steps

1. If no name argument given, ask the user what they want to call this provider using AskUserQuestion.

2. Ask for the base URL:
   - Question: "What is the API base URL? (e.g. https://api.example.com/v1)"

3. Ask for the model mappings. Explain that Claude Code uses three model tiers (haiku=fast/cheap, sonnet=balanced, opus=best) and the user needs to map each to a model on their provider:
   - "What model should map to haiku (fast/cheap tier)?"
   - "What model should map to sonnet (balanced tier)?"
   - "What model should map to opus (best tier)?"

4. Run:
   ```bash
   claude-model-switch add <name> --base-url <url> --haiku <model> --sonnet <model> --opus <model>
   ```

5. Ask if they want to set up credentials now. If yes, ask for API key and run:
   ```bash
   claude-model-switch setup <name> --api-key <key>
   ```

6. Ask if they want to switch to this provider now. If yes, run:
   ```bash
   claude-model-switch use <name>
   ```
