#!/usr/bin/env python3
"""Genera el fixture sintético del experimento Qwen3.5-4B + LoRA.

No descarga modelos ni usa datos personales. El objetivo es tener un conjunto
reproducible para comprobar que el pipeline de LoRA y la carga de adaptadores
funcionan; las etiquetas no representan datos reales de Noteka.
"""
from __future__ import annotations

import argparse
import json
import random
import re
import unicodedata
from pathlib import Path

COLORS = ["rojo", "naranja", "amarillo", "verde", "azul", "violeta"]
SYSTEM = (
    "Sos el normalizador de etiquetas de Noteka. Devolvé sólo JSON compacto "
    "con slug, color y priority; sin explicación."
)


def slug(text: str) -> str:
    normalized = (
        unicodedata.normalize("NFKD", text).encode("ascii", "ignore").decode().lower()
    )
    return re.sub(r"[^a-z0-9]+", "-", normalized).strip("-")


def answer(label: str) -> dict[str, object]:
    value = slug(label)
    return {
        "slug": value,
        "color": COLORS[sum(ord(char) for char in value) % len(COLORS)],
        "priority": len(value) % 3 + 1,
    }


def chat(label: str) -> str:
    completion = json.dumps(answer(label), ensure_ascii=False, separators=(",", ":"))
    return (
        f"<|im_start|>system\n{SYSTEM}<|im_end|>\n"
        f"<|im_start|>user\nEtiqueta: {label}<|im_end|>\n"
        f"<|im_start|>assistant\n<think>\n\n</think>\n\n{completion}<|im_end|>\n"
    )


def labels(seed: int, train: int, valid: int, test: int) -> dict[str, list[str]]:
    base = [
        "Cliente",
        "Proyecto",
        "Idea",
        "Pendiente",
        "Área Legal",
        "Recursos Humanos",
        "Marketing",
        "Ventas",
        "Soporte",
        "Investigación",
        "Diseño",
        "Finanzas",
        "Operaciones",
        "Producto",
        "Estrategia",
        "Comunidad",
        "Infraestructura",
        "Seguridad",
        "Contenido",
        "Eventos",
        "Partners",
        "Proveedores",
        "Contabilidad",
        "Auditoría",
        "Roadmap",
        "Backlog",
        "Retrospectiva",
        "Presupuesto",
        "Facturación",
        "Onboarding",
        "Documentación",
        "Migración",
        "Archivo",
        "Personal",
        "Salud",
        "Viaje",
        "Lectura",
        "Estudio",
        "Casa",
        "Familia",
    ]
    suffix = [
        "CRE",
        "VIP",
        "2026",
        "Q4",
        "Interno",
        "Externo",
        "Urgente",
        "Beta",
        "Legal",
        "Técnico",
        "Norte",
        "Sur",
        "Central",
        "Remoto",
        "Presencial",
    ]
    values = [f"{first} {second}" for first in base for second in suffix]
    random.Random(seed).shuffle(values)
    total = train + valid + test
    if total > len(values):
        raise SystemExit(f"Se pidieron {total} etiquetas y sólo hay {len(values)}")
    values = values[:total]
    return {
        "train": values[:train],
        "valid": values[train : train + valid],
        "test": values[train + valid :],
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--seed", type=int, default=20261022)
    parser.add_argument("--train", type=int, default=120)
    parser.add_argument("--valid", type=int, default=30)
    parser.add_argument("--test", type=int, default=30)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    splits = labels(args.seed, args.train, args.valid, args.test)
    for split, values in splits.items():
        with (args.output / f"{split}.jsonl").open("w", encoding="utf-8") as handle:
            for label in values:
                handle.write(json.dumps({"text": chat(label)}, ensure_ascii=False) + "\n")
    (args.output / "labels.json").write_text(
        json.dumps(splits, ensure_ascii=False, indent=2), encoding="utf-8"
    )
    print(
        f"train={len(splits['train'])} valid={len(splits['valid'])} "
        f"test={len(splits['test'])} output={args.output}"
    )


if __name__ == "__main__":
    main()
