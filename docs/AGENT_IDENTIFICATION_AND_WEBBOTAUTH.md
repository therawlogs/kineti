# Agent Identification & The IETF Web Bot Auth Standard

How autonomous agents identify themselves to service providers, and where cryptographic verification replaces self-declared strings.

---

## 1. The Breakdown of Self-Declared Identity

Historically, clients identify themselves via the `User-Agent` HTTP header (RFC 9110). For AI agents, self-declared headers suffer from two opposing failure modes:

1. **Spoofing**: Agents pretending to be human desktop browsers (e.g. Chrome on macOS) to bypass bot defenses. This is adversarial and fragile.
2. **Unverifiable claims**: An agent sending headers like `User-Agent: MyAgent/1.0` or claiming `autonomy="supervised"` provides zero proof. Any scraper or malicious script can forge those strings.

Kineti's core thesis is **verification over self-declared statements**. An agent identification mechanism that relies on honest self-disclosure contradicts that premise unless backed by cryptographic proof.

---

## 2. The IETF Web Bot Auth Protocol

The **IETF Web Bot Auth Working Group** is actively standardizing verifiable automated client identification:

* **Specification**: `draft-ietf-webbotauth-httpsig-protocol-00`, *"HTTP Message Signatures for automated traffic"* (dated 2026-09-01).
* **Authorship**: Co-authored by engineers from **Cloudflare** and **Google**.
* **Production verification**: Verified in production networks by **Akamai** and **AWS WAF**.
* **Core mechanism**: Leverages **RFC 9421 (HTTP Message Signatures)**. Instead of relying on static IP allowlists or forgeable `User-Agent` strings, automated clients sign their HTTP requests using an asymmetric key pair.
* **The `Signature-Agent` header (Section 5.2.1)**: Points to a discovery document (such as a well-known JWKS directory) where the service provider can retrieve the public key and verify the signature on incoming requests.

*(Note: While Google documentation corroborates the RFC 9421 foundation, the draft's formal RFC reference entry remains subject to working group advancement).*

---

## 3. How Providers and Agents Interface

When automated traffic is cryptographically verifiable, service providers can safely adjust how they handle agent requests:

| Provider Challenge | Legacy Handling | Web Bot Auth Handling |
|---|---|---|
| **Bot detection** | CAPTCHAs, Turnstile challenges, 403 blocks | Cryptographic signature verification against public keys |
| **Rate limiting** | Opaque IP-based throttles | Deterministic `RateLimit-*` headers tied to the verified operator |
| **Content payload** | Heavy HTML/SPA bundles with scripts & ads | Clean structured representation (`text/markdown`, `application/json`) via opt-in content negotiation |
| **Errors** | HTML error pages | Machine-readable status codes and `Retry-After` headers |

---

## 4. Privacy & Anti-Patterns to Avoid

When designing agent-to-service communication:

1. **Never broadcast budget caps**: Proposing headers like `cost-cap="50.00-usd"` is a severe commercial and privacy leak. It reveals the operator's spending ceiling to external servers.
2. **Never break existing API clients**: Changing the default `Accept` header in generic HTTP clients to prefer `text/markdown` breaks existing API endpoints that expect JSON. Content negotiation for markdown must be **strictly opt-in** on specific documentation or web-scraping requests.
3. **Do not claim unowned standards**: Browser engines restrict the `Sec-` prefix for browser-controlled fetch operations (W3C / WHATWG Fetch specification). Protocol design must adhere to IETF standards (`Signature-Agent`).

---

## 5. Kineti's Cryptographic Boundary

Kineti provides cryptographic primitives that align with signed automated traffic, but maintains honest boundaries:

* **Native OVT Signatures (`core-native/crates/kineti-harness`)**: Since `v0.4.0`, Kineti uses `ed25519-dalek` to sign and verify Outcome Verification Tickets (OVT) with Ed25519 keys, enforcing role separation between workers and reviewers.
* **Local Scope**: The in-memory swarm signing demonstrated in TypeScript is a **local demo only**. It is not a distributed PKI or public identity infrastructure.
* **Egress Logging (`bin/kineti-egress.ts`)**: Outbound requests are recorded in a local hash-chained ledger before execution for human auditing, but Kineti does not yet generate external RFC 9421 HTTP signatures on general curl traffic.

---

## See Also

- [FINGERPRINT_LIMITATIONS.md](./FINGERPRINT_LIMITATIONS.md) — What workspace fingerprints and exit codes prove
- [UNDO_LIMITATIONS.md](./UNDO_LIMITATIONS.md) — SAGA rollback boundaries
- [PLAN.md](./PLAN.md) — Historical architecture specification
