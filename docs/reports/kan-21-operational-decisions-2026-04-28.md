# KAN-21 Operational Decision Clarification

Date: 2026-04-28

## Decision

Three operating decisions are now explicit so agents do not reopen the same questions.

## SonarCloud

SonarCloud is not a valid path for the current GitGov repository.

- The current GitHub repository/account is personal, not organizational.
- SonarCloud onboarding for this repo is not viable under that constraint.
- The selected runtime is local SonarQube at `http://localhost:9000`.
- Agents must not ask again to use SonarCloud for this repo unless the repository is moved to a GitHub organization.

## OpenAPI and SDKs

OpenAPI is the machine-readable API description used by Swagger tools and generated SDKs. It is not the API itself.

Current state:

- GitGov can be operated through the real backend routes/API.
- `/api-docs` is intentionally a partial schema explorer.
- `docs/ARCHITECTURE.md` plus the `main.rs` route table remain the operational route source of truth.

Full OpenAPI annotation is optional product work. Implement it only if generated SDKs or Swagger-based contract tests become a real requirement.

## Jenkins Trigger-Only

Jenkins authenticated API access is already configured and is the normal agent path.

It supports:

- inspection
- logs
- queue state
- build history
- authenticated build operations

`JENKINS_BUILD_TRIGGER_TOKEN` is only for the unauthenticated/manual URL flow:

```text
{JENKINS_SERVER_URL}/job/{JENKINS_JOB_NAME}/build?token={JENKINS_BUILD_TRIGGER_TOKEN}
```

That trigger-only token can start a build, but it cannot inspect logs or build status. Do not ask for it unless the user explicitly wants unauthenticated/manual URL build starts.

## Files Updated

- `AGENTS.md`
- `docs/IMPLEMENTATION_STATUS.md`
- `docs/reports/implementation-progress-summary-2026-04-25.md`
- `docs/runbooks/local-sonar-self-hosted-runner.md`
- `docs/runbooks/jenkins-trigger-token-flow.md`

## Validation Plan

- `git diff --check` - passed.
- `.\scripts\security\publication_guard.ps1` - passed.
