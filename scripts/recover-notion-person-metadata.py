#!/usr/bin/env python3
"""Recuperación explícita de contactos; fuentes y datos personales fuera de Git.

Dry-run por defecto. Apply exige backup nuevo, usa CAS y registro incremental.
Nunca envía document en una mutación ni infiere fotos o relaciones por título.
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
from pathlib import Path


def normalized(value):
    return " ".join(unicodedata.normalize("NFKC", value).split()).casefold()


def original_id(value):
    match = re.search(r"([0-9a-f]{32})(?:\?|\.|$)", value.replace("-", ""), re.I)
    return match[1].lower() if match else None


def main():
    parser = argparse.ArgumentParser()
    for name in ["workspace", "import-manifest", "source", "report"]:
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--cli", type=Path, default=Path("target/debug/noteka"))
    parser.add_argument("--apply", action="store_true")
    parser.add_argument("--backup", type=Path)
    args = parser.parse_args()
    if args.apply and (not args.backup or args.backup.exists()):
        parser.error("--apply exige un destino de backup nuevo")

    def call(operation, data):
        result = subprocess.run([str(args.cli.resolve()), "--json", "--workspace", str(args.workspace.resolve()), *operation.split(".")], input=json.dumps(data), text=True, capture_output=True)
        reply = json.loads(result.stdout)
        if result.returncode or not reply.get("ok"):
            raise RuntimeError(json.dumps(reply.get("error", {})))
        return reply["data"]

    def list_all():
        entities = []
        for kind in ["note", "task", "project", "tag", "person"]:
            cursor = None
            while True:
                page = call("entity.list", {"kind": kind, "limit": 100, **({"cursor": cursor} if cursor else {})})
                entities.extend(page["items"])
                cursor = page["next_cursor"]
                if not cursor:
                    break
        return entities

    def digest(entities):
        return hashlib.sha256(json.dumps(sorted((e["id"], e.get("document")) for e in entities), sort_keys=True, ensure_ascii=False).encode()).hexdigest()

    entities = list_all()
    by_id = {e["id"]: e for e in entities}
    manifest = json.loads(args.import_manifest.read_text())
    export = Path(manifest["source"])
    rows = json.loads((args.source / "contacts.json").read_text())["results"]
    gallery = json.loads((args.source / "gallery.json").read_text())
    assets = json.loads((args.source / "assets.json").read_text())
    csvs = []
    for path in export.rglob("People*_all.csv"):
        with path.open(encoding="utf-8-sig", newline="") as handle:
            reader = csv.DictReader(handle)
            fields = reader.fieldnames or []
            if "Full Name" in fields or "Membership Type" in fields:
                csvs.append((path, list(reader), "contacts" if "Full Name" in fields else "members"))
    if sorted(kind for _, _, kind in csvs) != ["contacts", "members"]:
        raise ValueError("Las dos colecciones People no están acreditadas")
    source_csv, csv_rows, _ = next(item for item in csvs if item[2] == "contacts")
    member_csv, member_rows, _ = next(item for item in csvs if item[2] == "members")
    report = {"applied": args.apply, "before_ids": sorted(by_id), "before_document_digest": digest(entities), "identity_proof": {}, "changes": [], "skipped": []}

    def save():
        args.report.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
        args.report.chmod(0o600)

    people = [i for i in manifest["items"] if i["kind"] == "person"]
    identities = {}
    for row in rows:
        oid = original_id(row["url"])
        title = normalized(row["Full Name"])
        original_rows = [r for r in csv_rows if normalized(r["Full Name"]) == title]
        typed = [i for i in people if normalized(i["title"]) == title]
        pages = [i for i in manifest["items"] if i["kind"] == "note" and original_id(i.get("source", "")) == oid and normalized(i["title"]) == title]
        if len(original_rows) != 1 or len(typed) != 1 or len(pages) != 1 or typed[0]["id"] not in by_id:
            raise ValueError("Identidad de contacto ambigua o ausente; no se aplicó recuperación")
        entity = by_id[typed[0]["id"]]
        if entity["kind"] != "person" or oid in identities:
            raise ValueError("Identidad original incompatible")
        identities[oid] = entity
    member_entities = []
    contact_ids = {e["id"] for e in identities.values()}
    for row in member_rows:
        matches = [i for i in people if normalized(i["title"]) == normalized(row["Name"]) and i["id"] not in contact_ids]
        if len(matches) != 1 or matches[0]["id"] not in by_id:
            raise ValueError("Identidad de miembro ambigua; no se aplicó recuperación")
        member_entities.append(by_id[matches[0]["id"]])
    if len(contact_ids) + len({e["id"] for e in member_entities}) != len(people):
        raise ValueError("Personas fuera de las dos colecciones acreditadas")
    report["identity_proof"] = {"contacts": len(identities), "members": len(member_entities), "contact_csv_sha256": hashlib.sha256(source_csv.read_bytes()).hexdigest(), "member_csv_sha256": hashlib.sha256(member_csv.read_bytes()).hexdigest(), "contact_original_page_ids_verified": True}
    project_ids = {original_id(e["payload"].get("source", {}).get("record_id", "")): e for e in entities if e["kind"] == "project" and e["payload"].get("source", {}).get("record_id")}
    order = {original_id(card["id"]): index for index, card in enumerate(gallery)}
    # Saved Contacts contains all eleven records; the gallery initially shows ten.
    for index, row in enumerate(rows):
        order.setdefault(original_id(row["url"]), index)
    for asset in assets:
        path = Path(asset["path"]).resolve(strict=True)
        if original_id(asset["record_id"]) not in identities or path.suffix.lower() not in {".png", ".jpg", ".jpeg", ".webp"} or path.stat().st_size > 1_048_576:
            raise ValueError("Foto no acreditada o incompatible")
    if args.apply:
        report["backup"] = call("workspace.backup", {"destination": str(args.backup.resolve())})
        save()

    def update(entity, extra, reason):
        current = call("entity.get", {"id": entity["id"]}) if args.apply else by_id[entity["id"]]
        payload = {**current["payload"], **{k: v for k, v in extra.items() if k not in current["payload"]}}
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

    for row in rows:
        oid = original_id(row["url"])
        extra = {"source": {"provider": "notion", "collection": "People", "record_id": oid, "created_at": row["Created"], "updated_at": row["Edited"], "archived": False}, "gallery_order": order[oid], "relationships": json.loads(row.get("Relationship") or "[]"), "pipeline_status": row["Pipeline Status"].lower()}
        if row.get("Company"):
            extra["company"] = row["Company"]
        for remote, local in [("Birthday", "birthday"), ("Check-In", "check_in"), ("Last Check-In", "last_check_in")]:
            if row.get("date:" + remote + ":start"):
                extra[local] = row["date:" + remote + ":start"]
        update(identities[oid], extra, "contact-metadata")
    for entity in member_entities:
        update(entity, {"source": {"provider": "notion", "collection": "WorkspacePeople", "archived": False}}, "workspace-member-provenance")

    seen_links = set()
    for row in rows:
        person = identities[original_id(row["url"])]
        for url in json.loads(row.get("Projects") or "[]"):
            project = project_ids.get(original_id(url))
            if not project:
                report["skipped"].append({"reason": "project-unavailable-in-export", "record_id": original_id(url)})
                continue
            key = (project["id"], person["id"])
            if key in seen_links:
                continue
            seen_links.add(key)
            entry = {"reason": "person-project-relation", "source_id": project["id"], "target_id": person["id"], "kind": "person"}
            if args.apply:
                current = call("entity.get", {"id": project["id"]})
                entry["previous_revision"] = current["revision"]
                entry["result"] = call("relation.add", {"source_id": project["id"], "target_id": person["id"], "kind": "person", "expected_revision": current["revision"]})
            report["changes"].append(entry)
            if args.apply:
                save()

    for asset in assets:
        entity = identities[original_id(asset["record_id"])]
        if by_id[entity["id"]]["payload"].get("photo"):
            continue
        if args.apply:
            attachment = call("attachment.import", {"source_path": str(Path(asset["path"]).resolve())})
            report["changes"].append({"reason": "source-photo-import", "attachment_id": attachment["id"], "entity_id": entity["id"]})
            save()
            update(entity, {"photo": attachment["id"], "photo_position": asset.get("photo_position", 50)}, "explicit-source-photo")
        else:
            report["changes"].append({"reason": "explicit-source-photo", "id": entity["id"], "path": asset["path"]})
    if args.apply:
        after = list_all()
        report["after_ids"] = sorted(e["id"] for e in after)
        report["after_document_digest"] = digest(after)
        report["documents_and_ids_unchanged"] = report["before_ids"] == report["after_ids"] and report["before_document_digest"] == report["after_document_digest"]
        save()
        if not report["documents_and_ids_unchanged"]:
            raise RuntimeError("Cambió digest de documentos/IDs; inspeccionar concurrencia e informe")
    save()
    print(json.dumps({"applied": args.apply, "identity_proof": report["identity_proof"], "changes": dict(Counter(c["reason"] for c in report["changes"])), "skipped": len(report["skipped"]), "documents_and_ids_unchanged": report.get("documents_and_ids_unchanged")}))


if __name__ == "__main__":
    main()
