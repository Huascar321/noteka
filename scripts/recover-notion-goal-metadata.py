#!/usr/bin/env python3
"""Recover typed Goal/Milestone metadata through public operations, never bodies."""
import argparse
import collections
import datetime as dt
import hashlib
import json
from pathlib import Path
import re
import subprocess
from zoneinfo import ZoneInfo


def original_id(value):
    matches = re.findall(r"[0-9a-fA-F]{32}|[0-9a-fA-F]{8}(?:-[0-9a-fA-F]{4}){3}-[0-9a-fA-F]{12}", value or "")
    return matches[-1].replace("-", "").lower() if matches else None


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--workspace", required=True, type=Path)
    parser.add_argument("--source", required=True, type=Path)
    parser.add_argument("--activity", required=True, type=Path)
    parser.add_argument("--report", required=True, type=Path)
    parser.add_argument("--backup", type=Path)
    parser.add_argument("--binary", type=Path, default=Path(__file__).resolve().parents[1] / "target/debug/noteka")
    parser.add_argument("--apply", action="store_true")
    args = parser.parse_args()
    if args.report.exists():
        raise ValueError("El informe ya existe; elegí otro nombre para conservar evidencia anterior")
    if args.apply and not args.backup:
        raise ValueError("--apply requiere un destino --backup nuevo")
    data = json.loads(args.source.read_text())
    activity = json.loads(args.activity.read_text())
    goals = data["goals"]["results"]
    milestones = data["milestones"]["results"]
    report = {"applied": args.apply, "changes": [], "skipped": [], "source_sha256": hashlib.sha256(args.source.read_bytes()).hexdigest(), "activity_sha256": hashlib.sha256(args.activity.read_bytes()).hexdigest()}

    def save():
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(json.dumps(report, ensure_ascii=False, indent=2))

    def call(operation, value):
        result = subprocess.run([str(args.binary), "--json", "--workspace", str(args.workspace), *operation.split(".")], input=json.dumps(value), text=True, capture_output=True)
        try:
            response = json.loads(result.stdout)
        except ValueError as error:
            raise RuntimeError(f"Respuesta CLI no interpretable para {operation}") from error
        if result.returncode or not response.get("ok"):
            raise RuntimeError(json.dumps(response.get("error"), ensure_ascii=False))
        return response["data"]

    def all_entities():
        items = []
        for archived in [False, True]:
            cursor = None
            while True:
                page = call("entity.list", {"archived": archived, "cursor": cursor, "limit": 200})
                items.extend(page["items"])
                cursor = page.get("next_cursor")
                if not cursor:
                    break
        return items

    def digest(items):
        h = hashlib.sha256()
        for entity in sorted(items, key=lambda e: e["id"]):
            h.update(entity["id"].encode())
            h.update(json.dumps(entity.get("document"), sort_keys=True, ensure_ascii=False, separators=(",", ":")).encode())
        return h.hexdigest()

    before = all_entities()
    by_id = {e["id"]: e for e in before}
    report["before_ids"] = sorted(by_id)
    report["before_document_digest"] = digest(before)
    identities = collections.defaultdict(list)
    for entity in before:
        source = entity["payload"].get("source", {})
        oid = original_id(source.get("record_id", ""))
        if source.get("provider") == "notion" and oid:
            identities[(entity["kind"], source.get("collection"), oid)].append(entity)

    def resolve(url, kind, collection):
        found = identities[(kind, collection, original_id(url))]
        if len(found) != 1:
            raise ValueError(f"Identidad original no inequívoca: {kind}/{collection}/{original_id(url)} ({len(found)})")
        return found[0]

    def urls(value):
        result = json.loads(value or "[]")
        if not isinstance(result, list) or not all(isinstance(v, str) for v in result):
            raise ValueError("Relación de origen no interpretable")
        return result

    def date(row, name):
        value = row.get(f"date:{name}:start")
        if value is None:
            return None
        if row.get(f"date:{name}:is_datetime", 0) or row.get(f"date:{name}:end"):
            raise ValueError(f"Fecha {name} con instante/rango fuera del recorte civil")
        if dt.date.fromisoformat(value).isoformat() != value:
            raise ValueError("Fecha no canónica")
        return value

    activity_by_id = {}
    for row in activity["values"]:
        attributes = dict(row["attributes"])
        oid = original_id(attributes.get("href"))
        shown = row["text"].splitlines()[-1]
        parsed = dt.datetime.strptime(shown, "%B %d, %Y %I:%M %p").replace(tzinfo=ZoneInfo(activity["timezone"]))
        if not oid or oid in activity_by_id:
            raise ValueError("Actividad sin identidad inequívoca")
        activity_by_id[oid] = parsed.isoformat()
    goal_entities = {original_id(row["url"]): resolve(row["url"], "note", "Goals") for row in goals}
    if len(goal_entities) != len(goals) or len(activity_by_id) != len(goals) or set(activity_by_id) != set(goal_entities):
        raise ValueError("Conjunto de Goals/actividad incompatible")
    goal_fields = {}
    for row in goals:
        entity = goal_entities[original_id(row["url"])]
        state = {"Active": "active", "Dream": "dream", "Achieved": "achieved"}.get(row["Status"])
        if not state:
            raise ValueError("Estado Goal no reconocido")
        goal_fields[entity["id"]] = {k: v for k, v in {"state": state, "goal_set": date(row, "Goal Set"), "target_deadline": date(row, "Target Deadline"), "achieved_on": date(row, "Achieved")}.items() if v is not None}
        for url in urls(row.get("Projects")):
            resolve(url, "project", "Projects")
        if len(urls(row.get("Tag"))) > 1:
            raise ValueError("Goal con más de un tag en origen")
        for url in urls(row.get("Tag")):
            resolve(url, "tag", "Tags")
    milestone_fields = {}
    for row in milestones:
        entity = resolve(row["url"], "note", "Milestones")
        links = urls(row.get("Goal"))
        if len(links) > 1:
            raise ValueError("Milestone con varios Goals")
        target = goal_entities.get(original_id(links[0])) if links else None
        if links and target is None:
            raise ValueError("Milestone apunta a un Goal no incluido")
        milestone_fields[entity["id"]] = {k: v for k, v in {"goal_id": target["id"] if target else None, "target_deadline": date(row, "Target Deadline"), "completed_on": date(row, "Date Completed")}.items() if v is not None}
    for row in goals:
        expected = {resolve(url, "note", "Milestones")["id"] for url in urls(row.get("Milestones"))}
        goal = goal_entities[original_id(row["url"])]["id"]
        actual = {mid for mid, fields in milestone_fields.items() if fields.get("goal_id") == goal}
        if expected != actual:
            raise ValueError("Los extremos Goal↔Milestone no coinciden")
    report["identity_proof"] = {"goals": len(goal_entities), "milestones": len(milestone_fields), "linked_milestones": sum("goal_id" in f for f in milestone_fields.values()), "activity_precision": "minute", "timezone": activity["timezone"], "template_promoted": False}
    save()
    if args.apply:
        report["backup"] = call("workspace.backup", {"destination": str(args.backup)})
        save()

    newly_typed_goals = set()

    def update(entity_id, patch, reason):
        current = call("entity.get", {"id": entity_id}) if args.apply else by_id[entity_id]
        entry = {"id": entity_id, "reason": reason, "expected_revision": current["revision"], "fields": patch}
        if args.apply:
            result = call("entity.update", {"id": entity_id, "expected_revision": current["revision"], "payload": patch})
            entry["revision"] = result["revision"]
            by_id[entity_id] = result
        report["changes"].append(entry)
        save()

    for row in goals:
        entity = goal_entities[original_id(row["url"])]
        if entity["payload"].get("goal") is not None:
            report["skipped"].append({"id": entity["id"], "reason": "goal-already-typed-preserve-local"})
            continue
        source = {**entity["payload"]["source"], "updated_at": row["Updated"]}
        update(entity["id"], {"inbox": False, "goal": goal_fields[entity["id"]], "source": source}, "goal-subtype")
        newly_typed_goals.add(entity["id"])
    for mid, fields in milestone_fields.items():
        if by_id[mid]["payload"].get("milestone") is not None:
            report["skipped"].append({"id": mid, "reason": "milestone-already-typed-preserve-local"})
            continue
        update(mid, {"inbox": False, "milestone": fields}, "milestone-subtype")

    relation_cache = {}

    def relations(entity_id):
        if entity_id not in relation_cache:
            items, cursor = [], None
            while True:
                page = call("relation.list", {"entity_id": entity_id, "limit": 200, "cursor": cursor})
                items.extend(page["items"])
                cursor = page.get("next_cursor")
                if not cursor:
                    break
            relation_cache[entity_id] = items
        return relation_cache[entity_id]

    for row in goals:
        goal = goal_entities[original_id(row["url"])]["id"]
        for field, kind, collection in [("Projects", "project", "Projects"), ("Tag", "tag", "Tags")]:
            for url in urls(row.get(field)):
                target = resolve(url, kind, collection)["id"]
                links = [r for r in relations(goal) if r["source_id"] == goal and r["kind"] == kind]
                if any(r["target_id"] == target for r in links):
                    continue
                if kind == "tag" and links:
                    report["skipped"].append({"id": goal, "reason": "preserve-existing-tag", "source_tag": target})
                    continue
                entry = {"reason": "goal-relation", "source_id": goal, "target_id": target, "kind": kind}
                if args.apply:
                    current = call("entity.get", {"id": goal})
                    entry["result"] = call("relation.add", {"source_id": goal, "target_id": target, "kind": kind, "expected_revision": current["revision"]})
                    relation_cache.pop(goal, None)
                report["changes"].append(entry)
                save()
    if args.apply:
        current = {e["id"]: e for e in all_entities()}
        for row in goals:
            goal = goal_entities[original_id(row["url"])]["id"]
            if goal not in newly_typed_goals:
                continue
            source_ids = {goal}
            source_ids.update(e["id"] for e in current.values() if e["payload"].get("milestone", {}).get("goal_id") == goal)
            for link in relations(goal):
                if link["kind"] != "project" or link["source_id"] != goal:
                    continue
                project = link["target_id"]
                source_ids.add(project)
                for edge in relations(project):
                    candidate = current.get(edge["source_id"])
                    if edge["kind"] == "project" and edge["target_id"] == project and candidate and candidate["kind"] in {"note", "task"} and candidate["payload"].get("goal") is None and candidate["payload"].get("milestone") is None:
                        source_ids.add(candidate["id"])
            revisions = {eid: current[eid]["revision"] for eid in source_ids}
            fresh = call("entity.get", {"id": goal})
            revisions[goal] = fresh["revision"] + 1
            snapshot = {"at": activity_by_id[original_id(row["url"])], "revisions": revisions}
            update(goal, {"goal": {**fresh["payload"]["goal"], "activity_snapshot": snapshot}}, "goal-activity-baseline")
        after = all_entities()
        report["after_ids"] = sorted(e["id"] for e in after)
        report["after_document_digest"] = digest(after)
        report["documents_and_ids_unchanged"] = report["before_ids"] == report["after_ids"] and report["before_document_digest"] == report["after_document_digest"]
        if not report["documents_and_ids_unchanged"]:
            save()
            raise RuntimeError("Cambió el digest de IDs/documentos; revisar informe y concurrencia")
    save()
    print(json.dumps({"applied": args.apply, "identity_proof": report["identity_proof"], "changes": dict(collections.Counter(c["reason"] for c in report["changes"])), "skipped": len(report["skipped"]), "documents_and_ids_unchanged": report.get("documents_and_ids_unchanged")}, ensure_ascii=False))


if __name__ == "__main__":
    main()
