"""Verify that all skills from the original registry are preserved.

Exits with code 0 if all original skill IDs are found in the merged file,
otherwise exits with code 1 and lists the missing skills.

Usage: python3 verify_skills.py <original.toml> <merged.toml>
"""

import re
import sys


def get_skill_ids(content: str) -> set[str]:
    """Extract all top-level [skills.<name>] IDs from TOML content."""
    return set(re.findall(r'^\[skills\.([^.]+)\]$', content, re.MULTILINE))


def main() -> None:
    if len(sys.argv) < 3:
        print(f"Usage: {sys.argv[0]} <original.toml> <merged.toml>", file=sys.stderr)
        sys.exit(1)

    original_path, merged_path = sys.argv[1], sys.argv[2]

    with open(original_path) as f:
        old = f.read()
    with open(merged_path) as f:
        merged = f.read()

    old_ids = get_skill_ids(old)
    merged_ids = get_skill_ids(merged)

    still_missing = sorted(old_ids - merged_ids)
    if still_missing:
        print(f'FAIL: {len(still_missing)} skill(s) still missing from registry.toml:')
        for s in still_missing:
            print(f'  - {s}')
        sys.exit(1)

    print(f'OK: All {len(old_ids)} skills from original registry are preserved')
    print(f'Total skills now: {len(merged_ids)}')


if __name__ == '__main__':
    main()
