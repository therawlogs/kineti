# Static Hosting Guide — Cloudflare Pages (`getkineti.com`)

**Date:** November 23, 2026  
**Cost:** `$0/month` (Cloudflare Pages Free — commercial use permitted, works with private GitHub repos)  
**Vercel status:** Legacy `vercel.json` and `website/vercel.json` have been removed from the repository.

---

## 1. Why Cloudflare Pages Free (`$0/month`)

- **Commercial-safe at `$0`:** Cloudflare Pages Free permits commercial product websites with unlimited static requests and automatic TLS certificates.
- **Direct GitHub integration on private repos:** Connect `therawlogs/kineti` directly to Cloudflare Pages while keeping the repository **private** at `$0/month`—every merge to `main` auto-deploys `getkineti.com` in ~20 seconds.
- **Zero build step & native security headers:** The static site under `website/` contains 7 self-contained files with zero frameworks, zero external CDN scripts, and zero trackers:
  - `website/index.html`
  - `website/docs.html`
  - `website/terms.html`
  - `website/privacy.html`
  - `website/styles.css`
  - `website/favicon.svg`
  - `website/_headers` (enforces `nosniff`, `X-Frame-Options: DENY`, `Referrer-Policy: no-referrer`, and zero-egress CSP)
- **Automatic clean URLs:** Cloudflare Pages automatically serves `/docs` → `/docs.html`, `/terms` → `/terms.html`, and `/privacy` → `/privacy.html`.

---

## 2. Connect GitHub (`therawlogs/kineti`) to Cloudflare Pages

1. Sign in to the **Cloudflare Dashboard** → **Workers & Pages** → **Create** → **Pages** → **Connect to Git**.
2. Authorize the **Cloudflare Pages GitHub App** and grant access **only** to `therawlogs/kineti` (works for both private and public repositories).
3. Select `therawlogs/kineti` and configure the build settings:
   - **Project name:** `getkineti`
   - **Production branch:** `main`
   - **Framework preset:** `None`
   - **Build command:** *(leave completely empty)*
   - **Build output directory:** `website`
4. Click **Save and Deploy**.
5. Verify the preview deployment (`https://getkineti.pages.dev`):
   - `/` loads the Crisp Light Editorial homepage, interactive 4-scene PR simulator, tabbed install switcher, and Open-Core MIT + Founding Team Pilot pricing (`$1,500` / `$299/mo`).
   - `/docs.html`, `/terms.html`, `/privacy.html`, `/styles.css`, and `/favicon.svg` all return HTTP `200`.
   - Browser DevTools Network tab shows zero third-party requests, zero `404`s, and zero console errors.

*(Optional alternative: if you prefer not to grant GitHub App access at all, choose **Upload assets / Direct Upload** or `wrangler pages deploy website --project-name getkineti`.)*

---

## 3. Attach Custom Domain (`getkineti.com`) Without Breaking Email

1. Before editing DNS, screenshot/export all existing DNS records (`A`, `AAAA`, `CNAME`, `TXT`, `MX`, `SPF`, `DKIM`, `DMARC`).
   - **Critical:** Do **not** touch `MX` or email `TXT` (SPF/DKIM/DMARC) records so `hello@getkineti.com` and `security@getkineti.com` remain uninterrupted.
2. In your Cloudflare Pages project → **Custom domains**, add `getkineti.com` and `www.getkineti.com`.
3. Follow Cloudflare’s DNS prompt (flattened apex `CNAME` to `getkineti.pages.dev` if DNS is on Cloudflare, or the TXT verification + CNAME/ALIAS records provided by Cloudflare if DNS stays at your registrar).
4. Add a `301 Permanent Redirect` rule from `https://www.getkineti.com/*` → `https://getkineti.com/$1`.
5. Verify live HTTPS, routes, and security headers:
   ```bash
   curl -sSI https://getkineti.com/
   curl -sSI https://getkineti.com/docs.html
   curl -sSI https://getkineti.com/terms.html
   curl -sSI https://getkineti.com/privacy.html
   curl -sSI https://getkineti.com/styles.css
   curl -sSI https://getkineti.com/favicon.svg
   curl -sSI https://getkineti.com/kineti-cosmic-loop.mp4  # expect 404
   ```
6. Send a test email to `hello@getkineti.com` to confirm mail routing still works cleanly.

---

## 4. Alternative: GitHub Pages (`$0/month` Only on Public Repos)

If you ever prefer GitHub Pages instead of Cloudflare Pages:
- On a personal GitHub Free plan, GitHub Pages requires a **public** repository (either making `therawlogs/kineti` public later, or copying only `website/` into a separate public `therawlogs/getkineti-site` repo with `CNAME` and `.nojekyll`).
- All internal links in `website/` explicitly use `/docs.html`, `/terms.html`, and `/privacy.html`, so both Cloudflare Pages and GitHub Pages work cleanly with zero broken links.
