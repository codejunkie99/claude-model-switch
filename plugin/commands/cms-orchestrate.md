---
name: cms-orchestrate
description: Launch multi-model tmux orchestration session
argument: "[preset]"
---

# Multi-Model Orchestration

Launch a tmux session with multiple Claude Code instances, each routed through a different model provider.

## Steps

1. Check that tmux is installed by running `tmux -V`. If not, tell the user to install it.

2. If no preset argument given, present options using AskUserQuestion:
   - **trio**: 3 panes — planner (Claude/sonnet), coder (GLM-5/opus), reviewer (MiniMax/sonnet)
   - **duo**: 2 panes — planner (Claude/sonnet), coder (GLM-5/opus)

3. Ask which working directory to use:
   - Current directory (recommended)
   - Other (let them type a path)

4. Run:
   ```bash
   claude-model-switch orchestrate start --preset <preset> --cwd <dir>
   ```

5. Show the user how to attach:
   ```
   tmux attach -t cms-swarm
   ```

6. Explain the available follow-up commands:
   - `claude-model-switch orchestrate status` — see pane status
   - `claude-model-switch orchestrate send <role> "prompt"` — send work to a role
   - `claude-model-switch orchestrate capture <role>` — see a role's output
   - `claude-model-switch orchestrate switch <role> <provider>` — change a role's provider
   - `claude-model-switch orchestrate stop` — tear down the session
