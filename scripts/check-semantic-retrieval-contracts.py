#!/usr/bin/env python3
"""Contraste 5b/5c contra el catálogo independiente de 57 operaciones de 5a.

No compara contra un baseline inventado: exige que las 57 operaciones previas
sean byte-idénticas y que sólo existan las cinco adiciones nombradas.
"""
import argparse
import json
import subprocess
from pathlib import Path

ADDED = {"embedding.index", "retrieval.search", "agent.ask", "agent.cancel", "agent.trace"}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--cli", type=Path, required=True)
    parser.add_argument("--baseline", type=Path, required=True)
    args = parser.parse_args()
    raw_baseline = json.loads(args.baseline.read_text())
    baseline = raw_baseline.get("data", raw_baseline)
    result = subprocess.run(
        [str(args.cli.resolve()), "--json", "operations", "describe"],
        input=b"{}", capture_output=True, timeout=30,
    )
    assert result.returncode == 0 and not result.stderr, result.stderr
    current = json.loads(result.stdout)["data"]
    for group in ("inputs", "outputs"):
        assert set(baseline[group]) <= set(current[group])
        assert set(current[group]) - set(baseline[group]) == ADDED
        for name, schema in baseline[group].items():
            assert current[group][name] == schema, f"{group}.{name} cambió"
    assert current["api_version"] == baseline["api_version"] == 1
    assert current["batch"] == baseline["batch"]
    assert current["batch"]["mutations"].__len__() == 9
    assert current["attachment_index"] == baseline["attachment_index"]
    assert current["extraction"] == baseline["extraction"]
    for key, value in baseline.items():
        if key in {"inputs", "outputs", "contract_schema"}:
            continue
        if key == "operation_capabilities":
            for name, capability in value.items():
                assert current[key][name] == capability, name
            continue
        assert current[key] == value, key
    assert "agent" in current and current["agent"]["endpoint"] == "loopback_only"
    old_defs = baseline["contract_schema"].get("$defs", {})
    new_defs = current["contract_schema"].get("$defs", {})
    assert set(old_defs) <= set(new_defs)
    for name, schema in old_defs.items():
        if name == "Operation":
            old_variants = schema["oneOf"]
            new_variants = new_defs[name]["oneOf"]
            assert all(variant in new_variants for variant in old_variants)
            assert len(new_variants) == len(old_variants) + len(ADDED)
            continue
        assert new_defs[name] == schema, f"definición previa {name} cambió"
    old_props = baseline["contract_schema"].get("properties", {})
    new_props = current["contract_schema"].get("properties", {})
    assert set(old_props) <= set(new_props)
    for name, schema in old_props.items():
        assert new_props[name] == schema, f"propiedad previa {name} cambió"
    print(json.dumps({
        "baseline_operations": len(baseline["inputs"]),
        "current_operations": len(current["inputs"]),
        "added": sorted(ADDED),
        "prior_inputs_outputs_identical": True,
        "prior_definitions_identical": True,
        "batch_mutations": len(current["batch"]["mutations"]),
        "agent_tools": current["agent"]["tools"],
        "attachment_index_identical": True,
    }, indent=2))


if __name__ == "__main__":
    main()
