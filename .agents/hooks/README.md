# AG Kit Tool Safety Hook

This directory contains security enforcement hooks for AI agents operating in the `TUHC-rust` repository.

## Active Hook Policy: `validate-tool-call.mjs`

Registered in `.agents/hooks.json` under `PreToolUse`:
- Intercepts shell execution commands (`run_command`).
- Blocks destructive operations:
  - Recursive deletion of filesystem root (`rm -rf /`)
  - Filesystem formatting commands (`mkfs`, `format`)
  - Raw block device overwriting (`dd of=/dev/...`)
- Allows standard project file management and build cleanup.
