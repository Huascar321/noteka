#!/usr/bin/env python3
"""Exercise export recovery against an isolated real SQLite workspace."""
import csv
import json
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CLI = ROOT / "target/debug/noteka"
RECOVER = ROOT / "scripts/recover-notion-note-metadata.py"


def run(args, data=None, check=True):
    return subprocess.run(args, input=json.dumps(data) if data is not None else None,
                          text=True, capture_output=True, check=check)


with tempfile.TemporaryDirectory(prefix="noteka-voice-recovery-") as temp:
    root = Path(temp).resolve()
    workspace, export = root / "workspace", root / "export"
    export.mkdir()

    def cli(operation, data):
        result = run([str(CLI), "--json", "--workspace", str(workspace), *operation.split(".")], data)
        response = json.loads(result.stdout)
        assert response["ok"], response
        return response["data"]

    cli("workspace.create", {"timezone": "America/La_Paz"})
    rows = [("Voice missing", " Voice  Note ", {}),
            ("Voice existing", "Voice Note", {"note_type": "reference"}),
            ("Daily", "Daily", {}), ("Book", "Book", {}),
            ("Recipe", "Recipe", {}), ("Outlier", "Outlier Content", {}),
            ("Creator", "Creator Tips", {}), ("Tools", "Tools", {}),
            ("Unknown", "Unrecognized", {}),
            ("Ambiguous", "Voice Note", {})]
    items, ids = [], {}
    for index, (title, typ, payload) in enumerate(rows, 1):
        note = cli("entity.create", {"kind": "note", "title": title, "payload": payload})
        ids[title] = note["id"]
        source = f"note-{index:032x}.md"
        (export / source).write_text("# Synthetic\n\nBody\n", encoding="utf-8")
        items.append({"id": note["id"], "kind": "note", "title": title,
                      "database": "Notes", "source": source})
    with (export / "Notes_all.csv").open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=["Name", "Type", "Archived"])
        writer.writeheader()
        for title, typ, _ in rows:
            writer.writerow({"Name": title, "Type": typ, "Archived": "No"})
        writer.writerow({"Name": "Ambiguous", "Type": "Voice Note", "Archived": "No"})
    manifest, reference, assets, report = (root / name for name in ("manifest.json", "reference.json", "assets.json", "report.json"))
    manifest.write_text(json.dumps({"source": str(export), "items": items}), encoding="utf-8")
    reference.write_text("[]", encoding="utf-8")
    assets.write_text("[]", encoding="utf-8")
    base = ["python3", str(RECOVER), "--workspace", str(workspace), "--import-manifest", str(manifest),
            "--reference", str(reference), "--assets", str(assets), "--report", str(report),
            "--export-timezone", "America/La_Paz", "--cli", str(CLI)]
    before = {name: cli("entity.get", {"id": entity_id}) for name, entity_id in ids.items()}
    run(base)
    proposed = json.loads(report.read_text())
    changes = {entry["id"]: entry for entry in proposed["changes"] if entry["reason"] == "export-metadata"}
    assert changes[ids["Voice missing"]]["payload"]["note_type"] == "voice"
    assert changes[ids["Voice existing"]]["payload"]["note_type"] == "reference"
    assert "note_type" not in changes[ids["Unknown"]]["payload"]
    assert any(item["id"] == ids["Ambiguous"] and item["reason"] == "not-a-unique-Notes-row" for item in proposed["skipped"])
    assert all(cli("entity.get", {"id": entity_id}) == before[name] for name, entity_id in ids.items())
    assert run(base + ["--apply"], check=False).returncode != 0
    backup = root / "backup"
    run(base + ["--apply", "--backup", str(backup)])
    applied = json.loads(report.read_text())
    assert applied["documents_and_ids_unchanged"] and backup.exists()
    after = {name: cli("entity.get", {"id": entity_id}) for name, entity_id in ids.items()}
    assert after["Voice missing"]["payload"]["note_type"] == "voice"
    assert after["Voice existing"]["payload"]["note_type"] == "reference"
    for title, typ in [("Daily", "daily"), ("Book", "book"), ("Recipe", "recipe"),
                       ("Outlier", "outlier_content"), ("Creator", "creator_tips"), ("Tools", "tools")]:
        assert after[title]["payload"]["note_type"] == typ
    assert "note_type" not in after["Unknown"]["payload"] and "note_type" not in after["Ambiguous"]["payload"]
    assert all(after[name]["document"] == before[name]["document"] for name in ids)
    assert all(after[name]["revision"] == before[name]["revision"] + 1 for name in ids)
    voices = cli("home.query", {"view": "notes_voice", "limit": 100})
    assert ids["Voice missing"] in [item["entity"]["id"] for item in voices["items"]]
    run(base)
    assert not json.loads(report.read_text())["changes"]
    print("Voice export recovery: dry-run, backup, CAS, ambiguity and Home passed")
