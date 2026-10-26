#!/usr/bin/env python3
"""Fixture propia MIT: PDF textual grande, golden previo; no usa PDFKit."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[3]
spec = importlib.util.spec_from_file_location('pdf', ROOT/'tests/fixtures/pdf/generate.py')
pdf = importlib.util.module_from_spec(spec);spec.loader.exec_module(pdf)
base = Path(os.environ['NOTEKA_TEST_TMP']).resolve(strict=True)
out = Path(sys.argv[1]).resolve()
assert out.is_relative_to(base) and not out.exists();out.mkdir()
pages = [];streams = []
for page in range(1,51):
    lines = [f'notekapdfkit evidencia contrato page{page:03} line{line:03} datos locales texto literal verificable' for line in range(1,101)]
    pages.append('\n'.join(lines))
    streams.append(b'BT /F1 4 Tf 7 TL 10 780 Td '+b' T* '.join(b'('+line.encode()+b') Tj' for line in lines)+b' ET')
raw = pdf.serialize(pdf.document(streams,compressed=True));assert len(raw)<=1_048_576
(out/'pages.pdf').write_bytes(raw)
manifest=dict(license='MIT',expected_before_reader=True,generator_uses_pdfkit=False,pages=pages,sha256=hashlib.sha256(raw).hexdigest())
(out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps(dict(source_bytes=len(raw),expected_text_bytes=sum(len(p.encode()) for p in pages),pages=len(pages))))
