#!/usr/bin/env python3
"""Recupera tags y relaciones acreditados; no modifica cuerpos ni usa la red."""
from __future__ import annotations

import argparse
import csv
import hashlib
import json
import re
import subprocess
import unicodedata
from collections import Counter, defaultdict
from datetime import datetime, timezone
from pathlib import Path


def normalized(value):
    return " ".join(unicodedata.normalize("NFKC", value).split()).casefold()


def original_id(value):
    match = re.search(r"([0-9a-f]{32})(?:\?|\.|$)", value.replace("-", ""), re.I)
    return match[1].lower() if match else None


def properties(path):
    result, started = {}, False
    with path.open(encoding="utf-8-sig") as handle:
        for index, line in enumerate(handle):
            if index == 0:
                continue
            if index > 100:
                break
            if not line.strip():
                if started:
                    break
                continue
            match = re.match(r"^([^:]{1,80}):\s*(.*?)\s*$", line)
            if not match:
                break
            started = True
            if match[1] in {"Type", "Archived", "Favorite", "Notes", "Latest Activity"}:
                result[match[1]] = match[2]
    return result


def main():
    parser = argparse.ArgumentParser()
    for name in ["workspace", "import-manifest", "assets", "report"]:
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--cli", type=Path, default=Path("target/debug/noteka"))
    parser.add_argument("--apply", action="store_true")
    parser.add_argument("--backup", type=Path)
    args = parser.parse_args()
    if args.apply and (not args.backup or args.backup.exists()):
        parser.error("--apply exige un destino nuevo de backup")

    def call(operation, data):
        result = subprocess.run([str(args.cli.resolve()), "--json", "--workspace", str(args.workspace.resolve()), *operation.split(".")], input=json.dumps(data), text=True, capture_output=True)
        reply = json.loads(result.stdout)
        if result.returncode or not reply.get("ok"):
            raise RuntimeError(json.dumps(reply.get("error", {})))
        return reply["data"]

    def list_all(kind):
        entities, cursor = [], None
        while True:
            page = call("entity.list", {"kind": kind, "limit": 100, **({"cursor": cursor} if cursor else {})})
            entities.extend(page["items"])
            cursor = page["next_cursor"]
            if not cursor:
                return entities

    def all_entities():
        return [e for kind in ["note", "task", "project", "tag", "person"] for e in list_all(kind)]

    def digest(entities):
        return hashlib.sha256(json.dumps(sorted((e["id"], e.get("document")) for e in entities), sort_keys=True, ensure_ascii=False).encode()).hexdigest()

    entities = all_entities()
    by_id = {e["id"]: e for e in entities}
    manifest = json.loads(args.import_manifest.read_text())
    export = Path(manifest["source"])
    csv_files = list(export.rglob("Tags*_all.csv"))
    if len(csv_files) != 1:
        raise ValueError("Export Tags ambiguo")
    rows = list(csv.DictReader(csv_files[0].open(encoding="utf-8-sig", newline="")))
    items = [i for i in manifest["items"] if i["kind"] == "tag"]
    if len(items) != len(rows) or any(normalized(i["title"]) != normalized(r["Name"][:240]) for i, r in zip(items, rows)):
        raise ValueError("La secuencia CSV/manifiesto no coincide")
    source_pages = [(i, properties(export / i["source"])) for i in manifest["items"] if i["kind"] == "note" and Path(i.get("source", "")).parent.name == "Tags"]
    notes = defaultdict(list)
    for item in manifest["items"]:
        if item["kind"] == "note" and original_id(item.get("source", "")) and item["id"] in by_id:
            notes[original_id(item["source"])].append(by_id[item["id"]])
    report = {"applied": args.apply, "before_ids": sorted(by_id), "before_document_digest": digest(entities), "csv_sha256": hashlib.sha256(csv_files[0].read_bytes()).hexdigest(), "sequence_verified": len(rows), "changes": [], "skipped": [], "classification_differences": []}

    def save():
        args.report.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
        args.report.chmod(0o600)

    bindings, tags = [], {}
    for item, row in zip(items, rows):
        matches = [(page, fields) for page, fields in source_pages if normalized(page["title"]) == normalized(item["title"]) and all(fields.get(k) == row[k] for k in ["Type", "Favorite", "Archived"])]
        entity = by_id.get(item["id"])
        if len(matches) != 1 or not entity or entity["kind"] != "tag":
            raise ValueError("Tag sin identidad inequívoca; no se aplicó recuperación")
        page, fields = matches[0]
        oid = original_id(page["source"])
        if not oid or oid in tags:
            raise ValueError("ID original ausente o duplicado")
        tags[oid] = entity
        bindings.append((entity, oid, row, fields))

    if args.apply:
        report["backup"] = call("workspace.backup", {"destination": str(args.backup.resolve())})
        save()

    def update(entity, extra, reason):
        current = call("entity.get", {"id": entity["id"]}) if args.apply else by_id[entity["id"]]
        extra = {k: v for k, v in extra.items() if k not in current["payload"]}
        payload = {**current["payload"], **extra}
        if payload == current["payload"]:
            return
        entry = {"id": current["id"], "reason": reason, "previous_payload": current["payload"], "previous_revision": current["revision"], "payload": payload}
        if args.apply:
            confirmed = call("entity.update", {"id": current["id"], "expected_revision": current["revision"], "payload": payload})
            entry["revision"] = confirmed["revision"]
            by_id[current["id"]] = confirmed
        else:
            by_id[current["id"]] = {**current, "payload": payload}
        report["changes"].append(entry)
        if args.apply:
            save()

    for entity, oid, row, fields in bindings:
        extra = {"source": {"provider": "notion", "collection": "Tags", "record_id": oid, "archived": row["Archived"] == "Yes"}, "favorite": row["Favorite"] == "Yes"}
        if fields.get("Latest Activity"):
            extra["source_activity_at"] = datetime.strptime(fields["Latest Activity"], "%B %d, %Y %I:%M %p (UTC)").replace(tzinfo=timezone.utc).isoformat()
        update(entity, extra, "tag-provenance")
        source_type = row["Type"].lower()
        if source_type != entity["payload"].get("classification"):
            report["classification_differences"].append({"id": entity["id"], "source_type": source_type, "local_type": entity["payload"].get("classification")})
        for note_id in sorted(set(re.findall(r"([a-f0-9]{32})\.md", fields.get("Notes", "")))):
            candidates = notes.get(note_id, [])
            if len(candidates) != 1 or candidates[0]["kind"] != "note":
                report["skipped"].append({"reason": "ambiguous-or-missing-note", "original_id": note_id, "tag_id": entity["id"]})
                continue
            note = candidates[0]
            entry = {"reason": "source-note-relation", "source_id": note["id"], "target_id": entity["id"], "kind": "tag"}
            if args.apply:
                current = call("entity.get", {"id": note["id"]})
                entry["previous_revision"] = current["revision"]
                entry["result"] = call("relation.add", {"source_id": note["id"], "target_id": entity["id"], "kind": "tag", "expected_revision": current["revision"]})
            report["changes"].append(entry)
            if args.apply:
                save()

    for asset in json.loads(args.assets.read_text()):
        path = Path(asset["path"]).resolve(strict=True)
        if path.suffix.lower() != ".png" or path.stat().st_size > 1_048_576:
            raise ValueError("Arte incompatible o excesivo")
        targets = {}
        for target in asset["targets"]:
            oid = original_id(target.get("record_id", target.get("url", "")))
            entity = tags.get(oid) if target["kind"] == "tag" else notes.get(oid, [None])[0] if len(notes.get(oid, [])) == 1 else None
            if entity and not by_id[entity["id"]]["payload"].get("icon"):
                targets[entity["id"]] = entity
        if args.apply and targets:
            attachment = call("attachment.import", {"source_path": str(path)})
            for entity in targets.values():
                update(entity, {"icon": "image-" + attachment["id"].replace("-", "")}, "explicit-source-icon")
        elif targets:
            report["changes"].append({"reason": "explicit-source-icon", "path": str(path), "ids": sorted(targets)})

    if args.apply:
        after = all_entities()
        report["after_ids"] = sorted(e["id"] for e in after)
        report["after_document_digest"] = digest(after)
        report["documents_and_ids_unchanged"] = report["before_ids"] == report["after_ids"] and report["before_document_digest"] == report["after_document_digest"]
        save()
        if not report["documents_and_ids_unchanged"]:
            raise RuntimeError("Cambió el digest de documentos/IDs; revisar concurrencia e informe")
    save()
    print(json.dumps({"applied": args.apply, "sequence_verified": report["sequence_verified"], "changes": dict(Counter(c["reason"] for c in report["changes"])), "skipped": len(report["skipped"]), "classification_differences": len(report["classification_differences"]), "documents_and_ids_unchanged": report.get("documents_and_ids_unchanged")}))


if __name__ == "__main__":
    main()
