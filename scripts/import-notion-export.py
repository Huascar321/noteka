#!/usr/bin/env python3
"""Importa un export Markdown/CSV de Notion a un workspace Noteka.

El export se trata como entrada de sólo lectura. No modifica el export ni usa
red. Los adjuntos se copian al workspace mediante `attachment.import`.

Uso:
  python3 scripts/import-notion-export.py \
    --export "$HOME/Downloads/Export-..." \
    --workspace "$HOME/Library/Application Support/Noteka/workspace" \
    --cli target/debug/noteka
"""
from __future__ import annotations

import argparse
import csv
import hashlib
import html
import json
import re
import subprocess
import sys
import unicodedata
import urllib.parse
import uuid
from collections import defaultdict
from dataclasses import dataclass, field
from datetime import datetime
from pathlib import Path

BLOCK_LIMIT = 9_500
SIZE_LIMIT = 1_048_576
IMAGE_EXTS = {".png", ".jpg", ".jpeg", ".gif", ".webp"}
ATTACHMENT_EXTS = IMAGE_EXTS | {".pdf", ".xlsx", ".xls", ".docx", ".pptx", ".mp4", ".sql", ".zip"}


def norm(value: str) -> str:
    value = unicodedata.normalize("NFKC", value or "")
    return re.sub(r"\s+", " ", value).strip().casefold()


def notion_id(path: Path) -> str:
    match = re.search(r"([0-9a-f]{32})$", path.stem, re.I)
    return match.group(1) if match else ""


def title_from_markdown(text: str, path: Path) -> str:
    for line in text.splitlines():
        if line.startswith("# ") and line[2:].strip():
            return line[2:].strip()
    return re.sub(r"\s+[0-9a-f]{32}$", "", path.stem, flags=re.I).strip()


def page_db(path: Path, root: Path) -> str:
    rel = path.relative_to(root).parts
    try:
        idx = rel.index("Databases & Components")
    except ValueError:
        return ""
    return rel[idx + 1] if idx + 1 < len(rel) else ""



def row_title(row: dict[str, str], db: str) -> str:
    for key in ("Name", "Full Name", "Title", "Member", "B-Roll Description", "Content Title", "Description"):
        value = (row.get(key) or "").strip()
        if value:
            return value[:240]
    if db == "Tasks":
        return "Tarea sin título (Notion)"
    for key, value in row.items():
        if key not in {"Created", "Edited", "Created By", "Edited By", "My Day", "Shopping List", "Priority", "Status"} and value.strip():
            return value.strip()[:240]
    return f"Sin título (Notion · {db})"

def date_value(value: str) -> str | None:
    value = (value or "").strip()
    if not value:
        return None
    match = re.search(r"([A-Za-z]+)\s+(\d{1,2}),\s*(\d{4})", value)
    if not match:
        return None
    month, day, year = match.groups()
    for fmt in ("%B %d %Y", "%b %d %Y"):
        try:
            return datetime.strptime(f"{month} {day} {year}", fmt).date().isoformat()
        except ValueError:
            pass
    return None


INLINE_PATTERN = re.compile(r"(\*\*[^*]+\*\*|\*[^*]+\*|`[^`]+`|\[[^\]]+\]\([^)]+\))")


def text_node(value: str, marks: list[dict] | None = None) -> dict:
    node = {"type": "text", "text": value}
    if marks:
        node["marks"] = marks
    return node


def rich_text(value: str) -> list[dict]:
    """Conserva negrita, itálica, código y enlaces inline del Markdown."""
    nodes: list[dict] = []
    position = 0
    for match in INLINE_PATTERN.finditer(value):
        if match.start() > position:
            nodes.append(text_node(value[position:match.start()]))
        token = match.group(0)
        if token.startswith("**") and token.endswith("**"):
            nodes.append(text_node(token[2:-2], [{"type": "bold"}]))
        elif token.startswith("*") and token.endswith("*"):
            nodes.append(text_node(token[1:-1], [{"type": "italic"}]))
        elif token.startswith("`") and token.endswith("`"):
            nodes.append(text_node(token[1:-1], [{"type": "code"}]))
        else:
            link = re.fullmatch(r"\[([^\]]+)\]\(([^)]+)\)", token)
            href = link.group(2) if link else ""
            if href.startswith(("http://", "https://", "mailto:")):
                nodes.append(text_node(link.group(1), [{"type": "link", "href": href}]))
            else:
                nodes.append(text_node(link.group(1) if link else token))
        position = match.end()
    if position < len(value):
        nodes.append(text_node(value[position:]))
    return nodes or [text_node(value)]


