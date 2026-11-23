# Static Hosting Migration Checklist (`getkineti.com`)

**Date:** November 23, 2026  
**Approval Gate (`AGENTS.md` Rule 10):** **NO live hosting change, DNS edit, or website deployment has been executed.** Migrating hosting or deploying `getkineti.com` requires separate explicit owner approval.

---

## 1. Why Migrate Off Vercel Hobby Before Commercial Launch

- **Vercel Hobby Terms of Service restriction:** Vercel’s Hobby (`$0`) tier is restricted to **personal, non-commercial use**. Hosting a commercial B2B landing page with paid Founding Team Pilot pricing (`$1,500` setup / `$299/mo` pilot) on Vercel Hobby violates Vercel’s Fair Use / Hobby Plan terms.
- **Zero-new-paid-spend guardrail (`$0`):** Upgrading to Vercel Pro costs `$20/seat/month`, which is unnecessary for a 6-file pure static site (`website/index.html`, `website/docs.html`, `website/terms.html`, `website/privacy.html`, `website/styles.css`, `website/favicon.svg`) with zero server-side runtime, zero build step, and zero external dependencies.
- **Current repo state:** Both root `vercel.json` and `website/vercel.json` already have `"git": { "deploymentEnabled": false }` so no automatic Vercel deployments trigger on push.

---

## 2. Commercial-Safe `$0` Hosting Options

### Option A (Recommended): Cloudflare Pages Free (`$0/month`)
- **Commercial use:** Permitted on Cloudflare’s Free plan for static sites.
- **Private repo compatibility:** Works cleanly with a private GitHub repository OR direct CLI/manual upload (`wrangler pages deploy website`) so `therawlogs/kineti` can stay **private** at `$0/month`.
- **Clean URLs parity:** Cloudflare Pages natively supports extensionless `.html` routes (`/docs` → `/docs.html`, `/terms` → `/terms.html`, `/privacy` → `/privacy.html`), matching `website/vercel.json`’s `"cleanUrls": true` setting.
- **Limits & SSL:** Unlimited static requests, automatic Let’s Encrypt / Google Trust Services TLS certificates, apex custom domain flattening (`getkineti.com` + `www.getkineti.com`).

### Option B: GitHub Pages (`$0/month` on public repo only)
- **Commercial use:** Permitted for company/product static marketing sites.
- **Important private-repo caveat:** On a personal GitHub Free account, GitHub Pages requires either a **public** repository or a paid GitHub Pro/Team plan (`$4/mo+`). To stay at `$0` while keeping `therawlogs/kineti` private, you would need a separate public static-only repository containing only `website/`.
- **Recommendation:** Use **Cloudflare Pages Free (Option A)** to keep `therawlogs/kineti` private with `$0` monthly spend.

---

## 3. Step-by-Step Migration Checklist (Cloudflare Pages)

### Phase 1 — Pre-Flight Audit (Before Touching DNS)
- [ ] Obtain explicit owner approval to deploy `website/` and migrate `getkineti.com`.
- [ ] Verify local `website/` directory contains **only** intended static files:
  - `website/index.html`
  - `website/docs.html`
  - `website/terms.html`
  - `website/privacy.html`
  - `website/styles.css`
  - `website/favicon.svg`
  - Confirm unused media (`kineti-cosmic-loop.mp4`, `kineti-cosmic-poster.jpg`) remain deleted.
- [ ] Export/screenshot **all** existing DNS records at your domain registrar / DNS provider (`A`, `AAAA`, `CNAME`, `TXT`, `MX`, `SPF`, `DKIM`, `DMARC`).
  - **Critical:** Do **not** alter `MX` or email authentication (`TXT` SPF/DKIM/DMARC) records so `hello@getkineti.com` and `security@getkineti.com` mail delivery is completely uninterrupted.
- [ ] Lower DNS TTL for `getkineti.com` and `www.getkineti.com` to `300` seconds (5 minutes) 12–24 hours ahead of cutover if supported.

