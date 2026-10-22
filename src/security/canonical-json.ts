/** Validated legacy JSON and versioned, ASCII-only cross-language encoding. */
function wellFormed(text: string): boolean {
  for (let i = 0; i < text.length; i++) {
    const code = text.charCodeAt(i);
    if (code >= 0xd800 && code <= 0xdbff) {
      const next = text.charCodeAt(++i);
      if (!(next >= 0xdc00 && next <= 0xdfff)) return false;
    } else if (code >= 0xdc00 && code <= 0xdfff) return false;
  }
  return true;
}
export function assertJson(value: unknown): void {
  const active = new Set<object>();
  function visit(item: unknown, depth: number): void {
    if (depth > 128) throw new Error("JSON nesting limit exceeded");
    if (item === null || typeof item === "boolean") return;
    if (typeof item === "number") {
      if (!Number.isFinite(item) || (Number.isInteger(item) && !Number.isSafeInteger(item))) throw new Error("invalid or unsafe JSON number");
      return;
    }
    if (typeof item === "string") {
      if (!wellFormed(item)) throw new Error("unpaired Unicode surrogate");
      return;
    }
    if (!item || typeof item !== "object" || (!Array.isArray(item) && ![Object.prototype, null].includes(Object.getPrototypeOf(item)))) throw new Error("unsupported JSON value");
    if (active.has(item)) throw new Error("cyclic JSON value");
    if (Object.getOwnPropertySymbols(item).length) throw new Error("symbol JSON keys are unsupported");
    active.add(item);
    if (Array.isArray(item)) {
      if (Object.keys(item).length !== item.length) throw new Error("sparse arrays and extra array properties are unsupported");
      for (let i = 0; i < item.length; i++) {
        const descriptor = Object.getOwnPropertyDescriptor(item, String(i));
        if (!descriptor || !Object.hasOwn(descriptor, "value")) throw new Error("JSON accessors are unsupported");
        visit(descriptor.value, depth + 1);
      }
    } else for (const key of Object.keys(item)) {
      if (!wellFormed(key)) throw new Error("unpaired Unicode key surrogate");
      const descriptor = Object.getOwnPropertyDescriptor(item, key)!;
      if (!Object.hasOwn(descriptor, "value")) throw new Error("JSON accessors are unsupported");
      visit(descriptor.value, depth + 1);
    }
    active.delete(item);
  }
  visit(value, 0);
}

/** Keep the original finite-value hash bytes. This is not RFC 8785. */
export function legacyCanonicalJson(value: unknown): string {
  assertJson(value);
  const normalize = (item: any): any => {
    if (Array.isArray(item)) return item.map(normalize);
    if (item && typeof item === "object") return Object.fromEntries(Object.keys(item).sort().map(key => [key, normalize(item[key])]));
    return item;
  };
  return JSON.stringify(normalize(value));
}

/** kineti-canonical-v2: typed trees, UTF-8 hex strings, IEEE-754 hex numbers. */
export function canonicalJsonV2(value: unknown): string {
  assertJson(value);
  const hex = (text: string) => Buffer.from(text, "utf8").toString("hex");
  const encode = (item: any): unknown => {
    if (item === null) return ["null"];
    if (typeof item === "boolean") return ["bool", item];
    if (typeof item === "string") return ["string", hex(item)];
    if (typeof item === "number") {
      const bytes = Buffer.alloc(8); bytes.writeDoubleBE(Object.is(item, -0) ? 0 : item);
      return ["f64", bytes.toString("hex")];
    }
    if (Array.isArray(item)) return ["array", item.map(encode)];
    return ["object", Object.keys(item).sort().map(key => [hex(key), encode(item[key])])];
  };
  return JSON.stringify(["kineti-canonical-v2", encode(value)]);
}
