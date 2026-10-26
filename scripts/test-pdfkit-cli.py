#!/usr/bin/env python3
"""Corpus nativo por cola/CLI real y lectura independiente de la representación."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import sqlite3
import subprocess

ROOT = Path(__file__).resolve().parent.parent


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--profile', choices=['debug', 'release'], required=True)
    ap.add_argument('--corpus', type=Path, required=True)
    ap.add_argument('--searchable', action='store_true', help='Verificar el índice nuevo; sin flag comprueba el backend14 histórico.')
    ap.add_argument('--output', type=Path, required=True)
    args = ap.parse_args()
    base = Path(os.environ['NOTEKA_TEST_TMP']).resolve(strict=True)
    out, corpus = args.output.resolve(), args.corpus.resolve(strict=True)
    assert out.is_relative_to(base) and corpus.is_relative_to(base) and not out.exists()
    out.mkdir(); runtime = out/'bin'; runtime.mkdir()
    target = ROOT/'target'/args.profile
    for name in ['noteka', 'noteka-pdfkit-worker']:
        shutil.copy2(target/name, runtime/name)
    ws = out/'workspace'
    calls = []

    def call(op, payload, error=None):
        command = [str(runtime/'noteka'), '--workspace', str(ws), '--json', *[s.replace('_', '-') for s in op.split('.')]]
        p = subprocess.run(['/usr/bin/sandbox-exec', '-p', '(version 1)(allow default)(deny network*)', *command], input=json.dumps(payload).encode(), capture_output=True, timeout=12)
        assert not p.stderr, p.stderr[:400]
        r = json.loads(p.stdout); calls.append(op)
        if error:
            assert not r['ok'] and p.returncode and r['error']['details']['reason'] == error, r
            return r['error']
        assert r['ok'] and not p.returncode, r
        return r['data']

    call('workspace.create', dict(timezone='UTC'))
    ctl = call('extraction.control', dict(action='get'))
    call('extraction.control', dict(action='set', paused=False, expected_revision=ctl['revision']))
    report = dict(profile=args.profile, network_denied=True, cases=[], no_helper_reads=0)
    for case in json.loads((corpus/'manifest.json').read_text())['cases']:
        original = corpus/case['name']
        assert hashlib.sha256(original.read_bytes()).hexdigest() == case['sha256']
        a = call('attachment.import', dict(source_path=str(original)))
        if a['bytes'] > 1048576:
            call('extraction.enqueue', dict(attachment_id=a['id'], extractor='pdfkit_text'), 'source_size_limit')
            report['cases'].append(dict(name=case['name'], source_cap=True)); continue
        j = call('extraction.enqueue', dict(attachment_id=a['id'], extractor='pdfkit_text'))['job']
        # Encolar mantiene el contador en cero; no genera páginas.
        assert j['attempts'] == 0 and j['generation'] is None
        if 'error' in case:
            call('extraction.run', dict(id=j['id']), case['error'])
            j = call('extraction.get', dict(id=j['id']))
            assert j['state'] == 'failed' and j['generation'] is None
        else:
            j = call('extraction.run', dict(id=j['id']))['job']
            assert j['state'] == 'succeeded'
            with sqlite3.connect(ws/'noteka.sqlite') as c:
                chunks = c.execute('SELECT text FROM extraction_pdfkit_chunks WHERE generation_id=? ORDER BY ordinal', [j['generation']['id']]).fetchall()
                raw = ''.join(row[0] for row in chunks).encode()
            canonical = json.loads(raw)
            assert [p['text'] for p in canonical['pages']] == case['pages']
            assert raw == json.dumps(canonical, ensure_ascii=False, separators=(',', ':')).encode()
            items, cursor = [], None
            # El lector funciona incluso cuando desaparece el helper de su runtime.
            helper = runtime/'noteka-pdfkit-worker'; absent = runtime/'helper.absent'; helper.rename(absent)
            try:
                while True:
                    p = call('extraction.pdfkit_pages', dict(id=j['id'], limit=17, cursor=cursor))
                    assert p['manifest']['representation_sha256'] == hashlib.sha256(raw).hexdigest()
                    assert p['manifest']['representation_bytes'] == len(raw)
                    items.extend(p['items']); cursor = p['next_cursor']; report['no_helper_reads'] += 1
                    if cursor is None: break
                assert items == canonical['pages']
                call('extraction.ocr_regions', dict(id=j['id']), 'unsupported_generation_config')
                call('extraction.pdf_fragments', dict(id=j['id']), 'unsupported_generation_config')
                idx = call('extraction.index', dict(action='get'))
                if args.searchable:
                    call('extraction.index', dict(action='put', job_id=j['id'], expected_generation_id=j['generation']['id'], expected_revision=idx['revision']))
                    # Oracle del corpus anterior al parser: sólo texto físico no vacío.
                    with sqlite3.connect(ws/'noteka.sqlite') as c:
                        postings = c.execute('SELECT text FROM extraction_search_fts WHERE job_id=? ORDER BY ordinal', [j['id']]).fetchall()
                    assert [r[0] for r in postings] == [text for text in case['pages'] if text]
                    for text in case['pages']:
                        if not text:
                            continue
                        query = text.split()[0]
                        hits = call('extraction.search', dict(query=query, job_id=j['id'], extractor='pdfkit_text', limit=64))
                        assert hits['items'], hits
                        for hit in hits['items']:
                            locator = hit['locator']; page = canonical['pages'][locator['page_number']-1]
                            assert locator['kind'] == 'pdfkit_page_text'
                            assert locator['engine'] == canonical['engine']
                            assert locator['profile'] == p['manifest']['profile']
                            assert locator['limitations'] == canonical['limitations']
                            assert locator['page_bytes'] == len(page['text'].encode())
                            assert hit['text'] == page['text'].encode()[locator['text_byte_start']:locator['text_byte_end']].decode()
                    assert not call('extraction.search', dict(query='PDFKit', job_id=j['id'], extractor='pdfkit_text', limit=64))['items']
                else:
                    call('extraction.index', dict(action='put', job_id=j['id'], expected_generation_id=j['generation']['id'], expected_revision=idx['revision']), 'unsupported_index_extractor')
            finally:
                absent.rename(helper)
        assert hashlib.sha256((ws/'attachments'/a['id']).read_bytes()).hexdigest() == case['sha256']
        report['cases'].append(dict(name=case['name'], passed=True))
    report['cli_calls'] = len(calls)
    report['passed'] = True
    (out/'results.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps(dict(profile=args.profile, cases=len(report['cases']), no_helper_reads=report['no_helper_reads'], cli_calls=len(calls), passed=True)))


if __name__ == '__main__':
    main()
