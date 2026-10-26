#!/usr/bin/env python3
"""Asocia un manifiesto explícito de imágenes a proyectos locales.

El modo inicial es dry-run. --apply exige un backup canónico nuevo. El
manifiesto y las imágenes son datos del usuario y deben permanecer fuera de Git.
"""
from __future__ import annotations

import argparse
import json
import subprocess
import unicodedata
from pathlib import Path


def normalized(value: str) -> str:
    return " ".join(unicodedata.normalize("NFKC", value).split()).casefold()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--workspace", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--cli", type=Path, default=Path("target/debug/noteka"))
    parser.add_argument("--apply", action="store_true")
    parser.add_argument("--backup", type=Path)
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    if args.apply and (not args.backup or args.backup.exists()):
        parser.error("--apply exige --backup con un destino todavía inexistente")

    def call(operation: str, data: dict) -> dict:
        command = [str(args.cli.resolve()), "--json", "--workspace", str(args.workspace.resolve()), *operation.split(".")]
        result = subprocess.run(command, input=json.dumps(data), text=True, capture_output=True, check=False)
        response = json.loads(result.stdout)
        if result.returncode or not response.get("ok"):
            raise RuntimeError(json.dumps(response.get("error", {}), ensure_ascii=False))
        return response["data"]

    projects = []
    cursor = None
    while True:
        page = call("entity.list", {"kind": "project", "limit": 100, **({"cursor": cursor} if cursor else {})})
        projects.extend(page["items"])
        cursor = page["next_cursor"]
        if not cursor:
            break
    manifest = json.loads(args.manifest.read_text())
    selected, skipped = [], []
    used = set()
    for asset in manifest:
        path = Path(asset["path"]).resolve(strict=True)
        if not path.is_file() or path.suffix.lower() not in {".png", ".jpg", ".jpeg", ".gif", ".webp"} or path.stat().st_size > 1_048_576:
            raise ValueError("El arte debe ser una imagen local compatible de hasta 1 MiB")
        for title in asset["titles"]:
            matches = [p for p in projects if normalized(p["title"]) == normalized(title)]
            if len(matches) != 1 or matches[0]["payload"].get("icon"):
                skipped.append({"title": title, "reason": "ambiguous/missing/custom-icon"})
                continue
            entity = matches[0]
            if entity["id"] in used:
                raise ValueError("El manifiesto asigna dos imágenes al mismo proyecto")
            used.add(entity["id"])
            selected.append({"entity": entity, "path": str(path)})
    report = {"workspace": str(args.workspace.resolve()), "applied": False, "selected": [], "skipped": skipped}
    if args.apply and selected:
        report["backup"] = call("workspace.backup", {"destination": str(args.backup.resolve())})
    attachments = {}
    for item in selected:
        entity = item["entity"]
        record = {"id": entity["id"], "title": entity["title"], "previous_revision": entity["revision"], "previous_payload": entity["payload"], "source": item["path"]}
        if args.apply:
            if item["path"] not in attachments:
                attachments[item["path"]] = call("attachment.import", {"source_path": item["path"]})
            attachment = attachments[item["path"]]
            updated = call("entity.update", {"id": entity["id"], "expected_revision": entity["revision"], "payload": {**entity["payload"], "icon": "image-" + attachment["id"].replace("-", "")}})
            record.update(attachment=attachment, revision=updated["revision"])
        report["selected"].append(record)
        args.report.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
        args.report.chmod(0o600)
    report["applied"] = args.apply
    args.report.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    args.report.chmod(0o600)
    print(json.dumps({"applied": args.apply, "selected": len(selected), "skipped": len(skipped)}, ensure_ascii=False))


if __name__ == "__main__":
    main()
