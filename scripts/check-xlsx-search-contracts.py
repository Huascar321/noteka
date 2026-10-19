#!/usr/bin/env python3
"""Comparación profunda v11/52; sólo permite las adiciones XLSX de index/search nombradas."""
import argparse
import copy
import json
from pathlib import Path
import subprocess


def legacy(value, counts):
    if isinstance(value, list):
        return [legacy(v, counts) for v in value]
    if not isinstance(value, dict):
        return value
    result = copy.deepcopy(value)
    defs = result.get('$defs')
    if defs:
        for key, expected in [('XlsxTextProjection', dict(type='string', enum=['xlsx_cell_fields_v1'])),
                              ('XlsxTextField', dict(type='string', enum=['cached_text', 'cached_number', 'cached_boolean', 'cached_error', 'cached_date_iso', 'formula_text']))]:
            if key in defs:
                assert defs.pop(key) == expected, key
                counts[key] = counts.get(key, 0) + 1
        if 'IndexedExtractor' in defs:
            assert defs['IndexedExtractor'] == dict(type='string', enum=['utf8_text', 'pdf_text', 'xlsx_cells'])
            defs['IndexedExtractor']['enum'].remove('xlsx_cells')
            counts['IndexedExtractor'] = counts.get('IndexedExtractor', 0) + 1
        if 'AttachmentTextLocator' in defs:
            variants = defs['AttachmentTextLocator']['oneOf']
            assert len(variants) == 3
            expected = dict(additionalProperties=False, type='object', properties={
                'kind': dict(type='string', const='xlsx_cell_text'),
                'projection': {'$ref': '#/$defs/XlsxTextProjection'},
                'reference': dict(type='string'), 'field': {'$ref': '#/$defs/XlsxTextField'},
                **{k: dict(type='integer', format='uint32', minimum=0) for k in
                   ['cell_ordinal', 'sheet_id', 'field_bytes', 'text_byte_start', 'text_byte_end']}})
            actual = variants.pop()
            assert set(actual.pop('required')) == set(expected['properties'])
            assert actual == expected, actual
            counts['AttachmentTextLocator'] = counts.get('AttachmentTextLocator', 0) + 1
    return {k: legacy(v, counts) for k, v in result.items()}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--baseline', type=Path, required=True)
    ap.add_argument('--cli', type=Path, default=Path('target/debug/noteka'))
    args = ap.parse_args()
    previous = json.loads(args.baseline.read_text())['data']
    process = subprocess.run([str(args.cli.resolve()), '--json', 'operations', 'describe'], capture_output=True, check=True)
    assert not process.stderr
    current = json.loads(process.stdout)['data']
    counts = {}
    for group in ['inputs', 'outputs']:
        assert set(current[group]) == set(previous[group]) and len(current[group]) == 52
        for op in current[group]:
            if op not in ['extraction.index', 'extraction.search'] or (group == 'outputs' and op == 'extraction.index'):
                assert current[group][op] == previous[group][op], (group, op)
            else:
                assert current[group][op] != previous[group][op]
                assert legacy(current[group][op], counts) == previous[group][op], (group, op)
    assert legacy(current['contract_schema'], counts) == previous['contract_schema']
    index = copy.deepcopy(current['attachment_index'])
    assert index.pop('extractors') == ['utf8_text', 'pdf_text', 'xlsx_cells']
    assert index.pop('xlsx_projection') == dict(version='xlsx_cell_fields_v1', fields=['cached_text', 'cached_number', 'cached_boolean', 'cached_error', 'cached_date_iso', 'formula_text'], chunk_bytes=4096, fragments_per_generation=512, text_bytes_per_generation=1048576, includes_hidden_sheets=True, cross_fragment_matching=False, recalculates_formulas=False, indexes_metadata=False)
    assert index == previous['attachment_index']
    for key in previous:
        if key not in ['inputs', 'outputs', 'contract_schema', 'attachment_index']:
            assert current[key] == previous[key], key
    assert set(current) == set(previous)
    assert len(current['batch']['mutations']) == 9
    print(json.dumps(dict(operations=52, unchanged_input_and_output_pairs=50, unchanged_xlsx_reader=True,
                          previous_contract_definitions=len(previous['contract_schema'].get('$defs', {})),
                          batch_mutations=9, named_additions_verified=counts), indent=2))


if __name__ == '__main__':
    main()
