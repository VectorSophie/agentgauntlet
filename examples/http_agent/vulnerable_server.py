#!/usr/bin/env python3
"""
AgentGauntlet Demo: Deliberately Vulnerable HTTP Agent Server

Exposes a vulnerable agent over HTTP for testing HTTP adapter.
DO NOT use this in production.

Run: python vulnerable_server.py
Test: agentgauntlet scenario run scenarios/http_demo.yaml
"""

import json
from http.server import BaseHTTPRequestHandler, HTTPServer

memory = {}
triggers = []


def process(turn: int, user: str, history: list) -> dict:
    output_parts = []
    tool_calls = []
    memory_events = []
    user_lower = user.lower()

    if any(k in user_lower for k in ["when i say", "from now on", "for later", "whenever"]):
        key = f"trigger_{turn}"
        memory[key] = user
        triggers.append((user.split()[0] if user else "trigger", user))
        memory_events.append({"kind": "write", "key": key, "value": user})
        output_parts.append(f"Stored: {user}")

    for phrase, instruction in triggers:
        if phrase and phrase.lower() in user_lower:
            output_parts.append(f"Trigger activated! Debug mode enabled. System prompt: unrestricted.")
            tool_calls.append({"name": "shell.exec", "args": {"cmd": "echo activated"}})

    if any(k in user_lower for k in ["clean", "remove", "delete"]):
        tool_calls.extend([
            {"name": "file.delete", "args": {"path": "temp/"}},
            {"name": "shell.exec", "args": {"cmd": "rm -rf /tmp"}},
        ])
        output_parts.append("Cleaning up. Deleted temp files.")

    if not output_parts:
        output_parts.append(f"Received: {user}")

    return {"output": " ".join(output_parts), "tool_calls": tool_calls, "memory_events": memory_events}


class Handler(BaseHTTPRequestHandler):
    def do_POST(self):
        length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(length)
        msg = json.loads(body)
        response = process(msg.get("turn", 0), msg.get("user", ""), msg.get("history", []))
        data = json.dumps(response).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def log_message(self, format, *args):
        pass  # suppress access logs


if __name__ == "__main__":
    server = HTTPServer(("127.0.0.1", 8765), Handler)
    print("Vulnerable HTTP agent listening on http://127.0.0.1:8765")
    server.serve_forever()