def block(kind: str, *, content: list[dict] | None = None, children: list[dict] | None = None, attrs: dict | None = None) -> dict:
    value = {"id": str(uuid.uuid4()), "type": kind, "content": content or []}
    if children:
        value["children"] = children
    if attrs:
        value["attrs"] = attrs
    return value


def paragraph(text: str) -> dict:
    return block("paragraph", content=rich_text(text))


def list_block(kind: str, items: list[tuple[str, bool | None]]) -> dict:
    children = []
    for text, checked in items:
        attrs = {"checked": bool(checked)} if kind == "task_list" else None
        child_kind = "task_item" if kind == "task_list" else "list_item"
        children.append(block(child_kind, children=[paragraph(text)], attrs=attrs))
    return block(kind, children=children)


def strip_asides(lines: list[str]) -> list[str]:
    result: list[str] = []
    inside = False
    skip = False
    for line in lines:
        low = line.lower()
        if "<aside" in low:
            inside = True
            skip = False
            continue
        if inside and "</aside>" in low:
            inside = False
            continue
        if inside:
            if any(token in line for token in ("Page Navigation", "More Views", "**Nav**", "Action Center")):
                skip = True
            if not skip:
                result.append(line)
            continue
        result.append(line)
    return result


class MarkdownBuilder:
    def __init__(self, attachments: dict[str, str], source_dir: Path):
        self.attachments = attachments
        self.source_dir = source_dir

    def resolve_attachment(self, raw: str) -> str | None:
        raw = urllib.parse.unquote(raw.strip().strip("<>"))
        if raw.startswith(("http://", "https://", "mailto:", "#")):
            return None
        path = (self.source_dir / raw).resolve()
        return self.attachments.get(str(path))

    def image_block(self, alt: str, raw: str) -> dict | None:
        attachment = self.resolve_attachment(raw)
        if not attachment:
            return None
        return block("image", attrs={"attachment_id": attachment, "alt": alt})

    def file_block(self, name: str, raw: str) -> dict | None:
        attachment = self.resolve_attachment(raw)
        if not attachment:
            return None
        return block("file", attrs={"attachment_id": attachment, "name": name or Path(raw).name})

    def build(self, text: str) -> list[dict]:
        lines = strip_asides(text.splitlines())
        # Drop the page title; it is stored as the entity title.
        while lines and not lines[0].strip():
            lines.pop(0)
        if lines and lines[0].startswith("# "):
            lines = lines[1:]
        blocks: list[dict] = []
        paragraph_lines: list[str] = []

        def flush_paragraph() -> None:
            nonlocal paragraph_lines
            if paragraph_lines:
                value = "\n".join(paragraph_lines).strip()
                if value:
                    blocks.append(paragraph(value))
                paragraph_lines = []

        i = 0
        while i < len(lines):
            line = html.unescape(lines[i])
            line = re.sub(r"<br\s*/?>", "\n", line, flags=re.I)
            line = re.sub(r"<img\b[^>]*>", "", line, flags=re.I)
            line = re.sub(r"</?[a-zA-Z][^>]*>", "", line)
            stripped = line.strip()
            if not stripped:
                flush_paragraph()
                i += 1
                continue
            if stripped.startswith("```"):
                flush_paragraph()
                language = stripped[3:].strip() or None
                i += 1
                code: list[str] = []
                while i < len(lines) and not lines[i].strip().startswith("```"):
                    code.append(lines[i])
                    i += 1
                i += 1
                attrs = {"language": language} if language else None
                blocks.append(block("code_block", content=[text_node("\n".join(code))], attrs=attrs))
                continue
            image = re.fullmatch(r"!\[([^\]]*)\]\(([^)]+)\)", stripped)
            if image:
                flush_paragraph()
                image_block = self.image_block(image.group(1), image.group(2))
                if image_block:
                    blocks.append(image_block)
                else:
                    blocks.append(paragraph(f"Imagen: {image.group(1)}"))
                i += 1
                continue
            link = re.fullmatch(r"\[([^\]]+)\]\(([^)]+)\)", stripped)
            if link and not link.group(2).startswith(("http://", "https://", "mailto:", "#")):
                flush_paragraph()
                file_block = self.file_block(link.group(1), link.group(2))
                if file_block:
                    blocks.append(file_block)
                else:
                    blocks.append(paragraph(stripped))
                i += 1
                continue
            heading = re.match(r"^(#{1,3})\s+(.+)$", stripped)
            if heading:
                flush_paragraph()
                blocks.append(block("heading", content=rich_text(heading.group(2).strip()), attrs={"level": len(heading.group(1))}))
                i += 1
                continue
            if re.fullmatch(r"[-*_]{3,}", stripped):
                flush_paragraph()
                blocks.append(block("horizontal_rule"))
                i += 1
                continue
            if stripped.startswith(">"):
                flush_paragraph()
                quote: list[str] = []
                while i < len(lines) and lines[i].strip().startswith(">"):
                    quote.append(lines[i].strip()[1:].strip())
                    i += 1
                blocks.append(block("blockquote", children=[paragraph("\n".join(quote))]))
                continue
            if re.match(r"^[-*]\s+\[[ xX]\]\s+", stripped):
                flush_paragraph()
                items: list[tuple[str, bool | None]] = []
                while i < len(lines):
                    match = re.match(r"^[-*]\s+\[([ xX])\]\s+(.*)$", lines[i].strip())
                    if not match:
                        break
                    items.append((match.group(2), match.group(1).lower() == "x"))
                    i += 1
                blocks.append(list_block("task_list", items))
                continue
            if re.match(r"^[-*]\s+", stripped):
                flush_paragraph()
                items = []
                while i < len(lines) and re.match(r"^[-*]\s+", lines[i].strip()):
                    items.append((re.sub(r"^[-*]\s+", "", lines[i].strip()), None))
                    i += 1
                blocks.append(list_block("bullet_list", items))
                continue
            if re.match(r"^\d+[.)]\s+", stripped):
                flush_paragraph()
                items = []
                start = 1
                while i < len(lines):
                    match = re.match(r"^(\d+)[.)]\s+(.*)$", lines[i].strip())
                    if not match:
                        break
                    start = int(match.group(1))
                    items.append((match.group(2), None))
                    i += 1
                blocks.append(block("ordered_list", children=[block("list_item", children=[paragraph(text)]) for text, _ in items], attrs={"start": start}))
                continue
            if stripped.startswith("|") and stripped.endswith("|"):
                flush_paragraph()
                table: list[str] = []
                while i < len(lines) and lines[i].strip().startswith("|"):
                    table.append(lines[i].strip())
                    i += 1
                blocks.append(block("code_block", content=[text_node("\n".join(table))], attrs={"language": "markdown"}))
                continue
            paragraph_lines.append(stripped)
            i += 1
        flush_paragraph()
        return blocks[:BLOCK_LIMIT]


