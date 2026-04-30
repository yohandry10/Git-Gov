# Dashboard Workflow Template Pack MVP

Updated: 2026-04-30

Ticket: `KAN-34`

## Goal

Make workflow-template onboarding available inside the GitGov dashboard.

KAN-33 added the PowerShell generator for reviewed workflow template packs. KAN-34 brings the same product capability into the Enterprise Adoption dashboard so an admin can configure or load a persisted adoption profile and download a workflow template pack without leaving the UI.

## Scope

Implemented in:

```text
gitgov/src/components/control_plane/dashboard-helpers.ts
gitgov/src/components/control_plane/EnterpriseAdoptionPanel.tsx
gitgov/src/test/components/dashboard-helpers.test.ts
```

The dashboard now supports:

- building a workflow template pack from the current adoption profile.
- downloading a JSON pack with:
  - manifest.
  - reviewed workflow template summaries.
  - variable names.
  - secret names.
  - manual install checklist.
  - generated workflow file contents.
  - README text.
- keeping the existing adoption pack JSON export.

## Output Shape

The dashboard download is a single JSON file:

```text
<customer>-<owner>-<repo>-workflow-template-pack.json
```

It contains:

- `manifest`: customer, repository, branch, policy preset, selected providers/modules, safety flags, variables, secrets, and manual steps.
- `files`: `{ file, reason, content }` entries for each generated workflow.
- `readme`: operator-facing install notes.

The ExampleCo profile generates `13` workflow template files.

## Safety Model

KAN-34 keeps the KAN-33 safety boundary:

- no `.env` reads.
- no provider token reads.
- no secret value generation.
- no secret value display.
- no GitHub repository mutation.
- no automatic workflow installation.

The pack uses GitHub Actions secret references such as `${{ secrets.GITGOV_API_KEY }}` by name only.

## Non-Goals

- No Vercel AI SDK Copilot.
- No automatic workflow installation.
- No direct provider credential validation.
- No formal enterprise release approval.
- No generated SDK or OpenAPI expansion.

## Next Product Steps

1. Add an explicit reviewed install flow for GitHub repositories.
2. Add direct provider credential/reachability checks where a customer grants explicit access.
3. Add formal enterprise release approval.
4. Start Vercel AI SDK Copilot only after onboarding is finished enough for the copilot to explain a complete adoption state.
