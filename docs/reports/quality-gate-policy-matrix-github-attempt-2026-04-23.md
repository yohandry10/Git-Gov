# Quality Gate Matrix Cloud Attempt Report

Generated (UTC): 2026-04-23 08:45
Repository: `yohandry10/Git-Gov`

## Objective

Trigger `quality-gate-policy-matrix.yml` on GitHub-hosted CI and validate `quality_gates=warn/block` matrix in cloud.

## Checks Performed

1. Verified workflow file presence on remote branch:
   - Branch: `tier-risk-sla-tuning`
   - File: `.github/workflows/quality-gate-policy-matrix.yml`
   - Result: present (`sha=ae1918a0023d5ab6132a9af2b4faf3c8cc46c62b`)
2. Verified default-branch workflow inventory (`origin/main`) through GitHub Actions workflow list.
   - Result: matrix workflow not listed on current default-branch inventory.
3. Attempted API dispatch:
   - Endpoint: `POST /repos/yohandry10/Git-Gov/actions/workflows/quality-gate-policy-matrix.yml/dispatches`
   - Ref: `tier-risk-sla-tuning`
   - Result: `403 FORBIDDEN`
   - GitHub accepted permission hint: `actions=write`

## Outcome

Cloud matrix execution could not be started from automation with current PAT.

## Additional Cloud Execution Evidence

A GitHub-hosted run was executed by temporarily enabling push trigger on branch
`ci/quality-gate-matrix-main` and pushing commit `4e71a0cb004a8138ce5afaac5696833084c9d588`.

- Run URL: `https://github.com/yohandry10/Git-Gov/actions/runs/24826230934`
- Workflow: `Quality Gate Policy Matrix (Optional)`
- Job conclusion: `success`
- Effective matrix execution: `skipped`
- Skip reason from job logs:
  - `missing_gitgov_url_or_api_key`
  - `Skipping quality gate matrix: missing GITGOV_URL or GITGOV_API_KEY.`

After collecting evidence, branch trigger was reverted in commit
`7496477` to keep workflow trigger scope as `push/main`.

## Additional Cloud Execution Evidence (Fallback Name Mapping)

A second GitHub-hosted run was executed after extending workflow mapping to accept
alternate variable/secret names (`GITGOV_SERVER_URL`, `GIT_GOV_URL`,
`GITGOV_TOKEN`, `GIT_GOV_API_KEY`, and hyphenated variants).

- Run URL: `https://github.com/yohandry10/Git-Gov/actions/runs/24826556179`
- Workflow: `Quality Gate Policy Matrix (Optional)`
- Job conclusion: `success`
- Effective matrix execution: `skipped`
- Skip reason from job logs:
  - `missing_gitgov_url_or_api_key`
  - `Skipping quality gate matrix: missing GITGOV_URL or GITGOV_API_KEY.`

After collecting evidence, temporary branch trigger was reverted again in commit
`dbae64c` to keep workflow trigger scope as `push/main`.

## Missing to Close

1. Ensure `.github/workflows/quality-gate-policy-matrix.yml` is published on `main`.
2. Configure GitHub Actions repo config for cloud matrix precheck:
   - variable `GITGOV_URL`
   - secret `GITGOV_API_KEY`
3. Use PAT/GitHub App token with `actions=write` for API-based dispatch.
4. For strict CI config audits (`check_ci_repo_config.ps1`), grant `secrets=read` and `actions_variables=read`.
