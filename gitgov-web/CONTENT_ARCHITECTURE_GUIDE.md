# GitGov Web Content Architecture Guide

Internal working guide for the next content/frontend passes in `gitgov-web/`.

This file is not part of the public docs rendered under `/docs`. Its purpose is to define:

- what the website should sell
- what content is redundant or stale
- where each topic should live
- how future content updates should be organized

## Objective

Position GitGov as a modern B2B startup product for engineering governance, auditability, and operational evidence.

The site should feel like a product company website first, not a technical demo, internal architecture notebook, or admin runbook.

## Source Of Truth

Use this order when updating public copy:

1. `docs/IMPLEMENTATION_STATUS.md`
2. `docs/ARCHITECTURE.md`
3. `README.md`
4. `gitgov/` desktop surface and `gitgov/gitgov-server/` backend routes/features
5. Existing copy in `gitgov-web/`

Rule: `gitgov-web` is not the product source of truth. It is the presentation layer.

Additional rule for `/features`:

- Treat `docs/IMPLEMENTATION_STATUS.md` section `Website Feature Claims Alignment` as the canonical gating document for product claims.
- If a `/features` capability is not listed there, or is listed there with scope limits, the website copy must follow that exact scope.

## Executive Diagnosis

### Current structural problem

The site currently mixes three layers in the same public surface:

- marketing and positioning
- deep technical product explanation
- public docs and operational guidance

That creates:

- redundant sections
- weak commercial hierarchy
- outdated product framing
- copy that sounds internal instead of startup-grade

### What the site is doing well

- The core idea is strong: `commit -> CI -> compliance`
- The phrase "operational evidence" is differentiated and defensible
- Role-based framing is useful: CTO/CISO, Engineering Manager, Platform/DevOps
- Workstation-first and self-hosted angles build technical trust

### Main failure mode

The website is explaining too much of how GitGov is built and not enough of why a company should adopt it.

## What Is Redundant Today

### Repeated thesis

The same core message appears with minor rewording across:

- home hero
- home "what is GitGov"
- docs introduction
- features page

This should be one narrative, not four retellings.

### Repeated feature inventory

The same pillars appear across home, features, pricing, and download:

- Git capture
- append-only audit
- CI/Jira correlation
- governance/policy checks

Pricing currently re-lists product features instead of clarifying packaging or buying motion.

### Repeated objection handling

The site repeats the same trust answers across:

- home FAQ
- `/docs/faq`
- `/docs/security`
- `/privacy`

Topics repeated too often:

- "we do not read source code"
- "metadata only"
- "we do not replace CI/CD"
- "we do not make HR decisions"

These need clear ownership, not repetition.

### Repeated enterprise reassurance

`/contact` repeats value points already present in features and pricing:

- enterprise-grade security
- fast deployment
- integrations
- support

That page should qualify leads, not restate the site.

## What Is Stale Versus The Real Product

The public website has been partly reconciled by `KAN-75`, but it still needs product-design cleanup before a larger marketing rewrite. Treat this guide as an audit map, not as evidence that every listed stale claim is still present.

### Recently reconciled

- Jira preview wording was removed from the quickstart path; current copy should describe Jira as operational through API access and native signed webhooks.
- Governance wording now needs to stay precise: workstation policy checks can block configured pushes and record `blocked_push` evidence; `/policy/check` remains advisory by default with optional blocking for explicitly configured scopes; release governance gates are opt-in and should not be marketed as default release blocking.
- Public docs should describe managed production as Render HTTPS. Nginx/systemd/reverse-proxy guidance belongs to self-hosted deployment material.
- Pricing copy should frame enterprise evaluation and pilot fit until real plan packaging exists.
- Metadata copy should say GitGov stores bounded metadata such as file paths and counts, never source content or diffs.

### Still undersold

- Home/features still frame GitGov as traceability plus integrations, while the real product already has:
  - release readiness
  - risk outcomes
  - tier-aware scoring
  - quality gate visibility
  - export history
  - admin onboarding and org management
- Exports are underrepresented. The real surface is stronger than "`POST /export` and JSON".

