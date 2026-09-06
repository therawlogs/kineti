import json
import re

with open('docs/HARNESS_STRATEGY_BLUEPRINT.md', 'r') as f:
    lines = f.readlines()

in_block = False
current_lang = ""
current_block = []
blocks = []
start_line = 0

for idx, line in enumerate(lines, 1):
    if line.startswith('```'):
        if in_block:
            blocks.append((current_lang, start_line, idx, "".join(current_block)))
            in_block = False
            current_lang = ""
            current_block = []
        else:
            in_block = True
            current_lang = line.strip()[3:].strip()
            start_line = idx
    elif in_block:
        current_block.append(line)

print(f"Total fenced code blocks found: {len(blocks)}")
json_blocks = [b for b in blocks if b[0] == 'json' or b[0] == 'jsonld']
print(f"Total json/jsonld code blocks: {len(json_blocks)}")

for lang, start, end, code in json_blocks:
    trimmed = code.strip()
    try:
        data = json.loads(trimmed)
        print(f"Lines {start}-{end}: VALID JSON (keys: {list(data.keys())[:5] if isinstance(data, dict) else type(data)})")
    except Exception as e:
        print(f"Lines {start}-{end}: Error parsing JSON: {e}")
        # print snippet
        lines_snippet = trimmed.splitlines()
        print(f"   First line: {lines_snippet[0]}")
        print(f"   Last line: {lines_snippet[-1]}")
