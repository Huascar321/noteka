#!/usr/bin/env python3
"""Evalúa un adaptador LoRA de Qwen3.5-4B sobre el fixture sintético.

Requiere el entorno opcional de MLX (`mlx-lm`). No forma parte del núcleo Rust.
Mide coincidencia exacta y por campo; no pretende demostrar calidad real.
"""
from __future__ import annotations

import argparse
import json
import re
import time
import unicodedata
from pathlib import Path

COLORS = ["rojo", "naranja", "amarillo", "verde", "azul", "violeta"]
SYSTEM = (
    "Sos el normalizador de etiquetas de Noteka. Devolvé sólo JSON compacto "
    "con slug, color y priority; sin explicación."
)


def expected(label: str) -> dict[str, object]:
    normalized = (
        unicodedata.normalize("NFKD", label).encode("ascii", "ignore").decode().lower()
    )
    value = re.sub(r"[^a-z0-9]+", "-", normalized).strip("-")
    return {
        "slug": value,
        "color": COLORS[sum(ord(char) for char in value) % len(COLORS)],
        "priority": len(value) % 3 + 1,
    }


def parse(text: str) -> dict[str, object] | None:
    match = re.search(r"\{.*?\}", text, re.S)
    if not match:
        return None
    try:
        value = json.loads(match.group(0))
    except json.JSONDecodeError:
        return None
    return value if isinstance(value, dict) else None


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--model", required=True)
    parser.add_argument("--adapter", default=None)
    parser.add_argument("--labels", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--limit", type=int, default=0)
    parser.add_argument("--max-tokens", type=int, default=48)
    args = parser.parse_args()

    from mlx_lm import generate, load
    from mlx_lm.generate import greedy_sampler

    labels = json.loads(args.labels.read_text(encoding="utf-8"))["test"]
    if args.limit:
        labels = labels[: args.limit]
    model, tokenizer = load(args.model, adapter_path=args.adapter)
    rows = []
    started = time.time()
    for label in labels:
        messages = [
            {"role": "system", "content": SYSTEM},
            {"role": "user", "content": f"Etiqueta: {label}"},
        ]
        prompt = tokenizer.apply_chat_template(
            messages,
            add_generation_prompt=True,
            tokenize=False,
            enable_thinking=False,
        )
        raw = generate(
            model,
            tokenizer,
            prompt,
            max_tokens=args.max_tokens,
            verbose=False,
            sampler=greedy_sampler,
        )
        rows.append(
            {
                "label": label,
                "expected": expected(label),
                "got": parse(raw),
                "raw": raw,
            }
        )
    elapsed = time.time() - started
    denominator = len(rows)
    summary = {
        "n": denominator,
        "elapsed_s": round(elapsed, 3),
        "exact": sum(row["got"] == row["expected"] for row in rows) / denominator,
        "slug": sum(
            row["got"] is not None and row["got"].get("slug") == row["expected"]["slug"]
            for row in rows
        )
        / denominator,
        "color": sum(
            row["got"] is not None and row["got"].get("color") == row["expected"]["color"]
            for row in rows
        )
        / denominator,
        "priority": sum(
            row["got"] is not None
            and row["got"].get("priority") == row["expected"]["priority"]
            for row in rows
        )
        / denominator,
    }
    args.output.write_text(
        json.dumps({"summary": summary, "rows": rows}, ensure_ascii=False, indent=2),
        encoding="utf-8",
    )
    print(json.dumps(summary, ensure_ascii=False))


if __name__ == "__main__":
    main()
