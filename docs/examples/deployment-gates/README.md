# Deployment Gate Provider Examples

Updated: 2026-06-14

These examples show how common CI/CD providers call GitGov before a production deployment.

They all use the same product contract:

```text
POST /deployment-gates/authorize
```

Required inputs:

- `GITGOV_URL`
- `GITGOV_API_KEY`
- `GITGOV_ORG_NAME` when the API key is global or platform-scoped
- repository full name
- branch
- target SHA
- release id
- environment
- deployer
- release-bound evidence packet hash

The evidence packet hash must already exist in GitGov and must match the release id, repository,
branch, target SHA, and environment. A stale packet from another commit should fail authorization.

Examples:

| Provider | File |
| --- | --- |
| GitHub Actions | `github-actions-deployment-gate.yml` |
| Jenkins Pipeline | `Jenkinsfile.deployment-gate` |
| GitLab CI | `gitlab-ci-deployment-gate.yml` |

Safe defaults:

- Missing GitGov configuration skips only when the gate is not enforced.
- Missing release-bound evidence hash skips only when the gate is not enforced.
- `blocking=true` fails the job.
- `would_block=true` fails only when the provider job opts into advisory failure.
- Secret values are read from provider secret stores and are not printed.

Break-glass:

- Provider jobs should not add `break_glass` automatically.
- Break-glass is accepted only when GitGov would otherwise return a blocking decision.
- A valid exception includes `requested=true`, a concrete reason, an authorizing actor, and optional expiry.
- GitGov returns `decision=break_glass`, `approved=true`, `blocking=true`, and `would_block=true`; original blockers remain in `blocked_by`.