### Phase 2 — Provision Cloudflare Pages Project (`$0`)
- [ ] Sign into Cloudflare Dashboard → **Workers & Pages** → **Create** → **Pages**.
- [ ] Choose deployment path:
  - **Direct Upload (lowest privilege — no GitHub OAuth grant needed):** Upload the `website/` folder directly (or deploy via reviewed `wrangler pages deploy website --project-name getkineti`).
  - **Git Integration (optional):** Connect `therawlogs/kineti` with:
    - Production branch: `main` (only after owner approves merging `feat/path-a-pr-gate`)
    - Build command: *(leave empty — zero build step)*
    - Build output directory: `website`
- [ ] Verify the preview URL (`https://<project>.pages.dev`) renders cleanly:
  - `/` loads with the Light Editorial layout, interactive 4-scene PR simulator, tabbed install switcher, Open-Core MIT + Founding Team Pilot pricing (`$1,500` / `$299/mo`), and zero broken links.
  - `/docs.html` and `/docs` both return HTTP `200`.
  - `/terms.html` and `/terms` both return HTTP `200` with `Last Updated: November 23, 2026`.
  - `/privacy.html` and `/privacy` both return HTTP `200`.
  - `/styles.css` and `/favicon.svg` return HTTP `200`.
  - Browser DevTools Network tab shows **zero** third-party requests, **zero** `404`s, and **zero** console errors.

### Phase 3 — DNS Cutover for `getkineti.com`
- [ ] In Cloudflare Pages → **Custom domains**, add `getkineti.com` and `www.getkineti.com`.
- [ ] If DNS is hosted on Cloudflare: let Cloudflare create the flattened apex `CNAME` (`getkineti.com` → `<project>.pages.dev`) and `www` `CNAME` (`www` → `<project>.pages.dev`).
- [ ] If DNS remains at an external registrar: add the TXT validation record Cloudflare provides, wait for TLS certificate issuance (`Active`), then switch apex/www records to Cloudflare’s assigned target.
- [ ] Add a single redirect rule: `https://www.getkineti.com/*` → `https://getkineti.com/$1` (`301 Permanent Redirect`).
- [ ] Verify live HTTPS & routes from an independent terminal:
  ```bash
  curl -sSI https://getkineti.com/
  curl -sSI https://getkineti.com/docs.html
  curl -sSI https://getkineti.com/terms.html
  curl -sSI https://getkineti.com/privacy.html
  curl -sSI https://getkineti.com/styles.css
  curl -sSI https://getkineti.com/favicon.svg
  curl -sSI https://getkineti.com/kineti-cosmic-loop.mp4  # expect 404
  ```
- [ ] Send a test email to `hello@getkineti.com` and confirm delivery still succeeds after DNS changes.

### Phase 4 — Decommission Vercel Hobby Project
- [ ] Wait 24 hours after clean DNS propagation and confirm zero errors on `getkineti.com`.
- [ ] Remove `getkineti.com` and `www.getkineti.com` from the Vercel Hobby project settings, then pause or delete the Vercel project.
- [ ] In a separate approved cleanup commit, optionally remove legacy `vercel.json` / `website/vercel.json`.

---

## 4. Alternative GitHub Pages Notes (Only If Moving Static Site to a Public Repo)

If you prefer GitHub Pages instead of Cloudflare Pages:
1. Create a dedicated **public** static-only repository (for example, `therawlogs/getkineti-site`) containing **only** the contents of `website/` plus a `CNAME` file (`getkineti.com`) and `.nojekyll`.
2. Keep `therawlogs/kineti` private (or publish it separately when ready).
3. Point `getkineti.com` apex `A` records to GitHub Pages IPs (`185.199.108.153`, `185.199.109.153`, `185.199.110.153`, `185.199.111.153`) and `www` `CNAME` to `therawlogs.github.io`, then enable **Enforce HTTPS**.
4. Note: GitHub Pages does not perform Cloudflare-style automatic `cleanUrls` extension stripping for every path without folder indices; all internal links in `website/` explicitly use `/docs.html`, `/terms.html`, and `/privacy.html` so both Cloudflare Pages and GitHub Pages work without broken links.