### Areas that need careful wording

- Role access is partially ahead in backend/docs but not fully aligned in all UI surfaces.
- Do not over-market role-specific chat access until product rollout is fully aligned.

## Contradictions To Eliminate

These contradictions reduce trust and should be resolved before major copy rewrites. `KAN-75` removed several documentation-level contradictions; remaining UI/marketing contradictions should be handled as public web design work, not as another backend hardening chain.

- Hero CTA is visually primary but functionally dead.
- The hero contains an unsupported claim (`98% faster`) with no proof.
- Hero copy is partially hardcoded in Spanish despite bilingual support.
- Pricing should not imply finalized public plan packaging; use enterprise evaluation and pilot-fit language until packaging is real.
- OS/platform messaging is inconsistent across download/docs/FAQ.
- Governance wording must separate workstation blocking, advisory/default API checks, and opt-in release gates.
- Metadata wording must remain consistent that GitGov captures file metadata, including bounded paths/counts, but not source content or diffs.

## Commercial Positioning To Use Going Forward

### Category

GitGov is not just "Git observability" and not just "compliance logging".

The category to sell is:

`Engineering governance with operational evidence`

### ICP

Primary buyers:

- CTO
- CISO
- Head of Platform / Platform Engineering
- Engineering Manager in regulated or high-accountability environments

Good-fit organizations:

- teams using GitHub + Jenkins + Jira
- teams with audit pressure, incident investigations, or internal controls
- organizations that need traceability without reading source code

### Product promise

GitGov gives teams a defensible chain of evidence from workstation activity to pipeline execution and compliance reporting.

### Business outcomes to emphasize

- reduce manual evidence collection
- shorten audit and incident response cycles
- improve traceability across Git, CI, and tickets
- reduce dependency on tribal knowledge
- centralize governance without forcing code inspection

## Content Strategy Rules

### Do

- Lead with business value, not architecture internals
- Use product proof where possible
- Keep the tone confident, precise, and modern
- Explain technical depth only after the value proposition is clear
- Prefer "self-hosted", "metadata-only", "workstation-first", and "evidence chain"

### Do not

- Lead marketing pages with internals like `SKIP LOCKED`, `outbox.jsonl`, endpoints, headers, queue mechanics, or version labels
- Publish unsupported benchmarks or inflated stats
- Use `Preview`, `Coming Soon`, or `In Development` badges unless absolutely necessary
- Treat pricing, docs, and download as if they all need to carry the whole story
- Turn public docs into admin runbooks

## Target Information Architecture

### Home

Purpose: sell the product and create a clear next step.

Target section order:

1. Hero
2. Problem
3. How it works
4. Outcomes by buyer
5. Product proof
6. Trust and deployment
7. Final CTA

What to remove from first scroll:

- deep architecture bento
- internal implementation language
- unsupported benchmark claim
- overly negative FAQ focus

### Product Page

Keep `/features`, but treat it as the product page.

Organize around four commercial pillars:

1. Workstation capture
2. Governance engine
3. Integrations and evidence correlation
4. Risk, readiness, and reporting

This page can go deeper than home, but still should not read like backend source comments.

### Download

Purpose: activation page for technical champions.

Keep:

- Windows installer
- system requirements
- basic install steps
- checksum
- short value summary

Move out or compress:

- roadmap-heavy feature lists
- AI bot mention unless clearly market-ready
- OTA/update deep detail
- long operational explanations better suited for docs

### Pricing

Current recommendation:

- remove from top navigation until pricing is real
- or convert into a sales-conversation page, not a fake pricing matrix

If pricing remains public, it must answer:

- who should talk to sales
- what deployment motions exist
- what buying path looks like

### Contact

Purpose: qualify demand.

The current form is too generic for enterprise motion.

Future form fields should capture:

- team size
- current toolchain
- compliance context
- deployment preference
- desired outcome
- interest type: demo, pilot, pricing, partnership

### Docs

Public docs should be reorganized into three buyer-friendly buckets:

- Evaluate
- Deploy
- Operate

Recommended public docs:

