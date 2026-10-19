#!/usr/bin/env python3
"""Compara el catálogo actual con un baseline v1/51 independiente, no regenerado."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import subprocess


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--baseline-catalog', type=Path, required=True)
    ap.add_argument('--cli', type=Path, default=Path('target/debug/noteka'))
    args = ap.parse_args()
    before = json.loads(args.baseline_catalog.read_bytes())['data']
    r = subprocess.run([str(args.cli.resolve()), '--json', 'operations', 'describe'], capture_output=True, check=True)
    assert not r.stderr, r.stderr
    envelope = json.loads(r.stdout)
    assert envelope['ok'] and envelope['api_version'] == 1
    after = envelope['data']
    assert len(before['inputs']) == len(before['outputs']) == 51
    assert set(after['inputs']) == set(before['inputs']) | {'extraction.xlsx_cells'}
    assert set(after['outputs']) == set(before['outputs']) | {'extraction.xlsx_cells'}
    for group in ['inputs', 'outputs', 'operation_capabilities']:
        for name, value in before[group].items():
            assert after[group][name] == value, (group, name)
    assert len(before['batch']['mutations']) == 9
    assert after['batch'] == before['batch']
    normalized = copy.deepcopy(after)
    for group in ['inputs', 'outputs', 'operation_capabilities']:
        del normalized[group]['extraction.xlsx_cells']
    assert normalized['extraction']['extractors'] == ['utf8_text', 'pdf_text', 'xlsx_cells']
    normalized['extraction']['extractors'].remove('xlsx_cells')
    defs = normalized['contract_schema']['$defs']
    old_defs = before['contract_schema']['$defs']
    added = sorted(set(defs) - set(old_defs))
    assert added == sorted(['ReadXlsxCells', 'XlsxCellItem', 'XlsxCellLocator', 'XlsxCellPage', 'XlsxLocatorKind',
                            'XlsxManifest', 'XlsxSheetDescriptor', 'Cell', 'CellValue', 'Formula', 'FormulaKind',
                            'NumberStyle', 'Visibility'])
    for name in added:
        del defs[name]
    variants = defs['Operation']['oneOf']
    extra = [v for v in variants if v not in old_defs['Operation']['oneOf']]
    assert len(extra) == 1 and extra[0]['properties']['operation']['const'] == 'extraction.xlsx_cells'
    variants.remove(extra[0])
    for name, target in [('read_xlsx_cells', 'ReadXlsxCells'), ('xlsx_cells', 'XlsxCellPage')]:
        assert normalized['contract_schema']['properties'].pop(name) == {'$ref': f'#/$defs/{target}'}
        normalized['contract_schema']['required'].remove(name)
    assert normalized == before, 'Sólo se permite el contrato/variante/defs/policy aditivos y la lista de extractores.'
    digest = lambda v: hashlib.sha256(json.dumps(v, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()).hexdigest()
    print(json.dumps(dict(previous_operations=51, current_operations=52, batch_mutations=9,
                          previous_definitions=len(old_defs), added_definitions=added,
                          baseline_sha256=digest(before), current_sha256=digest(after), identical_after_removing_additions=True), indent=2))


if __name__ == '__main__':
    main()
