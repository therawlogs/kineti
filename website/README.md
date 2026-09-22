# Kineti website (MIT)

6-route React static site. Light theme. No images.

## Routes

- Home `/` — open research for human progress, first front: agent verification
- Tools `/tools` — all 15 tools, search, tier filter, sort. Gate for MCP leads.
- Research `/research` — papers with author, search, field filter, sort. Full text lands here later.
- Docs `/docs` — library of tools, one page per tool at `/docs/:slug`
- Roadmap `/roadmap` — every tool and paper with status and dates. Request via issues, star the repo.
- Contribute `/contribute` — request form. Everything lands as a public issue.

Forms open GitHub issues. No database.

## Run

```sh
cd website
bun install
bun run dev
```

## Ship

```sh
cd website
bun run build
```

`dist/` is a static folder. Hash routing means it works from any path.
Zip `dist/` and drop it into the kineti repo, or serve it as-is.
