# KAN-50 Remote Workflow Installation PR MVP

Updated: 2026-05-01

## Summary

KAN-50 adds the first remote pull-request based installation path for GitGov enterprise workflow template packs.

This turns the existing self-service workflow pack into a customer-reviewable GitHub PR:

- dry-run by default.
- remote mutation only with explicit `-Apply`.
- creates a branch, one commit, and a draft PR by default.
- writes only workflow YAML files directly under `.github/workflows`.
- never reads `.env` files or provider secret values.

## Script

```text
scripts/control-plane/open_enterprise_workflow_template_pr.ps1
```

Supported sources:

- `-PackDir`: output from `generate_enterprise_workflow_templates.ps1`.
- `-PackPath`: JSON workflow template pack downloaded from the dashboard.

Supported target:

- GitHub repository in `owner/repo` format.
- If `-Repository` is omitted, the script can infer `repository_full_name` from the pack manifest.
- If `-BaseBranch` is omitted, the script can infer `default_branch` from the pack manifest, then falls back to `main`.

## Behavior

Dry-run:

```powershell
.\scripts\control-plane\open_enterprise_workflow_template_pr.ps1 `
  -PackDir out\enterprise-workflow-templates `
  -Repository example-org/example-repo `
  -OutputPlanPath out\remote-workflow-pr-plan.json
```

Apply:

```powershell
.\scripts\control-plane\open_enterprise_workflow_template_pr.ps1 `
  -PackDir out\enterprise-workflow-templates `
  -Repository example-org/example-repo `
  -TicketId EX-123 `
  -Apply `
  -OutputPlanPath out\remote-workflow-pr-apply.json
```

The apply path:

1. reads the current base branch through the GitHub API.
2. compares each generated workflow file with the remote base branch.
3. refuses differing existing files unless `-Overwrite` is explicitly passed.
4. creates a Git tree containing only `create` and reviewed `update` files.
5. creates one commit.
6. creates a new branch.
7. opens a draft PR unless `-ReadyForReview` is passed.

## Plan Output

The JSON plan records:

- mode: `dry-run` or `apply`.
- repository.
- base branch and base SHA.
- branch name.
- PR title.
- totals for `create`, `update`, `skip`, and `blocked`.
- per-file reason and SHA-256 content hash.
- safety flags.
- PR URL and commit SHA when `-Apply` succeeds.

The plan does not store workflow file contents or secret values.

## Safety Rules

The script rejects:

- workflow paths outside `.github/workflows`.
- rooted, drive-qualified, nested, parent-directory, null-byte, or non-YAML workflow paths.
- duplicate workflow paths.
- packs that declare `contains_secret_values=true`.
- packs that declare repository mutation behavior.
- unsafe branch names.
- apply attempts with blocked existing files.
- apply attempts with no changes.
- apply attempts when the target branch already exists.

GitHub authentication is read from:

- `-GitHubToken`;
- `GITHUB_TOKEN`;
- `GH_TOKEN`; or
- authenticated `gh auth token`.

Token values are not printed.

## Customer-Facing Position

This is still customer-controlled installation.

GitGov prepares the PR, but the customer reviews, approves, and merges it. Generated workflows are not automatically enabled as required checks and do not automatically receive secret values.

## Non-Goals

- No GitHub App installation flow.
- No automatic GitHub Actions secret/variable creation.
- No forced merge.
- No branch protection mutation.
- No default release-blocking behavior.
- No provider webhooks or external provider settings mutation.
