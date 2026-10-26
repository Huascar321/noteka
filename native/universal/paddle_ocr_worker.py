#!/usr/bin/env python3
"""Adaptador universal PDFium + PaddleOCR para Noteka.

Entrada JSON por stdin: source_b64, source_sha256, remaining_ms.
Salida JSON: parser/version, hash y páginas de texto. No red, no workspace.
Los modelos se indican con NOTEKA_PADDLE_MODEL_PATHS; no se descargan aquí.
"""
import base64
import hashlib
import json
import os
import sys
import tempfile
from html.parser import HTMLParser
from pathlib import Path

MAX_PAGES = 20
MAX_TEXT = 1_048_576
MAX_STRUCTURE = 1_048_576
MAX_TABLES = 512
MAX_CELLS = 4096


class TableParser(HTMLParser):
    def __init__(self):
        super().__init__()
        self.row = -1
        self.column = 0
        self.cells = []
        self.occupied = set()
        self.current = None

    def handle_starttag(self, tag, attrs):
        if tag == "tr":
            self.row += 1
            self.column = 0
        if tag in ("td", "th"):
            while (self.row, self.column) in self.occupied:
                self.column += 1
            values = dict(attrs)
            row_span = int(values.get("rowspan", 1))
            col_span = int(values.get("colspan", 1))
            if row_span < 1 or col_span < 1 or row_span > 64 or col_span > 64:
                raise ValueError("table_span")
            self.current = {
                "row": self.row,
                "column": self.column,
                "row_span": row_span,
                "col_span": col_span,
                "text": "",
            }
            for r in range(self.row, self.row + row_span):
                for c in range(self.column, self.column + col_span):
                    self.occupied.add((r, c))
            self.column += col_span
        if tag == "br" and self.current:
            self.current["text"] += "\n"

    def handle_data(self, data):
        if self.current is not None:
            self.current["text"] += data

    def handle_endtag(self, tag):
        if tag in ("td", "th") and self.current is not None:
            if len(self.current["text"]) > 512:
                raise ValueError("table_cell_text_limit")
            self.cells.append(self.current)
            self.current = None

    def result(self):
        if self.current is not None:
            raise ValueError("table_cell_unclosed")
        rows = max((r + 1 for r, _ in self.occupied), default=0)
        columns = max((c + 1 for _, c in self.occupied), default=0)
        if len(self.cells) > MAX_CELLS:
            raise ValueError("table_cell_limit")
        return {"rows": rows, "columns": columns, "cells": self.cells}


def table(html):
    parser = TableParser()
    parser.feed(html)
    parser.close()
    return parser.result()


def main():
    request = json.load(sys.stdin)
    raw = base64.b64decode(request["source_b64"], validate=True)
    if hashlib.sha256(raw).hexdigest() != request["source_sha256"]:
        raise ValueError("hash")
    paths = json.loads(Path(os.environ["NOTEKA_PADDLE_MODEL_PATHS"]).read_text())
    from paddleocr import PPStructureV3

    prefixes = {
        "layout_detection": "PP-DocLayout_plus-L",
        "region_detection": "PP-DocBlockLayout",
        "doc_orientation_classify": "PP-LCNet_x1_0_doc_ori",
        "text_detection": "PP-OCRv6_small_det",
        "text_recognition": "PP-OCRv6_small_rec",
        "textline_orientation": "PP-LCNet_x1_0_textline_ori",
        "table_classification": "PP-LCNet_x1_0_table_cls",
        "wired_table_structure_recognition": "SLANeXt_wired",
        "wireless_table_structure_recognition": "SLANet_plus",
        "wired_table_cells_detection": "RT-DETR-L_wired_table_cell_det",
        "wireless_table_cells_detection": "RT-DETR-L_wireless_table_cell_det",
        "table_orientation_classify": "PP-LCNet_x1_0_doc_ori",
    }
    kwargs = {}
    for prefix, name in prefixes.items():
        kwargs[prefix + "_model_name"] = name
        kwargs[prefix + "_model_dir"] = paths[name]
    kwargs.update(
        device=os.environ.get("NOTEKA_PADDLE_DEVICE", "cpu"),
        cpu_threads=4,
        enable_hpi=False,
        use_doc_orientation_classify=True,
        use_doc_unwarping=False,
        use_textline_orientation=True,
        use_table_recognition=True,
        use_region_detection=True,
        use_seal_recognition=False,
        use_formula_recognition=False,
        use_chart_recognition=False,
        markdown_ignore_labels=[],
    )
    with tempfile.TemporaryDirectory(prefix="noteka-paddle-") as directory:
        source = Path(directory) / "source.pdf"
        source.write_bytes(raw)
        engine = PPStructureV3(**kwargs)
        pages = []
        structure_pages = []
        total = 0
        table_count = 0
        try:
            results = engine.predict(str(source))
            if len(results) > MAX_PAGES:
                raise ValueError("page_limit")
            for index, result in enumerate(results):
                body = result.json["res"]
                blocks = []
                for block in body.get("parsing_res_list", []):
                    content = block.get("block_content") or ""
                    if content:
                        blocks.append(content)
                text = "\n".join(blocks)
                total += len(text.encode("utf-8"))
                if total > MAX_TEXT:
                    raise ValueError("text_limit")
                pages.append({"page_number": index + 1, "text": text})
                tables = []
                for value in body.get("table_res_list", []):
                    parsed = table(value.get("pred_html", ""))
                    table_count += 1
                    if table_count > MAX_TABLES:
                        raise ValueError("table_limit")
                    tables.append(parsed)
                structure_pages.append({"page_number": index + 1, "tables": tables})
        finally:
            engine.close()
    if len(json.dumps({"version": 1, "pages": structure_pages}, ensure_ascii=False)) > MAX_STRUCTURE:
        raise ValueError("structure_limit")
    json.dump(
        {
            "parser_name": "paddleocr_ppstructurev3",
            "parser_version": "3.7.0",
            "source_sha256": request["source_sha256"],
            "pages": pages,
            "structure": {"version": 1, "pages": structure_pages},
        },
        sys.stdout,
        ensure_ascii=False,
    )


if __name__ == "__main__":
    main()
