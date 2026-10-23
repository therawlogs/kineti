/** Normalize public options before any side effects; preserve the command after --. */
export function strictOptions(argv: string[], values: string[], flags: string[] = [], repeatable: string[] = [], maxPositionals = 1, allowDelimiter = false): string[] {
  const out: string[] = [], seen = new Set<string>();
  let positionals = 0;
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === "--") { if (!allowDelimiter) throw new Error("unexpected command delimiter"); return [...out, ...argv.slice(i)]; }
    if (!arg.startsWith("-")) { if (++positionals > maxPositionals) throw new Error(`unexpected argument: ${arg}`); out.push(arg); continue; }
    const eq = arg.indexOf("="), name = eq < 0 ? arg : arg.slice(0, eq);
    if (!values.includes(name) && !flags.includes(name)) throw new Error(`unknown option: ${name}`);
    if (seen.has(name) && !repeatable.includes(name)) throw new Error(`duplicate option: ${name}`);
    seen.add(name);
    if (flags.includes(name)) {
      if (eq >= 0) throw new Error(`${name} does not take a value`);
      out.push(name);
    } else {
      const value = eq < 0 ? argv[++i] : arg.slice(eq + 1);
      if (value === undefined || value === "" || value.startsWith("--")) throw new Error(`${name} needs a value`);
      out.push(name, value);
    }
  }
  return out;
}

export function checkedOptions(argv: string[], values: string[], flags: string[] = [], repeatable: string[] = [], maxPositionals = 1, allowDelimiter = false): string[] {
  try { return strictOptions(argv, values, [...flags, "--help", "-h"], repeatable, maxPositionals, allowDelimiter); }
  catch (error) { console.error(`kineti: ${(error as Error).message}`); process.exit(2); }
}
