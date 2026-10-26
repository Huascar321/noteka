#!/usr/bin/env python3
"""Smoke offline CLI: build semántico con embeddings loopback, buscar, degradar y
ejecutar el harness con chat loopback. No usa modelos reales ni red externa."""
import argparse
import json
import math
import re
import subprocess
import tempfile
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

DIMENSION = 8


def vector(text):
    values = [0.0] * DIMENSION
    for word in re.findall(r"[a-z0-9]+", text.lower()):
        h = 0
        for byte in word.encode():
            h = (h * 31 + byte) % (2**32)
        values[h % DIMENSION] += 1.0
    if not any(values):
        values[0] = 1.0
    norm = math.sqrt(sum(v * v for v in values))
    return [v / norm for v in values]


class EmbeddingHandler(BaseHTTPRequestHandler):
    def do_POST(self):
        length = int(self.headers.get("Content-Length", "0"))
        body = json.loads(self.rfile.read(length))
        inputs = body.get("input", [])
        data = [{"index": i, "embedding": vector(text)} for i, text in enumerate(inputs)]
        payload = json.dumps({"data": data}).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(payload)))
        self.end_headers()
        self.wfile.write(payload)

    def log_message(self, *args):
        pass


class ChatHandler(BaseHTTPRequestHandler):
    responses = []
    index = 0

    def do_POST(self):
        length = int(self.headers.get("Content-Length", "0"))
        self.rfile.read(length)
        content = ChatHandler.responses[ChatHandler.index]
        ChatHandler.index += 1
        payload = json.dumps({"choices": [{"message": {"content": content}}]}).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(payload)))
        self.end_headers()
        self.wfile.write(payload)

    def log_message(self, *args):
        pass


def server(handler):
    httpd = ThreadingHTTPServer(("127.0.0.1", 0), handler)
    port = httpd.server_address[1]
    thread = threading.Thread(target=httpd.serve_forever, daemon=True)
    thread.start()
    return httpd, f"http://127.0.0.1:{port}"


def run(cli, workspace, command, payload):
    result = subprocess.run(
        [str(cli), "--json", "--workspace", str(workspace), *command],
        input=json.dumps(payload), text=True, capture_output=True, timeout=30,
    )
    assert not result.stderr, result.stderr
    envelope = json.loads(result.stdout)
    assert envelope["ok"], envelope
    return envelope["data"]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--cli", type=Path, required=True)
    parser.add_argument("--profile", default="debug")
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix="noteka-semantic-agent-") as temp:
        root = Path(temp)
        workspace = root / "workspace"
        source = root / "alpha.txt"
        source.write_text("alpha beta gamma\n", encoding="utf-8")
        run(args.cli, workspace, ["workspace", "create"], {})
        attachment = run(args.cli, workspace, ["attachment", "import"], {"source_path": str(source)})
        enqueued = run(args.cli, workspace, ["extraction", "enqueue"], {
            "attachment_id": attachment["id"], "extractor": "utf8_text",
        })
        job_id = enqueued["job"]["id"]
        control = run(args.cli, workspace, ["extraction", "control"], {"action": "get"})
        run(args.cli, workspace, ["extraction", "control"], {
            "action": "set", "paused": False, "expected_revision": control["revision"],
        })
        job = run(args.cli, workspace, ["extraction", "run"], {"id": job_id})["job"]
        generation = job["generation"]["id"]
        index_revision = run(args.cli, workspace, ["extraction", "index"], {"action": "get"})["revision"]
        run(args.cli, workspace, ["extraction", "index"], {
            "action": "put", "job_id": job_id,
            "expected_generation_id": generation, "expected_revision": index_revision,
        })

        embedding_server, embedding_url = server(EmbeddingHandler)
        try:
            profile = {
                "version": 1, "provider": "local_http", "runtime": "openai_embeddings_v1",
                "model": "fixture-embedding", "revision": "smoke-v1", "dimension": DIMENSION,
                "normalization": "l2", "task": "retrieval",
                "query_prefix": "Query: ", "document_prefix": "Document: ",
                "tokenizer": "fixture", "chunking": "extraction_fragment_v1",
                "quantization": "f32",
            }
            built = run(args.cli, workspace, ["embedding", "index"], {
                "action": "build", "profile": profile, "endpoint": embedding_url + "/v1/embeddings",
            })
            assert built["generation"]["chunk_count"] == 1, built
            semantic = run(args.cli, workspace, ["retrieval", "search"], {
                "query": "alpha", "mode": "semantic", "limit": 5,
            })
            assert semantic["mode_used"] == "semantic", semantic
            assert semantic["items"] and semantic["items"][0]["job_id"] == job_id, semantic
        finally:
            embedding_server.shutdown()
            embedding_server.server_close()

        fallback = run(args.cli, workspace, ["retrieval", "search"], {
            "query": "alpha", "mode": "hybrid", "limit": 5,
        })
        assert fallback["mode_used"] == "lexical", fallback
        assert "query_embedding_failed" in fallback["degraded"], fallback
        assert fallback["items"], fallback

        tool_call = json.dumps({
            "type": "tool_call", "tool": "retrieval.search",
            "input": {"query": "alpha", "mode": "lexical"},
        })
        final = json.dumps({
            "type": "final", "answer": "El fragmento menciona alpha.",
            "citations": [{"job_id": job_id, "generation_id": generation,
                           "ordinal": 0, "quote": "alpha beta"}],
            "insufficient_evidence": False,
        })
        ChatHandler.responses = [tool_call, final]
        ChatHandler.index = 0
        chat_server, chat_url = server(ChatHandler)
        try:
            asked = run(args.cli, workspace, ["agent", "ask"], {
                "question": "¿alpha?",
                "provider": {"runtime": "openai_chat_completions_v1",
                             "endpoint": chat_url + "/v1/chat/completions",
                             "model": "mock-chat", "timeout_ms": 5000,
                             "max_tokens": 256, "temperature": 0.0},
                "limits": {"max_steps": 4, "max_context_bytes": 65536,
                           "max_tool_bytes": 65536, "deadline_ms": 10000},
                "allow_apply": False,
            })
            assert asked["run"]["state"] == "succeeded", asked
            assert asked["citations"][0]["verified"], asked
            trace = run(args.cli, workspace, ["agent", "trace"], {
                "action": "get", "id": asked["run"]["id"],
            })
            assert len(trace["steps"]) == 3, trace
        finally:
            chat_server.shutdown()
            chat_server.server_close()
        print(json.dumps({
            "profile": args.profile,
            "schema": 18,
            "semantic_chunks": built["generation"]["chunk_count"],
            "semantic_mode": semantic["mode_used"],
            "fallback_mode": fallback["mode_used"],
            "fallback_degraded": fallback["degraded"],
            "agent_state": asked["run"]["state"],
            "citations_verified": len(asked["citations"]),
            "agent_steps": len(trace["steps"]),
            "external_network": False,
        }, indent=2))


if __name__ == "__main__":
    main()
