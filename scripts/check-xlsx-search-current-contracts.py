#!/usr/bin/env python3
"""Contrato actual: consistencia y adiciones exactas; NO comparación histórica v11."""
import argparse
import importlib.util
import json
from pathlib import Path
import subprocess


def references(value, root):
    if isinstance(value, list):
        return sum(references(item, root) for item in value)
    if not isinstance(value, dict):
        return 0
    count = 0
    if '$ref' in value:
        ref = value['$ref']
        assert ref.startswith('#/'), ref
        target = root
        for part in ref[2:].split('/'):
            target = target[part.replace('~1', '/').replace('~0', '~')]
        count += 1
    return count + sum(references(item, root) for key, item in value.items() if key != '$ref')


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--cli', type=Path, required=True)
    parser.add_argument('--restart-catalog', type=Path, required=True)
    args = parser.parse_args()
    result = subprocess.run([str(args.cli.resolve()), '--json', 'operations', 'describe'],
                            input=b'{}', capture_output=True, timeout=15)
    assert result.returncode == 0 and not result.stderr
    catalog = json.loads(result.stdout)['data']
    restart = json.loads(args.restart_catalog.read_text())['data']
    assert catalog == restart  # v12 capturado al retomar, explícitamente no v11.
    inputs, outputs, capabilities = [catalog[key] for key in ['inputs', 'outputs', 'operation_capabilities']]
    assert len(inputs) == 52 and inputs.keys() == outputs.keys() == capabilities.keys()
    mutations = {'entity.create', 'entity.update', 'collection.create', 'property.define',
                 'record.create', 'record.attach', 'record.update', 'formula.set', 'view.save'}
    assert set(catalog['batch']['mutations']) == mutations
    assert {name for name, value in capabilities.items() if value['batch_mutation']} == mutations
    assert {name for name, value in capabilities.items() if value['batch_check']} == {'database.query'}
    for name in inputs:
        if name.startswith('extraction.'):
            assert not any(capabilities[name].values())
    refs = sum(references(schema, schema) for schemas in [inputs, outputs] for schema in schemas.values())
    index = catalog['attachment_index']
    assert catalog['api_version'] == index['version'] == 1
    assert index['extractors'] == ['utf8_text', 'pdf_text', 'xlsx_cells']
    assert index['profile'] == 'literal_unicode61_v1'
    assert index['limits'] == dict(busy_timeout_ms=2000, cursor_bytes=2048, execution_ms=5000,
        fragments=16384, generations=1024, get_page_default=32, page_max=64, query_bytes=256,
        response_bytes=2097152, search_page_default=20, snapshot_bytes=67108864, terms=16, text_bytes=8388608)
    assert index['xlsx_projection'] == dict(chunk_bytes=4096, cross_fragment_matching=False,
        fields=['cached_text', 'cached_number', 'cached_boolean', 'cached_error', 'cached_date_iso', 'formula_text'],
        fragments_per_generation=512, includes_hidden_sheets=True, indexes_metadata=False,
        recalculates_formulas=False, text_bytes_per_generation=1048576, version='xlsx_cell_fields_v1')
    # Reutiliza sólo las comprobaciones de forma de las adiciones nombradas;
    # no invoca la comparación con baseline perdido ni fabrica ese catálogo.
    spec = importlib.util.spec_from_file_location('shapes', Path(__file__).with_name('check-xlsx-search-contracts.py'))
    shapes = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(shapes)
    additions = {}
    for schemas in [inputs, outputs]:
        for name in ['extraction.index', 'extraction.search']:
            shapes.legacy(schemas[name], additions)
    assert additions['XlsxTextField'] and additions['XlsxTextProjection'] and additions['AttachmentTextLocator']
    print(json.dumps(dict(operations=52, batch_mutations=9, refs_resolved=refs,
                          named_additions=additions, restart_v12_catalog_identical=True,
                          historical_v11_comparison=False), indent=2))


if __name__ == '__main__':
    main()
