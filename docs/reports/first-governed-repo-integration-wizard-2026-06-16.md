# First Governed Repo Integration Wizard

Ticket: `KAN-120`

## Decision

After GPT/product consultation and local repo audit, the next roadmap slice after KAN-119 is not
another compliance artifact. It is the missing activation layer for `0.1 Deployment Gates`: turn the
existing KAN-80 First Governed Repo Setup into a small manual-first integration wizard.

The wizard must orchestrate what GitGov already has instead of duplicating configuration:

- Reuse `enterprise_first_governed_repo_setups` and the KAN-80 baseline shape.
- Let Admins create/resume, update, validate, plan, and complete the first governed repo run.
- Let Auditors read state only.
- Keep Developers, unrelated tenants, and Agent Governance keys out.
- Validate only evidence already visible to GitGov; do not read provider secrets.
- Produce a first result for advisory Deployment Gate simulation, not release blocking by default.

## Implemented Surface

Backend routes:

- `GET /onboarding/first-governed-repo/state`
- `POST /onboarding/first-governed-repo/runs`
- `PATCH /onboarding/first-governed-repo/runs/{run_id}`
- `POST /onboarding/first-governed-repo/runs/{run_id}/validate`
- `POST /onboarding/first-governed-repo/runs/{run_id}/plan`
- `POST /onboarding/first-governed-repo/runs/{run_id}/complete`

Desktop/Tauri:

- DTOs and client methods for the wizard responses/actions.
- Tauri commands registered for every wizard step.
- Control Plane store state/actions for wizard state and action execution.
- `FirstGovernedRepoSetupPanel` now shows wizard state, backend gaps, provider health, and manual
  Start, Validate, Plan, and Complete actions.

Maintainability:

- Split backend code into focused files:
  - `first_governed_repo_setup.rs`
  - `first_governed_repo_wizard_helpers.rs`
  - `first_governed_repo_wizard.rs`
- No new database migration was required; KAN-120 uses the KAN-80 table and baseline JSON.

## Explicit Non-Goals

- No OAuth/provider connection wizard.
- No provider token storage.
- No provider or customer repository mutation.
- No deploy execution.
- No public links, email delivery, scheduler, compliance score, certification, official regulatory
  claim, or legal attestation.
- No Agent Governance or AI dependency.

## Validation

Completed locally:

- Backend `cargo fmt --check`.
- Backend `cargo check`.
- Backend `cargo clippy -- -D warnings`.
- Focused real Postgres test with `TEST_DATABASE_URL` on local `127.0.0.1:5433`:
  `first_governed_repo_setup` passed.
- The focused backend test covers Admin-only mutation, Auditor state read, Developer denial, Agent
  Governance key denial, tenant isolation, validate/plan/complete actions, admin audit records,
  secret-safety flags, advisory gate first result, and no Agent Governance evaluation creation.
- Full backend test suite with the same local Postgres `TEST_DATABASE_URL` and `--test-threads=2`:
  `311` passed.
- Tauri `cargo fmt --check`.
- Tauri `cargo check`.
- Tauri `cargo clippy -- -D warnings`.
- Tauri tests: `49` passed.
- Frontend typecheck.
- Frontend lint.
- Focused store test.
- Full frontend tests: `373` passed.
- Frontend production build; the existing Vite large chunk warning remains.

## Remaining Before Merge/Production

Completed after local validation:

- PR `#419` merged to `main` as `e244c1c`.
- Post-merge `main` checks passed.
- Render deploy `dep-d8o9t619rddc73cugjvg` for `e244c1c` reached `live`.
- No migration was applied or required.

Production smoke against `https://gitgov-api.onrender.com`:

- `/health=ok`.
- Authenticated `/stats=200`.
- Initial wizard state for `org_name=yohandry10` returned `found=false`.
- `POST /onboarding/first-governed-repo/runs` created run
  `71d55474-0833-4d15-b485-6281792841ae`.
- `validate` returned `providerHealthCount=3` and `stores_secret_values=false`.
- `plan` returned `provider_mutation=false` and `deployment_gate_mode=advisory`.
- `complete` returned `status=completed`, `release_blocking_default=false`,
  `agent_governance_required=false`, and `compliance_claim=false`.
