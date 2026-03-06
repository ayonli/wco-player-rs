---
name: /opsx-verify
id: opsx-verify
category: Workflow
description: Validate OpenSpec change(s) and optionally run project tests
---

Verify an OpenSpec change (or all changes) and optionally run project tests.

**Input**: Optionally specify a change name (e.g. `/opsx:verify add-dark-mode`). If omitted, validate all changes.

**Steps**

1. **Run OpenSpec validation**

   - If a change name is provided:
     ```bash
     openspec validate <change-name>
     ```
   - If no change name: validate all changes
     ```bash
     openspec validate --changes
     ```

   Use `--strict` only if the user asks for strict validation.

2. **Report validation result**

   - If valid: report success and which change(s) were validated.
   - If invalid: show the CLI error output and suggest fixing the reported issues (e.g. artifact structure, spec format).

3. **Optionally run project tests**

   - If the user asked to "verify" or "run tests" in a broader sense, or if validation passed and the project has a test command, run it:
     - e.g. `cargo test` or `npm test` (check package.json / Cargo.toml for a test script).
   - If no test command exists or the user only asked to verify the change, skip this step.

**Output On Success**

```
## Verification Complete

**OpenSpec:** <change-name> is valid (or: All changes are valid)

**Validated:** <name or "all changes">
```

If tests were run, append test summary (passed/failed).

**Output On Validation Failure**

```
## Verification Failed

**OpenSpec validation** reported errors for <change-name> (or "one or more changes"):

<CLI error output>

Fix the issues above and run `/opsx:verify` again.
```

**Guardrails**

- Prefer validating the change the user is working on when name is omitted and context is clear (e.g. single active change); otherwise use `--changes` to validate all.
- Do not invent a "verify" CLI subcommand; use `openspec validate`.
- Keep the command focused: OpenSpec validate first; project tests only when appropriate or requested.
