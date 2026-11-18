#!/usr/bin/env python3
"""Fix pattern matches that don't mention focus_tab field."""

import re

# Lines to fix: (file, line_number)
errors = [
    ("mobile/src/ui_tabs/ns.rs", 1095),
    ("mobile/src/ui_tabs/platform.rs", 1195),
    ("mobile/src/ui_tabs/platform.rs", 1254),
    ("mobile/src/ui_tabs/platform.rs", 1316),
    ("mobile/src/ui_tabs/platform.rs", 1372),
    ("mobile/src/ui_tabs/platform.rs", 1399),
    ("mobile/src/ui_tabs/platform.rs", 1520),
    ("mobile/src/ui_tabs/ssh.rs", 656),
    ("mobile/src/ui_tabs/ssh.rs", 661),
    ("mobile/src/ui_tabs/ssh.rs", 680),
    ("mobile/src/ui_tabs/ssh.rs", 764),
    ("mobile/src/ui_tabs/ssh.rs", 825),
    ("mobile/src/ui_tabs/ssh.rs", 893),
    ("mobile/src/ui_tabs/ssh.rs", 938),
    ("mobile/src/ui_tabs/ssh.rs", 961),
    ("mobile/src/ui_tabs/ssh.rs", 984),
    ("mobile/src/ui_tabs/ssh.rs", 1035),
    ("mobile/src/ui_tabs/ssh.rs", 1058),
    ("mobile/src/viewmodel/ssh/actor.rs", 2153),
    ("mobile/src/viewmodel/ssh/actor.rs", 2154),
]

# Group by file
from collections import defaultdict
files_to_fix = defaultdict(list)
for filepath, lineno in errors:
    files_to_fix[filepath].append(lineno)

for filepath, line_numbers in files_to_fix.items():
    with open(filepath, 'r') as f:
        lines = f.readlines()

    # Sort line numbers in reverse to avoid offset issues
    for lineno in sorted(line_numbers, reverse=True):
        idx = lineno - 1  # Convert to 0-indexed
        if idx >= len(lines):
            print(f"Warning: {filepath}:{lineno} out of range")
            continue

        line = lines[idx]

        # Check if this line has a struct pattern { ... }
        # Pattern: SomeEvent { field1, field2 }
        # We need to add .. if not present, or add focus_tab: _,

        # If the line ends with } or }) or }), we need to add .. before the }
        # Strategy: replace } with , .. } if it doesn't already have ..

        if ' {' in line:
            # This is the opening of a struct pattern
            # Need to look forward to find the closing }
            # For now, let's assume single-line patterns or we need .. at the end

            # Check if line already has ..
            if '..' in line:
                continue

            # Find the last } on this line or look forward
            if '}' in line:
                # Single line pattern - add .. before the }
                # Match: { fields } and replace with { fields, .. }
                # Be careful with nested braces

                # Simple approach: if line contains } and doesn't contain .., add .. before last }
                # Find position of last }
                last_brace = line.rfind('}')
                if last_brace > 0:
                    # Insert ", .." before the }
                    # Check if there's already a trailing comma
                    before_brace = line[:last_brace].rstrip()
                    if before_brace.endswith(','):
                        new_line = before_brace + ' ..' + line[last_brace:]
                    else:
                        new_line = before_brace + ', ..' + line[last_brace:]

                    lines[idx] = new_line
                    print(f"Fixed {filepath}:{lineno}")

    # Write back
    with open(filepath, 'w') as f:
        f.writelines(lines)

print("Done!")
