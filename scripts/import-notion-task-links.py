#!/usr/bin/env python3
"""Recupera sólo vínculos tarea/proyecto explícitos de un manifiesto autorizado.

No adivina duplicados ni modifica títulos, fechas, estados o documentos.
El modo inicial es dry-run; --apply exige un backup canónico nuevo.
"""
from __future__ import annotations

import argparse
import json
import subprocess
import unicodedata
from datetime import datetime
from pathlib import Path


def norm(value: str) -> str:
    return " ".join(unicodedata.normalize("NFKC", value).split()).casefold()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--workspace", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--report", type=Path, required=True)
    parser.add_argument("--backup", type=Path)
    parser.add_argument("--apply", action="store_true")
    parser.add_argument("--cli", type=Path, default=Path("target/debug/noteka"))
    args = parser.parse_args()
    if args.apply and (not args.backup or args.backup.exists()):
        parser.error("--apply exige un destino de backup nuevo")

    def call(operation: str, data: dict) -> dict:
        r = subprocess.run([str(args.cli.resolve()), "--json", "--workspace", str(args.workspace.resolve()), *operation.split(".")], input=json.dumps(data), capture_output=True, text=True, check=False)
        response = json.loads(r.stdout)
        if r.returncode or not response.get("ok"):
            raise RuntimeError(json.dumps(response.get("error", {}), ensure_ascii=False))
        return response["data"]

    def entities(kind: str) -> list:
        rows, cursor = [], None
        while True:
            page = call("entity.list", {"kind": kind, "limit": 100, **({"cursor": cursor} if cursor else {})})
            rows.extend(page["items"])
            cursor = page["next_cursor"]
            if not cursor:
                return rows

    tasks, projects = entities("task"), entities("project")
    selected, skipped = [], []
    seen = set()
    for item in json.loads(args.manifest.read_text()):
        due = datetime.strptime(item["date"], "%B %d, %Y").date().isoformat() if item.get("date") else None
        matches = [row for row in tasks if norm(row["title"]) == norm(item["title"]) and row["payload"].get("due_date") == due]
        if item.get("task_id"):
            matches = [row for row in matches if row["id"] == item["task_id"]]
        if len(matches) != 1:
            skipped.append({"title": item["title"], "reason": "ambiguous/missing-task", "candidates": [row["id"] for row in matches]})
            continue
        for title in item["projects"]:
            targets = [p for p in projects if norm(p["title"]) == norm(title)]
            if len(targets) != 1:
                skipped.append({"title": title, "reason": "ambiguous/missing-project"})
                continue
            pair = (matches[0]["id"], targets[0]["id"])
            if pair not in seen:
                seen.add(pair)
                selected.append({"task_id": pair[0], "project_id": pair[1], "source": item})
    report = {"workspace": str(args.workspace.resolve()), "applied": False, "selected": selected, "skipped": skipped, "changes": []}
    if args.apply and selected:
        report["backup"] = call("workspace.backup", {"destination": str(args.backup.resolve())})
        for item in selected:
            current = call("entity.get", {"id": item["task_id"]})
            response = call("relation.add", {"source_id": item["task_id"], "target_id": item["project_id"], "kind": "project", "expected_revision": current["revision"]})
            report["changes"].append({"task_id": item["task_id"], "previous_revision": current["revision"], "relation": response["relation"], "changed": response["changed"]})
            args.report.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
            args.report.chmod(0o600)
    report["applied"] = args.apply
    args.report.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    args.report.chmod(0o600)
    print(json.dumps({"applied": args.apply, "selected": len(selected), "skipped": len(skipped)}, ensure_ascii=False))


if __name__ == "__main__":
    main()
