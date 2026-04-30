# GitGov Web — Public Website

> Marketing site + documentation + download portal for GitGov.  
> Built with **Next.js 15.5.10** (App Router) + **TypeScript** + **Tailwind CSS** + **Framer Motion**.

## Quick Start

```bash
cd gitgov-web
pnpm install
pnpm dev
```

Open [http://localhost:3000](http://localhost:3000).

For faster local compilation:

```bash
pnpm dev:turbo
```

## Scripts

| Command | Description |
|---------|-------------|
| `pnpm dev` | Start development server |
| `pnpm dev:turbo` | Start dev server with Turbopack |
| `pnpm build` | Production build |
| `pnpm start` | Start production server |
| `pnpm lint` | Run ESLint |
| `pnpm typecheck` | TypeScript type checking |

## Pages

| Route | Description |
|-------|-------------|
| `/` | Landing page |
| `/features` | Feature overview |
| `/download` | Desktop app download |
| `/contact` | Contact form |
| `/pricing` | Pricing (coming soon) |
| `/docs` | Documentation |
| `/docs/[slug]` | Individual doc page |

## Desktop `.exe` for Download

Place the installer file at:

```
public/downloads/GitGov_0.1.0_x64-setup.exe
```

Update the version and filename in `lib/config/site.ts` if needed.

## Project Structure

```
gitgov-web/
├── app/                    # Next.js App Router pages
│   ├── (marketing)/        # Marketing pages (features, download, contact, pricing)
│   ├── api/                # API routes (contact, download, copilot)
│   └── docs/               # Documentation pages
├── components/
│   ├── layout/             # Header, Footer, Container, Preloader
│   ├── marketing/          # Hero, FeatureCard, CTASection, etc.
│   ├── download/           # DownloadCard, ReleaseInfo
│   └── ui/                 # Button, Badge, Card, Input, etc.
├── content/docs/           # Markdown documentation files
├── lib/
│   ├── config/             # Site configuration
│   ├── seo/                # Metadata helpers
│   ├── analytics/          # Analytics scaffold (no-op)
│   └── content/            # Docs loader
└── public/downloads/       # Place .exe here
```

## Tech Stack

- **Next.js 15.5.10** — App Router, RSC
- **TypeScript** — Strict mode
- **Tailwind CSS 3** — Custom design tokens
- **Framer Motion** — Animations, parallax, scroll reveal
- **React Icons** — Iconography
- **gray-matter + react-markdown + remark-gfm** — Markdown docs

## Performance Notes (Dev vs Prod)

- `next dev` compiles routes on-demand; first navigation to a route is expected to be slower.
- Measure real Web Vitals in production mode:
  ```bash
  pnpm build
  pnpm start
  ```
- Keep these guardrails to avoid LCP/CLS regressions:
  - Avoid global preloads of heavy assets that are not needed by every route.
  - Use `next/font` instead of CSS `@import` for Google Fonts.
  - Do not render fullscreen preload overlays on every route.

## Note

This is the **public-facing website only**. It does not replace:
- The Desktop App (`gitgov/`)
- The Control Plane Server (`gitgov/gitgov-server/`)

## Copilot API

The first Vercel AI SDK Copilot route is:

```text
POST /api/copilot/governance
```

It gathers bounded GitGov evidence server-side and returns a cited governance brief. The caller must provide a GitGov bearer token in the `Authorization` header unless explicitly configured server-key mode is enabled.
