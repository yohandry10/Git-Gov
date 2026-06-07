# KAN-69 Enterprise Action Center Guided UX

Date: 2026-06-07
Ticket: `KAN-69`

## Summary

KAN-69 adds a dedicated Action Center route to the GitGov desktop app.

The implementation packages existing capabilities into a guided, deterministic surface instead of adding another standalone hardening, monitor, report, or provider feature.

## Product Outcome

The Action Center now gives the operator:

- a selected goal: onboarding, release prep, or evidence export.
- a presentation lens: founder, developer, executive, platform, or auditor.
- one primary next action.
- supporting evidence.
- permission context.
- related alternatives.
- deep links to existing GitGov workflows.

The recommendation is advisory. Operators can ignore it, switch goals, open alternatives, or use the existing Workspace dashboard manually.

## Implementation

Files added:

- `gitgov/src/pages/ActionCenterPage.tsx`.
- `gitgov/src/components/action_center/ActionCenterWorkspace.tsx`.
- `gitgov/src/components/action_center/action-center-helpers.ts`.
- `gitgov/src/test/components/action-center-helpers.test.ts`.
- `docs/design/enterprise-action-center-guided-ux.md`.
- `docs/reports/enterprise-action-center-guided-ux-2026-06-07.md`.

Files updated:

- `gitgov/src/router.tsx`.
- `gitgov/src/components/layout/Sidebar.tsx`.
- `gitgov/src/components/layout/MainLayout.tsx`.
- `gitgov/src/components/control_plane/EnterpriseAdoptionPanel.tsx`.
- `gitgov/src/components/control_plane/EvidencePacketPanel.tsx`.
- `gitgov/src/components/control_plane/ReleaseApprovalPanel.tsx`.
- `gitgov/src/components/control_plane/GovernanceCopilotPanel.tsx`.
- `docs/design/enterprise-action-center-ux-focus.md`.
- `docs/design/enterprise-self-service-and-ai-copilot-roadmap.md`.
- `docs/AGENT_PUBLIC_CONTEXT.md`.
- `docs/IMPLEMENTATION_STATUS.md`.
- `docs/CURRENT_CONTEXT.md`.

## Rules Implemented

The helper ranks the primary action through explicit rules:

- invalid adoption profile before downstream work.
- provider configuration before provider evidence.
- provider evidence before onboarding completion.
- pipeline health before release approval.
- Jira traceability before release evidence confidence.
- Evidence Packet review before release decision recording.
- readiness/remediation export when evidence export is requested before onboarding is ready.

## Safety

- No provider credentials are read.
- No `.env` files are read.
- No secrets are printed or exported.
- No provider state is mutated.
- No customer repository is mutated.
- No release gate defaults are changed.
- AI is linked only as an explanation surface; the Action Center recommendation remains deterministic.

## Validation

Local validation:

- `npm --prefix gitgov run typecheck` - passed.
- `npm --prefix gitgov run test -- --run src/test/components/action-center-helpers.test.ts` - passed, `6` tests.
- `npm --prefix gitgov run test -- --run` - passed, `302` tests in `26` files.
- `npm --prefix gitgov run lint` - passed.
- `npm --prefix gitgov run build` - passed. Vite reported the pre-existing large chunk warning; no KAN-69 build failure.
- `git diff --check` - passed.
- `.\scripts\security\publication_guard.ps1` - passed.
- Browser smoke at `http://127.0.0.1:5173/action-center` - Vite served the app and console had no errors. The browser showed the expected `Requiere GitGov Desktop` gate because the full desktop UI requires Tauri runtime APIs.
