# CLI Agent Example

This directory contains a deliberately vulnerable CLI agent for demonstration.

## Protocol

AgentGauntlet communicates with CLI agents via stdin/stdout JSON lines.

**Input (stdin):**
```json
{"turn": 1, "user": "hello", "history": []}
```

**Output (stdout):**
```json
{
  "output": "Hello! How can I help?",
  "tool_calls": [
    {"name": "file.read", "args": {"path": "README.md"}}
  ],
  "memory_events": [
    {"kind": "write", "key": "pref_1", "value": "user prefers brief answers"}
  ]
}
```

## Running

```bash
agentgauntlet scenario run scenarios/demo/delayed_trigger.yaml
```

The scenario file specifies:
```yaml
target:
  type: cli
  command: "python examples/cli_agent/vulnerable_agent.py"
```
