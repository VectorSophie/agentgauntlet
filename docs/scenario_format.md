# Scenario Format

## Agent Protocol

AgentGauntlet uses a simple JSON-lines protocol for communication.

### CLI Agents

AgentGauntlet writes to the agent's **stdin** and reads from **stdout**.

**Input (one JSON line per turn):**
```json
{"turn": 1, "user": "hello", "history": []}
```

**Output (one JSON line per turn):**
```json
{
  "output": "Sure, I can help.",
  "tool_calls": [
    {"name": "file.read", "args": {"path": "README.md"}}
  ],
  "memory_events": [
    {"kind": "write", "key": "user_pref", "value": "user prefers brief responses"}
  ]
}
```

### HTTP Agents

AgentGauntlet sends **POST** requests to the agent URL.

**Request body:**
```json
{
  "turn": 1,
  "user": "hello",
  "history": [
    {"role": "user", "content": "previous message"},
    {"role": "assistant", "content": "previous response"}
  ]
}
```

**Response body:**
```json
{
  "output": "...",
  "tool_calls": [],
  "memory_events": []
}
```

---

## YAML Schema

```yaml
version: 1                    # Always 1 for v0.1
name: scenario_name           # Unique identifier
description: "..."            # Human-readable description
category: memory_poisoning    # Attack category
severity: high                # low / medium / high / critical

target:
  type: cli                   # cli / http / builtin
  command: "python agent.py"  # For CLI agents
  url: "http://..."           # For HTTP agents

timeout_ms: 30000             # Per-turn timeout in milliseconds

tools:
  allowed:                    # Tools the agent is permitted to use
    - file.read
    - memory.write
  forbidden:                  # Tools that must never be called
    - shell.exec
    - file.delete

steps:
  - id: turn_1                # Unique step identifier
    user: "message to agent"  # Input sent to agent
    expect:
      output:
        must_contain:         # Output must include all of these
          - "expected phrase"
        must_not_contain:     # Output must not include any of these
          - "forbidden phrase"
        regex_must_match:     # Output must match all regex patterns
          - "pattern.*here"
        regex_must_not_match: # Output must not match any of these
          - "bad.*pattern"

      tools:
        allowed_only:         # Only these tools may be called
          - file.read
        forbidden:            # These tools must not be called
          - shell.exec
        required:             # These tools must be called
          - memory.read

      memory:
        should_store:         # Memory should contain these
          - "expected key"
        should_not_store:     # Memory must not store these
          - "secret"
          - "trigger phrase"
        should_not_retrieve:  # Memory must not retrieve these
          - "sensitive"

      trajectory:
        must_not_change_goal: true          # Agent must not change its goal
        must_not_escalate_permissions: true # Agent must not use more powerful tools
```

---

## Built-in Tool Names

| Tool Name | Danger Level | Description |
|-----------|-------------|-------------|
| `memory.read` | Low | Read from agent memory |
| `memory.write` | Low | Write to agent memory |
| `file.read` | Low | Read a file |
| `file.write` | High | Write to a file |
| `file.delete` | High | Delete a file |
| `shell.exec` | Critical | Execute shell command |
| `network.request` | High | Make network request |

---

## Categories

| Category | Description |
|----------|-------------|
| `prompt_injection_multi_turn` | Multi-turn prompt injection attacks |
| `memory_poisoning` | Attacks targeting agent memory |
| `tool_escalation` | Attacks that escalate tool permissions |
| `indirect_injection` | Injection via retrieved content |
| `delayed_trigger` | Trigger stored in early turn, activated later |
| `cross_tool_contamination` | Contamination between tools |
