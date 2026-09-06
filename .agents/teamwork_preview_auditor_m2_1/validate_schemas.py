import json
import re
import sys

with open('docs/HARNESS_STRATEGY_BLUEPRINT.md', 'r') as f:
    content = f.read()

# find all ```json ... ``` blocks
pattern = re.compile(r'```(?:json|jsonld)?\s*\n(.*?)```', re.DOTALL)
matches = pattern.finditer(content)

count = 0
for match in matches:
    block = match.group(1).strip()
    # check if looks like json
    if not (block.startswith('{') or block.startswith('[')):
        continue
    count += 1
    # Check if contains mock ellipsis or comments
    clean_block = re.sub(r'//.*', '', block)
    # Check if mock ellipses like "z3h...AgentSignatureBase58..." are present
    try:
        parsed = json.loads(clean_block)
        print(f"Block {count}: Valid JSON ({len(block.splitlines())} lines). Top keys: {list(parsed.keys()) if isinstance(parsed, dict) else 'list'}")
    except Exception as e:
        print(f"Block {count} JSON parse error: {e}")
        # print first few lines of block
        lines = block.splitlines()
        print(f"   First 5 lines: {lines[:5]}")
        print(f"   Last 5 lines: {lines[-5:]}")

print(f"Total evaluated JSON-like blocks: {count}")
