#!/usr/bin/env python3
"""Recupera metadata explícita de notas; dry-run inicial, backup y CAS al aplicar.

Los manifiestos, imágenes, informes y datos personales permanecen fuera de Git.
No modifica documentos ni usa la red. La zona del export debe declararse.
"""
from __future__ import annotations

import argparse
import csv
import hashlib
import json
import re
import subprocess
import unicodedata
from collections import Counter, defaultdict
from datetime import datetime
from pathlib import Path
from zoneinfo import ZoneInfo


def normalized(value):
    return " ".join(unicodedata.normalize("NFKC", value).split()).casefold()


def original_id(value):
    match = re.search(r"([0-9a-f]{32})(?:\?|\.|$)", value.replace("-", ""), re.I)
    return match.group(1).lower() if match else None


def main():
    parser = argparse.ArgumentParser()
    for name in ["workspace", "import-manifest", "reference", "assets", "report"]:
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--live-metadata", type=Path)
    parser.add_argument("--repair-truncated-titles", action="store_true")
    parser.add_argument("--export-timezone", required=True)
    parser.add_argument("--cli", type=Path, default=Path("target/debug/noteka"))
    parser.add_argument("--apply", action="store_true")
    parser.add_argument("--backup", type=Path)
    args = parser.parse_args()
    if args.apply and (not args.backup or args.backup.exists()):
        parser.error("--apply exige un destino de backup nuevo")
    timezone = ZoneInfo(args.export_timezone)

    def call(operation, data):
        result = subprocess.run([str(args.cli.resolve()), "--json", "--workspace", str(args.workspace.resolve()), *operation.split(".")], input=json.dumps(data), text=True, capture_output=True)
        reply = json.loads(result.stdout)
        if result.returncode or not reply.get("ok"):
            raise RuntimeError(json.dumps(reply.get("error", {})))
        return reply["data"]

    def list_all(kind):
        result, cursor = [], None
        while True:
            page = call("entity.list", {"kind": kind, "limit": 100, **({"cursor": cursor} if cursor else {})})
            result.extend(page["items"])
            cursor = page["next_cursor"]
            if not cursor:
                return result

    def documents_digest(entities):
        return hashlib.sha256(json.dumps(sorted((e["id"], e.get("document")) for e in entities), sort_keys=True, ensure_ascii=False).encode()).hexdigest()

    entities = [entity for kind in ["note", "task", "project", "tag", "person"] for entity in list_all(kind)]
    by_id = {e["id"]: e for e in entities}
    manifest = json.loads(args.import_manifest.read_text())
    export = Path(manifest["source"])
    csv_paths = list(export.rglob("Notes*_all.csv"))
    if len(csv_paths) != 1:
        raise ValueError("No hay una única base Notes exportada")
    csv_rows = list(csv.DictReader(csv_paths[0].open(encoding="utf-8-sig", newline="")))
    rows_by_title = defaultdict(list)
    for row in csv_rows:
        rows_by_title[normalized(row["Name"])].append(row)
    source_records, source_ids = {}, {}
    for item in manifest["items"]:
        if item["kind"] != "note":
            continue
        parts = Path(item.get("source", "")).parts
        collection = item.get("database", "Pages")
        if "Databases & Components" in parts:
            index = parts.index("Databases & Components")
            collection = parts[index + 1] if len(parts) == index + 3 else "Pages"
            if collection.endswith(".md"):
                collection = "Pages"
        source_records[item["id"]] = (item, collection)
        record_id = original_id(item.get("source", ""))
        if record_id:
            source_ids.setdefault(record_id, []).append(item)

    report = {"applied": args.apply, "export_timezone": args.export_timezone, "before_document_digest": documents_digest(entities), "before_ids": sorted(by_id), "changes": [], "skipped": []}

    def save_report():
        args.report.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
        args.report.chmod(0o600)

    def resolve(href, title="", kind=None):
        candidates = source_ids.get(original_id(href), [])
        if len(candidates) == 1:
            source = candidates[0]
            collection = source_records[source["id"]][1]
            target_kind = {"Projects": "project", "Tags": "tag", "People": "person"}.get(collection, "note")
            if target_kind == "note":
                return by_id.get(source["id"])
            title, kind = source["title"], target_kind
        if kind or title:
            matches = [e for e in entities if (e["kind"] == kind if kind else e["kind"] in {"tag", "project"}) and normalized(e["title"]) == normalized(title)]
            if len(matches) == 1:
                return matches[0]
        return None

    if args.apply:
        report["backup"] = call("workspace.backup", {"destination": str(args.backup.resolve())})
        save_report()

    def update(entity, extra, reason):
        if not extra:
            return
        current = call("entity.get", {"id": entity["id"]}) if args.apply else entity
        if reason == "export-metadata":
            extra = {k: v for k, v in extra.items() if k not in current["payload"]}
        elif reason.startswith("explicit-") and current["payload"].get("icon"):
            return
        payload = {**current["payload"], **extra}
        if payload == current["payload"]:
            return
        entry = {"id": current["id"], "kind": current["kind"], "reason": reason, "previous_payload": current["payload"], "previous_revision": current["revision"], "payload": payload}
        if args.apply:
            confirmed = call("entity.update", {"id": current["id"], "expected_revision": current["revision"], "payload": payload})
            entry["revision"] = confirmed["revision"]
            by_id[current["id"]] = confirmed
        report["changes"].append(entry)
        if args.apply:
            save_report()

    note_types = {"reference", "plan", "web_clip", "voice", "journal", "meeting", "topic_research", "idea", "lecture"}

    def timestamp(value):
        if not value.strip():
            return None
        for fmt in ["%B %d, %Y %I:%M %p", "%B %d, %Y"]:
            try:
                return datetime.strptime(value, fmt).replace(tzinfo=timezone).isoformat()
            except ValueError:
                pass
        raise ValueError("Timestamp exportado no reconocido")

    for entity_id, (item, collection) in source_records.items():
        entity = by_id.get(entity_id)
        if not entity or entity["kind"] != "note":
            report["skipped"].append({"id": entity_id, "reason": "identity-changed-or-missing"})
            continue
        source = {"provider": "notion", "collection": collection, "archived": False}
        record_id = original_id(item.get("source", ""))
        if record_id:
            source["record_id"] = record_id
        extra = {}
        matches = rows_by_title[normalized(item["title"])] if collection == "Notes" else []
        if len(matches) > 1 and item.get("source"):
            # Sólo el bloque de propiedades anterior al cuerpo del Markdown.
            properties, started = {}, False
            with (export / item["source"]).open(encoding="utf-8-sig") as handle:
                for index, line in enumerate(handle):
                    if index > 80:
                        break
                    if not line.strip():
                        if started:
                            break
                        continue
                    if index == 0 and line.startswith("# "):
                        continue
                    match = re.match(r"^([^:]+):\s*(.*?)\s*$", line)
                    if not match or match.group(1) not in csv_rows[0]:
                        break
                    started = True
                    if match.group(1) in {"Created", "Updated", "Type", "Favorite", "Archived"}:
                        properties[match.group(1)] = match.group(2)
            if properties.get("Created") and properties.get("Updated"):
                matches = [row for row in matches if all(row.get(k, "").strip() == v.strip() for k, v in properties.items())]
        if collection == "Notes" and len(matches) != 1:
            source["collection"] = "Pages"
            report["skipped"].append({"id": entity_id, "reason": "not-a-unique-Notes-row"})
        if len(matches) == 1:
            row = matches[0]
            for field, key in [("Created", "created_at"), ("Updated", "updated_at")]:
                value = timestamp(row.get(field, ""))
                if value:
                    source[key] = value
            source["archived"] = row.get("Archived") == "Yes"
            note_type = row.get("Type", "").strip().lower().replace(" ", "_")
            if note_type in note_types and "note_type" not in entity["payload"]:
                extra["note_type"] = note_type
            if row.get("Favorite") in {"Yes", "No"} and "favorite" not in entity["payload"]:
                extra["favorite"] = row["Favorite"] == "Yes"
        previous_source = entity["payload"].get("source", {})
        repair = args.repair_truncated_titles and collection == "Notes" and len(matches) == 1 and len(item["title"]) > 240 and previous_source.get("provider") == "notion" and previous_source.get("collection") == "Pages" and previous_source.get("record_id") == record_id
        if "source" not in entity["payload"] or repair:
            extra["source"] = source
        update(entity, extra, "repair-source-title" if repair else "export-metadata")

    assets = json.loads(args.assets.read_text())
    for asset in assets:
        path = Path(asset["path"]).resolve(strict=True)
        if path.suffix.lower() not in {".png", ".jpg", ".jpeg", ".gif", ".webp"} or path.stat().st_size > 1_048_576:
            raise ValueError("Imagen incompatible o excesiva")
        targets = {e["id"]: e for t in asset["targets"] if (e := resolve(t["href"], t.get("title", "")))}
        targets = {key: entity for key, entity in targets.items() if not entity["payload"].get("icon")}
        attachment = call("attachment.import", {"source_path": str(path)}) if args.apply and targets else None
        for entity in targets.values():
            if entity["payload"].get("icon"):
                continue
            update(entity, {"icon": "image-" + attachment["id"].replace("-", "") if attachment else "image-" + "0" * 32}, "explicit-image")

    references = json.loads(args.reference.read_text())
    native_art = {}
    for reference in references:
        for cell in reference.get("cells", []):
            candidate = resolve("", cell.get("text", "").strip())
            if candidate and candidate["payload"].get("icon", "").startswith("image-"):
                for raw_icon in cell.get("icons", []):
                    if raw_icon.startswith("/icons/"):
                        native_art[raw_icon.split("?")[0]] = candidate["payload"]["icon"]
    for reference in references:
        entity = resolve(reference["id"].replace("-", ""))
        if not entity or entity["kind"] != "note":
            report["skipped"].append({"record_id": reference["id"], "reason": "missing-or-ambiguous-visible-note"})
            continue
        icon = (reference.get("icons") or [""])[0]
        if icon.startswith("/icons/") and not by_id[entity["id"]]["payload"].get("icon"):
            update(entity, {"icon": "key" if "/key_" in icon else "file-text", "icon_color": "#3183a6"}, "explicit-native-icon")
        for cell in reference.get("cells", []):
            if not cell.get("icons"):
                continue
            for title in cell.get("text", "").strip().split("\n"):
                if not title.strip():
                    continue
                target = resolve("", title.strip())
                kind = target["kind"] if target else None
                if not target:
                    report["skipped"].append({"id": entity["id"], "reason": "ambiguous-visible-context", "kind": kind, "title": title})
                    continue
                native = next((native_art.get(icon.split("?")[0]) for icon in cell.get("icons", []) if native_art.get(icon.split("?")[0])), None)
                if native and not target["payload"].get("icon"):
                    update(target, {"icon": native}, "explicit-native-icon")
                if args.apply:
                    current = call("entity.get", {"id": entity["id"]})
                    relation = call("relation.add", {"source_id": entity["id"], "target_id": target["id"], "kind": kind, "expected_revision": current["revision"]})
                else:
                    relation = {"changed": None}
                report["changes"].append({"id": entity["id"], "target_id": target["id"], "kind": kind, "reason": "visible-context", "result": relation})
                if args.apply:
                    save_report()
    if args.live_metadata:
        for row in json.loads(args.live_metadata.read_text()):
            entity = resolve(row["url"])
            if not entity or entity["kind"] != "note":
                raise ValueError("Metadata viva sin identidad local inequívoca")
            current = call("entity.get", {"id": entity["id"]}) if args.apply else entity
            source = current["payload"].get("source")
            if not source or source.get("provider") != "notion" or source.get("record_id") != original_id(row["url"]):
                raise ValueError("La procedencia no coincide con la metadata actual")
            patch = dict(source)
            for remote, local in [("Updated", "updated_at"), ("Created", "created_at")]:
                if row.get(remote):
                    patch[local] = datetime.fromisoformat(row[remote].replace(" ", "T").replace("Z", "+00:00")).isoformat()
            update(current, {"source": patch}, "live-source-metadata")
    if args.apply:
        after = [e for kind in ["note", "task", "project", "tag", "person"] for e in list_all(kind)]
        report["after_document_digest"] = documents_digest(after)
        report["after_ids"] = sorted(e["id"] for e in after)
        report["documents_and_ids_unchanged"] = report["before_document_digest"] == report["after_document_digest"] and report["before_ids"] == report["after_ids"]
        save_report()
        if not report["documents_and_ids_unchanged"]:
            raise RuntimeError("El digest de originales cambió; revisar el informe y cambios concurrentes")
    save_report()
    print(json.dumps({"applied": args.apply, "changes": dict(Counter(c["reason"] for c in report["changes"])), "skipped": len(report["skipped"]), "documents_and_ids_unchanged": report.get("documents_and_ids_unchanged")}))


if __name__ == "__main__":
    main()
