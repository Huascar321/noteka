#!/usr/bin/env python3
"""Calidad real separada de validez de protocolo; golden declarado antes de OCR."""
import argparse, importlib.util, json, os, sys
from pathlib import Path
sys.dont_write_bytecode=True
ap=argparse.ArgumentParser();ap.add_argument('--corpus',type=Path,required=True);ap.add_argument('--composed',type=Path,required=True);ap.add_argument('--output',type=Path,required=True);a=ap.parse_args()
base=Path(os.environ['NOTEKA_TEST_TMP']).resolve(strict=True)
assert all(p.resolve().is_relative_to(base) for p in [a.corpus,a.composed,a.output]) and not a.output.exists()
spec=importlib.util.spec_from_file_location('q',Path(__file__).with_name('test-ocr-reader.py'));q=importlib.util.module_from_spec(spec);spec.loader.exec_module(q)
manifest=json.loads((a.corpus/'manifest.json').read_text());cases={c['name']:c for c in manifest['cases']};rows=json.loads(a.composed.read_text());assert len(rows)==len(cases)
chars=words=cd=wd=0;quality=[];limitations=[]
for row in rows:
    if 'document' not in row:assert row['error']==cases[row['name']]['error'];continue
    c=cases[row['name']];actual_pages=['\n'.join(r['text'] for r in p['recognition']['regions']) for p in row['document']['pages']]
    if c.get('quality'):
        expected='\n'.join('\n'.join(p) for p in c['expected']);actual='\n'.join(actual_pages)
        d=q.distance(expected,actual);w=q.distance(expected.split(),actual.split());chars+=len(expected);words+=len(expected.split());cd+=d;wd+=w
        quality.append(dict(name=row['name'],expected=expected,actual=actual,char_distance=d,word_distance=w))
    else:limitations.append(dict(name=row['name'],actual_pages=actual_pages))
report=dict(characters=chars,words=words,char_distance=cd,word_distance=wd,cer=cd/chars,wer=wd/words,normalization='none; exact Unicode and line feeds',scope=manifest['quality_profile'],cases=quality,non_quality_cases=limitations)
a.output.write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
assert report['cer']<=manifest['max_aggregate_cer'],{k:v for k,v in report.items() if k not in ['cases','non_quality_cases']}
print(json.dumps({k:v for k,v in report.items() if k not in ['cases','non_quality_cases']}))
