# Enterprise Self-Service Adoption

Updated: 2026-04-30

Ticket: `KAN-29`

## Purpose

Use this runbook to generate the first GitGov adoption pack for a customer or internal demo tenant.

The adoption pack is a planning artifact. It lists what to connect, which workflows to install, what policy preset applies, and which evidence modules are expected.

It must not contain provider tokens or secret values.

## Example Profile

```text
docs/examples/enterprise-adoption-profile.example.json
```

## Generate A Pack

Run from the repository root:

```powershell
.\scripts\control-plane\generate_enterprise_adoption_pack.ps1 -ProfilePath docs/examples/enterprise-adoption-profile.example.json -OutputDir out/enterprise-adoption-pack
```

Expected outputs:

```text
out/enterprise-adoption-pack/enterprise-adoption-pack.md
out/enterprise-adoption-pack/enterprise-adoption-pack.json
```

## Policy Presets

`audit-only`:

- gathers evidence.
- avoids release blocking.

`moderate`:

- requires ticket traceability.
- requires fresh evidence artifacts.
- blocks reachable critical/high vulnerabilities.
- targets release readiness score `75`.

`strict`:

- requires ticket traceability.
- requires PR review evidence.
- requires fresh evidence artifacts.
- blocks reachable critical/high vulnerabilities.
- requires medium-risk acceptance.
- targets release readiness score `85`.
- enables vulnerability trend enforcement.

## Safe Handling

- Use placeholder examples in reusable docs.
- Store provider tokens only in customer secret stores or GitHub Actions secrets.
- Do not paste `.env` values into adoption profiles.
- Treat generated packs as customer-specific planning evidence, not as a secret store.
