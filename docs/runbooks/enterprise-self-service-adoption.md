# Enterprise Self-Service Adoption

Updated: 2026-04-30

Tickets: `KAN-29`, `KAN-30`, `KAN-31`, `KAN-32`, `KAN-33`, `KAN-34`, `KAN-35`

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

## Generate Workflow Templates

Run from the repository root:

```powershell
.\scripts\control-plane\generate_enterprise_workflow_templates.ps1 -ProfilePath docs/examples/enterprise-adoption-profile.example.json -OutputDir out/enterprise-workflow-templates -Force
```

Expected outputs:

```text
out/enterprise-workflow-templates/README.md
out/enterprise-workflow-templates/workflow-template-manifest.json
out/enterprise-workflow-templates/.github/workflows/*.yml
```

The workflow template pack is an onboarding artifact. It is meant to be reviewed before copying files into a customer repository.

## Download Workflow Templates From Dashboard

In the GitGov Enterprise Adoption panel:

1. Load or edit the adoption profile.
2. Save the profile if it should persist for the org.
3. Use `Workflows` to download the workflow template pack JSON.

The dashboard pack contains a manifest, README text, and generated workflow file contents. It keeps the same safety boundary as the PowerShell generator.

It does not:

- install workflows automatically.
- mutate customer repositories.
- read local `.env` files.
- include provider token values.
- generate secret values.

## Install Workflow Templates With Review

Use this only after reviewing the generated workflow pack. The installer is dry-run by default and writes files only when `-Apply` is passed.

Install from the CLI-generated pack directory:

```powershell
.\scripts\control-plane\install_enterprise_workflow_templates.ps1 -PackDir out/enterprise-workflow-templates -TargetRepoPath C:\path\to\customer-repo -OutputPlanPath out/workflow-install-plan.json
```

Install from the dashboard JSON pack:

```powershell
.\scripts\control-plane\install_enterprise_workflow_templates.ps1 -PackPath C:\path\to\workflow-template-pack.json -TargetRepoPath C:\path\to\customer-repo -OutputPlanPath out/workflow-install-plan.json
```

The plan reports each workflow file as:

- `create`: new workflow file would be added.
- `update`: existing workflow file would be replaced, only when `-Overwrite` is also used.
- `skip`: existing workflow file already matches.
- `blocked`: existing workflow file differs and needs review before overwrite.

After reviewing the plan, apply it:

```powershell
.\scripts\control-plane\install_enterprise_workflow_templates.ps1 -PackDir out/enterprise-workflow-templates -TargetRepoPath C:\path\to\customer-repo -OutputPlanPath out/workflow-install-plan-apply.json -Apply
```

Use `-Overwrite` only after reviewing replacements:

```powershell
.\scripts\control-plane\install_enterprise_workflow_templates.ps1 -PackDir out/enterprise-workflow-templates -TargetRepoPath C:\path\to\customer-repo -OutputPlanPath out/workflow-install-plan-overwrite.json -Apply -Overwrite
```

Safety boundaries:

- target path must be a git checkout with a `.git` marker.
- writes are limited to `.github/workflows/*.yml` and `.github/workflows/*.yaml`.
- unsafe paths such as `..`, rooted paths, drive-qualified paths, nested workflow paths, and non-YAML files are rejected.
- the installer does not read `.env` files, provider tokens, or secret values.
- the installer does not call GitHub APIs or mutate remote repositories.

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
- Treat generated workflow templates as customer-specific installation candidates, not as automatically approved production CI.