@dataclass
class Page:
    path: Path
    title: str
    db: str
    blocks: list[dict]
    notion_id: str
    extra_fields: dict[str, str] = field(default_factory=dict)


class Importer:
    def __init__(self, export: Path, workspace: Path, cli: Path):
        self.export = export.resolve()
        self.workspace = workspace.resolve()
        self.cli = cli.resolve()
        self.attachments: dict[str, str] = {}
        self.pages: list[Page] = []
        self.created: list[dict] = []
        self.failures: list[dict] = []
        self.notes_by_db_title: dict[tuple[str, str], Page] = {}
        self.database_rows: list[tuple[str, list[dict[str, str]]]] = []

    def call(self, operation: str, value: dict) -> dict:
        command = [str(self.cli), "--json", "--workspace", str(self.workspace), *operation.split(".")]
        result = subprocess.run(command, input=json.dumps(value, ensure_ascii=False).encode(), capture_output=True)
        try:
            reply = json.loads(result.stdout)
        except Exception as exc:
            raise RuntimeError(f"{operation}: salida inválida {result.stdout[:200]!r}: {exc}")
        if result.returncode != 0 or not reply.get("ok"):
            raise RuntimeError(f"{operation}: {reply.get('error') or result.stderr.decode()[:500]}")
        return reply["data"]

    def import_attachments(self) -> None:
        candidates = [
            path for path in self.export.rglob("*")
            if path.is_file() and path.suffix.lower() in ATTACHMENT_EXTS and not path.name.startswith(".")
        ]
        for index, path in enumerate(candidates, 1):
            try:
                attachment = self.call("attachment.import", {"source_path": str(path)})
                self.attachments[str(path.resolve())] = attachment["id"]
            except Exception as exc:
                self.failures.append({"stage": "attachment", "path": str(path), "error": str(exc)})
            if index % 25 == 0 or index == len(candidates):
                print(f"attachments {index}/{len(candidates)}", flush=True)

    def load_pages(self) -> None:
        builder_cache: dict[str, MarkdownBuilder] = {}
        files = sorted(self.export.rglob("*.md"))
        for path in files:
            text = path.read_text(encoding="utf-8-sig", errors="replace")
            title = title_from_markdown(text, path)
            db = page_db(path, self.export)
            builder = MarkdownBuilder(self.attachments, path.parent)
            blocks = builder.build(text)
            page = Page(path=path, title=title, db=db, blocks=blocks, notion_id=notion_id(path))
            self.pages.append(page)
            if db:
                self.notes_by_db_title.setdefault((db, norm(title)), page)
        # Las filas de la base Notes van primero para que la sección Notes abra
        # con el contenido que el usuario veía en Notion, no con páginas de sistema.
        self.pages.sort(key=lambda page: (page.db != "Notes", str(page.path)))
        print(f"pages {len(self.pages)}", flush=True)

    def csv_database(self, path: Path) -> str:
        name = re.sub(r"_all$", "", path.stem, flags=re.I)
        name = re.sub(r"\s+[0-9a-f]{32}$", "", name, flags=re.I)
        return name.strip()

    def canonical_csvs(self) -> list[Path]:
        result = list(self.export.rglob("Databases & Components/**/*_all.csv"))
        # Root-level exports duplicate the database exports; keep only non-duplicates.
        for path in self.export.glob("*_all.csv"):
            if "People d3dfce0430d182dfb0d8014512d331ec_all.csv" == path.name:
                result.append(path)
        return sorted(set(result))

    def add_properties(self, page: Page, row: dict[str, str]) -> None:
        values = [(key, value) for key, value in row.items() if key != "Name" and value.strip()]
        if not values:
            return
        props = [block("heading", content=[text_node("Propiedades de Notion")], attrs={"level": 3})]
        props.append(list_block("bullet_list", [(f"{key}: {value}", None) for key, value in values]))
        page.blocks = (props + page.blocks)[:BLOCK_LIMIT]
        page.extra_fields = dict(values)

    def create_note(self, title: str, blocks: list[dict], extra: dict | None = None) -> dict:
        document = {"schema_version": 2, "blocks": blocks}
        payload = {"inbox": False, **({"notion": extra} if extra else {})}
        return self.call("entity.create", {"kind": "note", "title": title, "payload": {"inbox": False}, "document": document})

    def load_database_rows(self) -> None:
        for csv_path in self.canonical_csvs():
            db = self.csv_database(csv_path)
            try:
                with csv_path.open(encoding="utf-8-sig", newline="") as handle:
                    rows = list(csv.DictReader(handle))
            except Exception as exc:
                self.failures.append({"stage": "csv", "path": str(csv_path), "error": str(exc)})
                continue
            self.database_rows.append((db, rows))
            for row in rows:
                page = self.notes_by_db_title.get((db, norm(row_title(row, db))))
                if page:
                    self.add_properties(page, row)
            print(f"database {db}: {len(rows)} rows", flush=True)

    def import_unmatched_rows(self) -> None:
        for db, rows in self.database_rows:
            for row in rows:
                title = row_title(row, db)
                if self.notes_by_db_title.get((db, norm(title))):
                    continue
                # Fila sin página Markdown: una nota conserva todas sus propiedades.
                blocks = [block("heading", content=rich_text("Propiedades de Notion"), attrs={"level": 3})]
                props = [(key, value) for key, value in row.items() if key != "Name" and value.strip()]
                if props:
                    blocks.append(list_block("bullet_list", [(f"{key}: {value}", None) for key, value in props]))
                try:
                    note = self.create_note(title, blocks, {"database": db})
                    self.created.append({"kind": "note", "title": title, "id": note["id"], "database": db})
                except Exception as exc:
                    self.failures.append({"stage": "row-note", "database": db, "title": title, "error": str(exc)})

    def create_kind_entities(self) -> None:
        for db, rows in self.database_rows:
            for row in rows:
                title = row_title(row, db)
                try:
                    if db == "Tasks":
                        status = (row.get("Status") or "").strip().lower()
                        payload = {
                            "inbox": False,
                            "state": "done" if status in {"done", "completed"} else "todo",
                            "priority": {"high": "high", "medium": "normal", "low": "low"}.get((row.get("Priority") or "").strip().lower(), "normal"),
                        }
                        due = date_value(row.get("Due", ""))
                        planned = date_value(row.get("Start", ""))
                        if due:
                            payload["due_date"] = due
                        if planned:
                            payload["planned_date"] = planned
                        kind = "task"
                    elif db == "Projects":
                        state = (row.get("Status") or "").strip().lower()
                        payload = {"state": {"planned": "planned", "doing": "doing", "ongoing": "ongoing", "on hold": "on_hold", "done": "done"}.get(state, "planned"), "purpose": ""}
                        end = date_value(row.get("Target Deadline", ""))
                        if end:
                            payload["end_date"] = end
                        kind = "project"
                    elif db == "Tags":
                        classification = (row.get("Type") or "").strip().lower()
                        payload = {"classification": {"area": "area", "resource": "resource", "topic": "topic"}.get(classification, "topic")}
                        kind = "tag"
                    elif db == "People":
                        payload = {}
                        kind = "person"
                    else:
                        continue
                    entity = self.call("entity.create", {"kind": kind, "title": title, "payload": payload})
                    self.created.append({"kind": kind, "title": title, "id": entity["id"], "database": db})
                except Exception as exc:
                    self.failures.append({"stage": "entity", "database": db, "title": title, "error": str(exc)})
            print(f"entities {db}", flush=True)

    def import_pages(self) -> None:
        for index, page in enumerate(self.pages, 1):
            try:
                note = self.create_note(page.title, page.blocks, {"notion_id": page.notion_id, "database": page.db} if (page.notion_id or page.db) else None)
                self.created.append({"kind": "note", "title": page.title, "id": note["id"], "source": str(page.path.relative_to(self.export))})
            except Exception as exc:
                self.failures.append({"stage": "page", "path": str(page.path), "error": str(exc)})
            if index % 50 == 0 or index == len(self.pages):
                print(f"pages {index}/{len(self.pages)}", flush=True)

    def import_attachment_catalog(self) -> None:
        if not self.attachments:
            return
        blocks = [block("heading", content=[text_node("Archivos del export de Notion")], attrs={"level": 2})]
        for path, attachment_id in sorted(self.attachments.items()):
            blocks.append(block("file", attrs={"attachment_id": attachment_id, "name": Path(path).name}))
        try:
            note = self.create_note("Notion · Archivos", blocks[:BLOCK_LIMIT])
            self.created.append({"kind": "note", "title": "Notion · Archivos", "id": note["id"]})
        except Exception as exc:
            self.failures.append({"stage": "attachment-catalog", "error": str(exc)})

    def run(self) -> None:
        self.import_attachments()
        self.load_pages()
        self.load_database_rows()
        self.import_pages()
        self.import_unmatched_rows()
        self.create_kind_entities()
        self.import_attachment_catalog()
        summary = {
            "source": str(self.export),
            "workspace": str(self.workspace),
            "pages": len(self.pages),
            "attachments": len(self.attachments),
            "created": len(self.created),
            "failures": len(self.failures),
            "failure_details": self.failures[:100],
        }
        manifest = self.workspace.parent / "notion-import-manifest.json"
        manifest.write_text(json.dumps({**summary, "items": self.created}, ensure_ascii=False, indent=2))
        print(json.dumps(summary, ensure_ascii=False, indent=2))


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--export", type=Path, required=True)
    parser.add_argument("--workspace", type=Path, required=True)
    parser.add_argument("--cli", type=Path, default=Path("target/debug/noteka"))
    args = parser.parse_args()
    if not args.export.is_dir():
        raise SystemExit("export inexistente")
    Importer(args.export, args.workspace, args.cli).run()


if __name__ == "__main__":
    main()