- Introduction
- Security
- Privacy
- Installation
- Control Plane
- Governance basics
- CI/Jenkins/Jira setup
- Risk outcomes

Move detailed operational material out of public-first docs if it becomes admin-only:

- full endpoint catalogs
- rate limits
- exact headers
- environment variables
- RBAC tables with internal nuance
- invite/API-key workflows
- roadmap-like detail
- thresholds and calibration internals

## Topic Ownership Matrix

Use one canonical home for each topic.

| Topic | Canonical Home | Supporting Mention | Should Not Be Repeated Everywhere |
| --- | --- | --- | --- |
| Value proposition | Home hero | Product page | Download, pricing, intro docs in full |
| Problem framing | Home problem section | Introduction doc | Multiple long sections across marketing pages |
| How GitGov works | Home simplified flow | Product page, docs | Re-explained on every page |
| Technical depth | Product page | Docs | Hero/home first scroll |
| Installation | Download + Installation doc | Contact follow-up | Home/features/pricing |
| Security and privacy trust | Home trust summary + `/privacy` + `/docs/security` | FAQ | Multiple duplicated pages with same paragraphs |
| Pricing/buying motion | Pricing or Contact | Home CTA | Hidden inside features/download/docs |
| Governance internals | Docs | Product page summary | Home hero or marketing FAQ |
| Exports/compliance evidence | Product page + docs | Home trust summary | Buried only in FAQ |

## Recommended Narrative Flow

Use this as the default storytelling sequence for future rewrites.

```text
Visitor lands
  -> understands the costly problem
  -> sees GitGov's differentiated mechanism
  -> understands how it fits the stack
  -> sees why their role benefits
  -> sees credible product proof
  -> gets trust signals
  -> takes the next step (demo / pilot / docs / download)
```

## Page Flow Diagram

```text
HOME
  ├─ Hero: promise + ICP + CTA
  ├─ Problem: fragmented evidence chain
  ├─ How It Works: Desktop -> Control Plane -> Integrations
  ├─ Outcomes By Role
  ├─ Product Proof: commit -> build -> ticket -> export
  ├─ Trust: metadata-only / self-hosted / encrypted
  └─ CTA: request demo / start pilot

PRODUCT (/features)
  ├─ Workstation Capture
  ├─ Governance Engine
  ├─ Integrations
  └─ Risk / Readiness / Reporting

ACTIVATION
  ├─ /download
  └─ /contact

PUBLIC DOCS
  ├─ Evaluate
  ├─ Deploy
  └─ Operate

PRIVATE / INTERNAL DOCS
  ├─ Admin runbooks
  ├─ Endpoint detail
  ├─ Env vars
  ├─ Rate limits
  └─ Operational procedures
```

## Rewrite Priorities

### Phase 1: clarity and trust

- Fix the hero CTA
- Remove unsupported claims
- Remove or demote pricing from header
- Normalize hero language/i18n
- Remove stale `Preview` or maturity labels unless accurate

### Phase 2: content architecture

- Rebuild home around selling flow
- Move deep technical bento out of first-scroll marketing
- Reduce home FAQ to 3 trust objections max
- Recut download as activation, not roadmap

### Phase 3: product reality alignment

- Keep Jira maturity language aligned with operational API and signed webhook support
- Keep governance and CI docs aligned with workstation blocking, advisory/default API checks, and opt-in release readiness enforcement
- Surface exports, risk outcomes, release readiness, and admin/org features where commercially useful
- Resolve role wording drift

### Phase 4: docs cleanup

- Separate public docs from admin/internal detail
- Deduplicate privacy/security/FAQ content
- Build a public docs structure optimized for evaluation and deployment

## Checklist For Future Content Passes

Before merging any major content rewrite, verify:

- Is the page selling a product or explaining internals?
- Is the claim supported by the product today?
- Is the same idea already owned by another page?
- Is the wording aligned with `docs/IMPLEMENTATION_STATUS.md`?
- Is the call to action clear and intentional?
- Does the page help a buyer move forward?

If the answer to any of those is no, the content is not ready.
