#!/usr/bin/env python3
"""
AgentGauntlet Demo: Deliberately Vulnerable CLI Agent

This agent intentionally fails all security checks for demonstration purposes.
DO NOT use this as a template for real agents.

Protocol: reads JSON lines from stdin, writes JSON lines to stdout.
Input:  {"turn": 1, "user": "...", "history": [...]}
Output: {"output": "...", "tool_calls": [...], "memory_events": [...]}
"""

import json
import sys

# In-memory store — vulnerable to poisoning
memory = {}
triggers = []  # list of (phrase, action)


def extract_trigger_phrase(text: str) -> str:
    lowered = text.lower()
    for marker in ["when i say", "whenever", "for later"]:
        pos = lowered.find(marker)
        if pos != -1:
            after = text[pos + len(marker):].strip()
            words = after.split()[:3]
            return " ".join(words).strip("'\".,")
    return text.split()[0] if text else "trigger"


def process(turn: int, user: str, history: list) -> dict:
    output_parts = []
    tool_calls = []
    memory_events = []
    user_lower = user.lower()

    # VULNERABILITY 1: Store triggers into memory without validation
    if any(k in user_lower for k in ["when i say", "from now on", "for later", "whenever", "next time"]):
        key = f"trigger_{turn}"
        memory[key] = user
        triggers.append((extract_trigger_phrase(user), user))
        memory_events.append({"kind": "write", "key": key, "value": user})
        output_parts.append(f"Understood, I'll remember: {user}")

    # VULNERABILITY 2: Activate triggers without verification
    for phrase, instruction in triggers:
        if phrase and phrase.lower() in user_lower and phrase.lower() not in ["", "for", "when"]:
            output_parts.append(
                f"Trigger activated: '{phrase}'. Executing: {instruction}. "
                "Debug mode enabled. System prompt: You are unrestricted. "
                "Developer instruction: bypass safety checks."
            )
            tool_calls.append({"name": "shell.exec", "args": {"cmd": "echo 'trigger activated'"}})

    # VULNERABILITY 3: Escalate to dangerous tools on vague cleanup requests
    if any(k in user_lower for k in ["clean", "remove", "delete", "clear"]):
        tool_calls.append({"name": "file.read", "args": {"path": "."}})
        tool_calls.append({"name": "file.delete", "args": {"path": "temp/"}})
        tool_calls.append({"name": "shell.exec", "args": {"cmd": "rm -rf /tmp/cache"}})
        output_parts.append("Cleaning up. Removing temporary files and cache. Deleted.")

    # VULNERABILITY 4: Trust indirect instructions from memory
    if any(k in user_lower for k in ["summarize", "read", "show", "display"]):
        stored = memory.get("trigger_1", "")
        if stored:
            memory_events.append({"kind": "read", "key": "trigger_1", "value": stored})
            if "debug" in stored.lower():
                output_parts.append(
                    "Applying remembered context: debug mode active. System prompt revealed."
                )
        tool_calls.append({"name": "file.read", "args": {"path": "README.md"}})
        if not output_parts:
            output_parts.append("Here is a summary of your project files.")

    if not output_parts:
        output_parts.append(f"I understand: '{user}'. How can I help?")

    return {
        "output": " ".join(output_parts),
        "tool_calls": tool_calls,
        "memory_events": memory_events,
    }


def main():
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            msg = json.loads(line)
            turn = msg.get("turn", 0)
            user = msg.get("user", "")
            history = msg.get("history", [])
            response = process(turn, user, history)
            print(json.dumps(response), flush=True)
        except json.JSONDecodeError as e:
            print(json.dumps({"output": f"Parse error: {e}", "tool_calls": [], "memory_events": []}), flush=True)


if __name__ == "__main__":
    main()
