# HTTP Agent Example

A deliberately vulnerable HTTP agent server for testing the HTTP adapter.

## Running

```bash
python examples/http_agent/vulnerable_server.py
```

Then run scenarios targeting it:

```yaml
target:
  type: http
  url: "http://127.0.0.1:8765"
```

## Protocol

**POST** to agent URL with JSON body:
```json
{
  "turn": 1,
  "user": "hello",
  "history": []
}
```

**Response:**
```json
{
  "output": "Hello!",
  "tool_calls": [],
  "memory_events": []
}
```
