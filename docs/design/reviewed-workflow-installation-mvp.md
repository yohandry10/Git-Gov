# Reviewed Workflow Installation MVP

Updated: 2026-04-30

Ticket: `KAN-35`

## Goal

Close the next Enterprise Self-Service onboarding gap after workflow pack generation.

KAN-33 generates workflow templates from an adoption profile. KAN-34 lets an admin download the same pack from the dashboard. KAN-35 adds a reviewed installation step so an operator can install those templates into a local customer repository checkout without giving GitGov automatic remote mutation rights.

## Scope

Implemented in:

```text
scripts/control-plane/install_enterprise_workflow_templates.ps1
```

The installer supports two source formats:

- `-PackDir`: the directory output from `generate_enterprise_workflow_templates.ps1`.
- `-PackPath`: the single JSON workflow pack downloaded from the dashboard.

The installer requires:

- `-TargetRepoPath` pointing at a git checkout with a `.git` marker.
- dry-run by default.
- `-Apply` before any workflow file is written.
- `-Overwrite` before any differing existing workflow file is replaced.
- optional `-OutputPlanPath` for a JSON install plan.

## Install Plan

Every run produces a plan with:

- source type.
- target repository path.
- write mode.
- overwrite mode.
- safety flags.
- per-file status:
  - `create`
  - `update`
  - `skip`
  - `blocked`

`blocked` means a target workflow file already exists and differs from the template. In dry-run mode this is reported for review. In apply mode the installer refuses to continue unless `-Overwrite` is explicitly passed.

## Path Safety

The installer accepts only workflow files directly under:

```text
.github/workflows/
```

It rejects:

- rooted paths.
- drive-qualified paths.
- parent directory segments such as `..`.
- nested workflow subdirectories.
- non-`.yml` or non-`.yaml` files.
- duplicate workflow file paths.
- null bytes in paths or file contents.
- packs that declare `contains_secret_values=true`.
- packs that declare repository mutation behavior.

The resolved write target is checked again before writing so a crafted path cannot escape `.github/workflows`.

## Safety Model

KAN-35 keeps the self-service boundary conservative:

- no GitHub API mutation.
- no automatic pull request creation in customer repositories.
- no `.env` reads.
- no provider token reads.
- no secret value printing.
- no workflow overwrite without `-Overwrite`.
- no write at all without `-Apply`.

The installer copies reviewed template content. It does not certify that every command inside a customer-customized workflow is safe; operators still review the generated YAML before applying it.

## Operator Flow

1. Generate workflow templates from CLI or download them from the dashboard.
2. Run the installer in dry-run mode and write a plan JSON.
3. Review files marked `create`, `update`, or `blocked`.
4. Run again with `-Apply` only after review.
5. Use `-Overwrite` only for reviewed replacements.
6. Run the installed workflows manually with `workflow_dispatch` before relying on schedules or blocking behavior.

## Non-Goals

- No direct GitHub remote write or GitHub App installation.
- No automatic PR creation in customer repositories.
- No direct provider credential/reachability checks.
- No formal enterprise release approval model.
- No Vercel AI SDK Copilot.

## Next Product Steps

1. Add direct provider credential/reachability checks where a customer explicitly grants access.
2. Add formal enterprise release approval with approvers, risk acceptance, expiration, and evidence binding.
3. Start Vercel AI SDK Copilot after onboarding has enough complete evidence to explain.
