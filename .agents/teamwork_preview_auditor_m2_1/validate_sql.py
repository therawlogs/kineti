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

sql_blocks = [b for b in blocks if b[0] == 'sql']
print(f"Found {len(sql_blocks)} SQL blocks.")
for lang, start, end, code in sql_blocks:
    print(f"--- SQL Block Lines {start}-{end} ---")
    print(code.strip()[:300])
    print("...")
