# Canonical identities

Legacy local receipts, evaluation protocol v1, review summaries and native node identifiers retain their original serialization formats. Valid finite-value hash bytes are unchanged. They are not described as RFC 8785 compliant. New code rejects unsupported values instead of silently dropping them or replacing invalid numbers with zero.

`kineti-canonical-v2` is an additive, explicitly versioned cross-language format implemented in TypeScript and Rust. It encodes a typed tree as compact ASCII JSON. Null and boolean values have explicit tags; strings and object keys use lowercase UTF-8 hexadecimal; finite numbers use big-endian IEEE-754 binary64 hexadecimal. Negative zero becomes zero. Object keys sort by UTF-16 code units. Arrays preserve their order. Integers outside the exactly representable JavaScript safe-integer range, invalid number spellings, unsupported values, cycles, sparse arrays, accessors and unpaired Unicode surrogates are rejected. Nesting is limited to 128 levels.

Both implementations exercise matching decimal, zero and supplementary-Unicode test vectors. This format is not JCS and must not be substituted for a v1 serializer without changing the signed protocol or identity version. Existing native identifiers remain legacy identifiers. No old signed history is rewritten.

The npm contents contract is `package-contents.json`. The native workspace and lock live only in `core-native/`. Run native commands with `--manifest-path core-native/Cargo.toml`.
