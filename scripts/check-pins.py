#!/usr/bin/env python3
"""Require exact versions in all workspace dependency declarations."""
import pathlib
import re
import sys
import tomllib

root = pathlib.Path(__file__).resolve().parent.parent
manifests = [root / 'Cargo.toml', *root.glob('crates/*/Cargo.toml')]
errors = []

def check_tables(node, manifest):
    if not isinstance(node, dict):
        return
    for key, value in node.items():
        if key in ('dependencies', 'dev-dependencies', 'build-dependencies') and isinstance(value, dict):
            for name, dep in value.items():
                if isinstance(dep, dict) and dep.get('workspace') is True:
                    continue
                version = dep if isinstance(dep, str) else dep.get('version', '')
                if not isinstance(version, str) or not re.fullmatch(r'=\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?', version):
                    errors.append(f'{manifest.relative_to(root)}: {name}: dependency version must be exact (=X.Y.Z), got {version!r}')
        else:
            check_tables(value, manifest)

for manifest in manifests:
    with manifest.open('rb') as f:
        check_tables(tomllib.load(f), manifest)
if errors:
    print('\n'.join(errors), file=sys.stderr)
    sys.exit(1)
print('All declared dependencies are exactly pinned')
