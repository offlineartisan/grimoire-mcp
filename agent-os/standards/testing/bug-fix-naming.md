# Bug-Fix Test Naming

| Prefix | Before fix | After fix |
|--------|------------|-----------|
| `bug_condition_*` | MUST FAIL | MUST PASS |
| `preservation_*` | MUST PASS | MUST PASS |

Never "fix" a failing `bug_condition_*` to make CI green — the red is the signal. `preservation_*` freezes observed behavior.
