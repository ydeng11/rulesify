"""Restore manually-added skills that were removed by update-registry.

Compares the original registry.toml (saved before auto-generation) with the
freshly-generated one. Any skill present in the original but missing from the
generated file is a manually-added skill (not from a known SourceRepo) and
gets appended back to the generated file.

Usage: python3 restore_skills.py <original> <generated>
"""

import re
import sys


def get_skill_ids(content: str) -> set[str]:
    """Extract all top-level [skills.<name>] IDs from TOML content."""
    return set(re.findall(r'^\[skills\.([^.]+)\]$', content, re.MULTILINE))


def parse_skill_blocks(content: str) -> dict[str, str]:
    """Split TOML content into per-skill blocks using a line-based state machine.

    A block starts at [skills.<id>] and includes all subsequent lines (including
    nested tables like [skills.<id>.install_action]) until the next top-level
    [skills.<other>] section or end-of-file.
    """
    lines = content.split('\n')
    blocks: dict[str, str] = {}
    current_id: str | None = None
    current_lines: list[str] = []

    for line in lines:
        m = re.match(r'^\[skills\.([^.]+)\]$', line)
        if m:
            if current_id and current_lines:
                blocks[current_id] = '\n'.join(current_lines)
            current_id = m.group(1)
            current_lines = [line]
        elif current_id:
            current_lines.append(line)

    if current_id and current_lines:
        blocks[current_id] = '\n'.join(current_lines)

    return blocks


def main() -> None:
    if len(sys.argv) < 3:
        print(f"Usage: {sys.argv[0]} <original.toml> <generated.toml>", file=sys.stderr)
        sys.exit(1)

    original_path, generated_path = sys.argv[1], sys.argv[2]

    with open(original_path) as f:
        old = f.read()
    with open(generated_path) as f:
        new = f.read()

    old_ids = get_skill_ids(old)
    new_ids = get_skill_ids(new)
    missing = sorted(old_ids - new_ids)

    if not missing:
        print('All skills preserved by auto-generator')
        return

    blocks = parse_skill_blocks(old)

    with open(generated_path, 'a') as f:
        for sid in missing:
            if sid in blocks:
                f.write('\n' + blocks[sid] + '\n')
                print(f'Restored: {sid}')
            else:
                print(f'WARNING: {sid} missing from original backup — cannot restore')

    print(f'Restored {len(missing)} manually-added skill(s)')


if __name__ == '__main__':
    main()
