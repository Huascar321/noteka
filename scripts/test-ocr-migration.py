#!/usr/bin/env python3
"""Migrar un baseline v12 real independiente; nunca construye v12 con SQL actual."""
import argparse
import ast
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import sqlite3
import subprocess
import sys
sys.dont_write_bytecode = True
ROOT=Path(__file__).resolve().parent.parent

def main():
    ap=argparse.ArgumentParser();ap.add_argument('--fixture',type=Path,required=True);ap.add_argument('--old',type=Path,required=True);ap.add_argument('--profile',choices=['debug','release'],required=True);ap.add_argument('--output',type=Path,required=True);args=ap.parse_args()
    base=Path(os.environ['NOTEKA_TEST_TMP']).resolve(strict=True);directory=args.fixture.resolve(strict=True);out=args.output.resolve()
    assert directory.is_relative_to(base) and out.is_relative_to(base) and not out.exists();out.mkdir()
    spec=importlib.util.spec_from_file_location('prior_migration_helpers',ROOT/'scripts/test-xlsx-search-migration.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
    old_source=args.old.resolve(strict=True);old_runtime=out/'old-client';old_runtime.mkdir()
    for name in ['noteka','noteka-mcp']:shutil.copy2(old_source.with_name(name),old_runtime/name)
    old=old_runtime/'noteka';source=ROOT/'target'/args.profile/'noteka';runtime=out/'new-client';runtime.mkdir()
    for name in ['noteka','noteka-mcp']:shutil.copy2(source.with_name(name),runtime/name)
    new=runtime/'noteka';ws=directory/'workspace';data=json.loads((directory/'baseline.json').read_text())
    with sqlite3.connect(ws/'noteka.sqlite') as db:assert db.execute('PRAGMA user_version').fetchone()[0]==12
    assert m.snapshot(ws)==data['tables']
    before=m.files(ws)
    m.call(new,ws,'extraction','index',dict(action='get'),'unsupported_workspace_version')
    m.call(new,ws,'extraction','search',dict(query='evidencia'),'unsupported_workspace_version')
    assert m.files(ws)==before
    assert m.call(new,ws,'workspace','open',{})['schema_version']==13
    after=m.snapshot(ws);assert set(after)-set(data['tables'])=={'extraction_ocr_chunks','extraction_ocr_generations'}
    for name,rows in data['tables'].items():
        if name!='workspace_meta':assert after[name]==rows,name
    normalized=[tuple(12 if i==1 else value for i,value in enumerate(row)) for row in ast.literal_eval(after['workspace_meta'])];assert repr(normalized)==data['tables']['workspace_meta']
    old_schema={tuple(row[:2]):row for row in data['schema']}
    now_schema={tuple(row[:2]):list(row) for row in m.schema(ws)}
    assert all(now_schema[key]==value for key,value in old_schema.items())
    assert m.files(ws/'attachments')==data['originals']
    assert m.call(new,ws,'extraction','index',dict(action='get',limit=1))==data['index']
    assert m.call(new,ws,'extraction','search',dict(query='evidencia',limit=1))==data['query']
    m.call(new,ws,'extraction','index',dict(action='get',limit=1,cursor=data['index']['next_cursor']))
    m.call(new,ws,'extraction','search',dict(query='evidencia',limit=1,cursor=data['query']['next_cursor']))
    rejected=m.call(new,ws,'changes','apply',dict(plan=data['plan'],expected=data['receipt']),'revision_conflict');assert rejected['details']['reason']=='base_hash_mismatch'
    assert m.snapshot(ws)==after
    backups=list(directory.glob('noteka-pre-migration-*'));assert len(backups)==1
    restored=out/'restored-v12';assert m.call(old,ws,'workspace','restore',dict(backup=str(backups[0]),destination=str(restored)))['schema_version']==12
    assert m.snapshot(restored)==data['tables'] and m.files(restored/'attachments')==data['originals']
    before=m.files(ws);m.call(old,ws,'workspace','open',{},'unsupported_workspace_version');assert m.files(ws)==before
    admission=subprocess.run([str(old.with_name('noteka-mcp')),'--workspace',str(ws)],input=b'',capture_output=True,timeout=8);assert admission.returncode and not admission.stdout and b'UnsupportedWorkspaceVersion' in admission.stderr
    assert m.files(ws)==before
    # Copias reales v12 desde el backup independiente para fallos/futuro. No
    # cambian versión ni simulan tablas antiguas con el código actual.
    failures=[]
    for label in ['before-version','after-version']:
        target=out/label;shutil.copytree(restored,target)
        with sqlite3.connect(target/'noteka.sqlite') as db:
            trigger=("CREATE TRIGGER fail_ocr BEFORE UPDATE OF schema_version ON workspace_meta WHEN NEW.schema_version=13 BEGIN SELECT RAISE(ABORT,'injected'); END" if label=='before-version' else "CREATE TRIGGER fail_ocr AFTER UPDATE OF schema_version ON workspace_meta WHEN NEW.schema_version=13 BEGIN SELECT RAISE(ABORT,'injected-after-version'); END")
            db.execute(trigger)
        prior=m.snapshot(target);m.call(new,target,'workspace','open',{},'storage_error');assert m.snapshot(target)==prior
        with sqlite3.connect(target/'noteka.sqlite') as db:assert db.execute('PRAGMA user_version').fetchone()[0]==12
        failures.append(label)
    future=out/'future';shutil.copytree(ws,future)
    with sqlite3.connect(future/'noteka.sqlite') as db:db.executescript('UPDATE workspace_meta SET schema_version=14;PRAGMA user_version=14;PRAGMA wal_checkpoint(TRUNCATE);')
    before=m.files(future);m.call(new,future,'workspace','open',{},'unsupported_workspace_version');assert m.files(future)==before
    report=dict(profile=args.profile,old_binary_sha256=hashlib.sha256(old.read_bytes()).hexdigest(),new_binary_sha256=hashlib.sha256(new.read_bytes()).hexdigest(),old_tables=len(data['tables']),new_tables=len(after),old_rows_exact=True,old_schema_exact=True,originals_exact=True,index_hashes_rows_cursors_exact=True,backup_v12_old_client_restore_exact=True,old_client_rejected_before_writes=True,future14_rejected_before_writes=True,rollback=failures,helper_available=False)
    (out/'results.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))

if __name__=='__main__':main()
