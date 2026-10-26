#!/usr/bin/env python3
"""Recupera metadata acreditada y relaciones, sin modificar documentos ni estados.

Dry-run predeterminado. Al aplicar exige backup nuevo, usa CAS e informe privado.
Las fuentes, imágenes, IDs personales e informes se proporcionan fuera de Git.
"""
from __future__ import annotations

import argparse
import csv
import hashlib
import json
import re
import subprocess
import unicodedata
from collections import Counter
from datetime import datetime
from pathlib import Path
from zoneinfo import ZoneInfo


def normalized(value):
    return " ".join(unicodedata.normalize("NFKC", value).split()).casefold()


def original_id(value):
    match = re.search(r"([0-9a-f]{32})(?:\?|\.|$)", value.replace("-", ""), re.I)
    return match[1].lower() if match else None


def main():
    parser = argparse.ArgumentParser()
    for name in ["workspace", "import-manifest", "source", "assets", "report"]:
        parser.add_argument("--" + name, type=Path, required=True)
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
    report = {"applied": args.apply, "before_ids": sorted(by_id), "before_document_digest": digest(entities), "identity_proof": {}, "changes": [], "skipped": [], "state_differences": []}

    def save():
        args.report.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
        args.report.chmod(0o600)

    # Se verifica toda la secuencia CSV/manifiesto, luego se acredita el ID
    # original con título y Created/Edited/Status del bloque de propiedades.
    # Nunca se lee más allá de la primera separación anterior al cuerpo.
    identities = {}
    for collection, kind in [("Tasks", "task"), ("Projects", "project")]:
        csv_paths = list(export.rglob(collection + "*_all.csv"))
        if len(csv_paths) != 1:
            raise ValueError("Colección exportada ambigua")
        rows = list(csv.DictReader(csv_paths[0].open(encoding="utf-8-sig", newline="")))
        items = [i for i in manifest["items"] if i["kind"] == kind]
        fallback = "Tarea sin título (Notion)" if kind == "task" else "Proyecto sin título (Notion)"
        if len(rows) != len(items) or any(normalized(i["title"]) != normalized(r["Name"][:240] or fallback) for i, r in zip(items, rows)):
            raise ValueError("La secuencia completa no coincide con el importador original")
        pages = []
        for item in manifest["items"]:
            if item["kind"] != "note" or Path(item.get("source", "")).parent.name != collection:
                continue
            properties, started = {}, False
            with (export / item["source"]).open(encoding="utf-8-sig") as handle:
                for index, line in enumerate(handle):
                    if index > 100:
                        break
                    if index == 0:
                        continue
                    if not line.strip():
                        if started:
                            break
                        continue
                    match = re.match(r"^([^:]{1,80}):\s*(.*?)\s*$", line)
                    if not match:
                        break
                    started = True
                    if match[1] in {"Created", "Edited", "Status"}:
                        properties[match[1]] = match[2]
            pages.append((item, properties))
        resolved = 0
        for item, row in zip(items, rows):
            title = normalized(row["Name"])
            matches = [p for p, fields in pages if all(normalized(fields.get(k, "")) == normalized(row.get(k, "")) for k in ["Created", "Edited", "Status"]) and (not title or normalized(p["title"])[:100] == title[:100])]
            if len(matches) != 1 or not original_id(matches[0]["source"]):
                report["skipped"].append({"entity_id": item["id"], "reason": "ambiguous-original-identity", "candidates": len(matches)})
                continue
            entity = by_id.get(item["id"])
            if not entity or entity["kind"] != kind:
                report["skipped"].append({"entity_id": item["id"], "reason": "missing-or-changed-entity-kind"})
                continue
            oid = original_id(matches[0]["source"])
            if oid in identities:
                raise ValueError("ID original duplicado")
            identities[oid] = entity
            resolved += 1
        report["identity_proof"][collection] = {"csv_sha256": hashlib.sha256(csv_paths[0].read_bytes()).hexdigest(), "sequence_verified": len(rows), "resolved_original_ids": resolved}

    # Notes importadas ya contienen el ID original: tags tipados se resuelven
    # por su página original y su único título en el manifiesto, no por arte.
    for item in manifest["items"]:
        if item["kind"] != "note" or not original_id(item.get("source", "")):
            continue
        collection = Path(item["source"]).parent.name
        if collection == "Goals" and item["id"] in by_id:
            identities[original_id(item["source"])] = by_id[item["id"]]
        if collection == "Tags":
            matches = [i for i in manifest["items"] if i["kind"] == "tag" and normalized(i["title"]) == normalized(item["title"])]
            if len(matches) == 1 and matches[0]["id"] in by_id:
                identities[original_id(item["source"])] = by_id[matches[0]["id"]]

    def resolve(value):
        return identities.get(original_id(value))

    projects = json.loads((args.source / "projects.json").read_text())["results"]
    board = json.loads((args.source / "board-reference.json").read_text())
    board_order = {original_id(r["id"]): r["order"] for r in board}
    required_ids = {original_id(r["url"]) for r in projects}
    required_ids.update(original_id(url) for r in projects for url in json.loads(r.get("Tasks") or "[]"))
    unresolved = sorted(oid for oid in required_ids if oid not in identities)
    report["unresolved_required_ids"] = unresolved
    if any(original_id(row["url"]) not in identities for row in projects):
        save()
        raise ValueError("Hay proyectos sin identidad; no se aplicó recuperación")

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

    def timestamp(value):
        if not value:
            return None
        try:
            return datetime.fromisoformat(value.replace("Z", "+00:00")).isoformat()
        except ValueError:
            return datetime.strptime(value, "%B %d, %Y %I:%M %p").replace(tzinfo=timezone).isoformat()

    seen_links = set()

    def relate(source, target, kind):
        key = (source["id"], target["id"], kind)
        if key in seen_links:
            return
        seen_links.add(key)
        entry = {"reason": "source-relation", "source_id": source["id"], "target_id": target["id"], "kind": kind}
        if args.apply:
            current = call("entity.get", {"id": source["id"]})
            entry["previous_revision"] = current["revision"]
            entry["result"] = call("relation.add", {"source_id": source["id"], "target_id": target["id"], "kind": kind, "expected_revision": current["revision"]})
        report["changes"].append(entry)
        if args.apply:
            save()

    for row in projects:
        entity = resolve(row["url"])
        source = {"provider": "notion", "collection": "Projects", "record_id": original_id(row["url"]), "archived": row["Archived"] in {"Yes", "__YES__", True, 1}}
        for remote, local in [("Created", "created_at"), ("Edited", "updated_at")]:
            if row.get(remote):
                source[local] = timestamp(row[remote])
        extra = {"source": source}
        if source["record_id"] in board_order:
            extra["board_order"] = board_order[source["record_id"]]
        update(entity, extra, "project-provenance")
        expected_state = row["Status"].lower().replace(" ", "_")
        if entity["payload"].get("state", "planned") != expected_state:
            report["state_differences"].append({"id": entity["id"], "source_state": expected_state, "local_state": entity["payload"].get("state")})
        for url in json.loads(row.get("Tasks") or "[]"):
            task = resolve(url)
            if task:
                relate(task, entity, "project")
            else:
                report["skipped"].append({"reason": "source-task-unavailable-in-export", "project_id": entity["id"], "original_id": original_id(url)})
        for field, kind in [("Tag", "tag"), ("Goal", "project")]:
            for url in json.loads(row.get(field) or "[]"):
                context = resolve(url)
                if not context:
                    report["skipped"].append({"reason": "unresolved-context", "field": field, "original_id": original_id(url)})
                    continue
                if field == "Tag" and context["kind"] == "tag":
                    relate(entity, context, kind)
                elif field == "Goal" and context["kind"] == "note" and context["payload"].get("source", {}).get("collection") == "Goals":
                    relate(context, entity, kind)
                else:
                    raise ValueError("Tipo de contexto incompatible")

    for asset in json.loads(args.assets.read_text()):
        path = Path(asset["path"]).resolve(strict=True)
        if path.suffix.lower() != ".png" or path.stat().st_size > 1_048_576:
            raise ValueError("Arte incompatible o excesivo")
        targets = []
        for target in asset["targets"]:
            if target["kind"] == "project":
                entity = resolve(target["record_id"])
            else:
                matches = [e for e in identities.values() if e["kind"] in {"tag", "note"} and normalized(e["title"]) == normalized(target["title"])]
                matches = {e["id"]: e for e in matches}
                entity = next(iter(matches.values())) if len(matches) == 1 else None
            if entity and entity["id"] not in {e["id"] for e in targets}:
                targets.append(entity)
        targets = [e for e in targets if not by_id[e["id"]]["payload"].get("icon")]
        if args.apply and targets:
            attachment = call("attachment.import", {"source_path": str(path)})
            for entity in targets:
                update(entity, {"icon": "image-" + attachment["id"].replace("-", "")}, "explicit-source-icon")
        elif targets:
            report["changes"].append({"reason": "explicit-source-icon", "path": str(path), "ids": [e["id"] for e in targets]})

    if args.apply:
        after = all_entities()
        report["after_ids"] = sorted(e["id"] for e in after)
        report["after_document_digest"] = digest(after)
        report["documents_and_ids_unchanged"] = report["before_ids"] == report["after_ids"] and report["before_document_digest"] == report["after_document_digest"]
        save()
        if not report["documents_and_ids_unchanged"]:
            raise RuntimeError("Cambió el digest de documentos/IDs; revisar concurrencia e informe")
    save()
    print(json.dumps({"applied": args.apply, "identity_proof": report["identity_proof"], "changes": dict(Counter(c["reason"] for c in report["changes"])), "skipped": len(report["skipped"]), "state_differences": len(report["state_differences"]), "documents_and_ids_unchanged": report.get("documents_and_ids_unchanged")}))


if __name__ == "__main__":
    main()
