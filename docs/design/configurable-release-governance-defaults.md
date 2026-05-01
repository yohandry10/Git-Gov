# KAN-44 Configurable Release Governance Defaults

Updated: 2026-05-01

## Summary

KAN-44 records an important product decision for future release governance features:

GitGov may support multi-approver quorum and release-blocking enforcement, but those capabilities must be customer-configurable and opt-in. They must not become silent defaults.

In plain language:

- GitGov can store release approval evidence by default.
- GitGov can show whether a release has enough evidence.
- GitGov can warn that a release is missing approval.
- GitGov must not block a customer's release unless that customer deliberately enables a blocking policy.
- GitGov must not require multiple approvers unless that customer deliberately configures a quorum rule.

This protects customer adoption. A team should be able to start with visibility and evidence capture, then move to stricter governance only when they are ready.

## Product Decision

The default release governance mode is `record-only`.

`record-only` means GitGov records approval evidence, risk acceptance, hashes, tickets, and audit metadata. It does not fail a pipeline, block a deployment, or require a release manager to add multiple approvers.

Future stricter modes are allowed only when selected by customer configuration.

The intended customer choices are:

| Mode | Blocks release? | Requires approval? | Requires quorum? | Intended use |
| --- | --- | --- | --- | --- |
| `record-only` | No | No | No | Default onboarding, audit history, low-friction adoption |
| `advisory` | No | Can warn if missing | No by default | Teams want visibility before enforcement |
| `approval-required` | Yes, if configured | Yes | No by default | Teams want one formal approval before production |
| `quorum-required` | Yes, if configured | Yes | Yes | Enterprise release governance with multiple required roles |

The important rule is that `approval-required` and `quorum-required` are not default behavior.

## Why This Matters

Release governance can be powerful, but it can also interrupt delivery.

For a new customer, immediately blocking releases is risky because:

- their workflows may not be fully connected yet.
- their teams may not know which evidence GitGov expects.
- their approval process may still live in Jira, Slack, GitHub reviews, or a change-management tool.
- their first goal may be audit visibility, not enforcement.
- accidental blocking can make adoption feel dangerous.

GitGov should let a customer climb the maturity ladder:

1. See evidence.
2. Save formal approvals.
3. Warn about missing evidence.
4. Require one approval.
5. Require multiple approvers.
6. Block releases only when the customer has intentionally selected that rule.

## Quorum Definition

Quorum means GitGov needs more than one approval before a release is considered fully approved.

Example:

```text
Production release requires:
- 1 engineering approval.
- 1 security approval.
- 1 product or business approval.
```

Another example:

```text
Critical-risk release requires:
- 2 security approvals.
- 1 VP or owner approval.
- risk acceptance expiration within 30 days.
```

This should be configurable per customer, and later per environment or risk level.

Quorum must not mean "GitGov always requires three people." Different companies have different governance models.

## Enforcement Definition

Enforcement means GitGov can make a release gate fail when required governance evidence is missing or invalid.

Example:

```text
Do not deploy to production unless release v1.4.0 has an unexpired approval
bound to the current evidence packet hash.
```

Another example:

```text
Do not deploy if a critical risk exists and there is no accepted-risk approval
with a reason and expiration.
```

Enforcement should be controlled by explicit customer policy.

Without that policy, GitGov can still say "this release is not fully approved", but it must not block the release.

## Default Customer Experience

For a new customer, the default behavior should be:

- Evidence collection is enabled when integrations are connected.
- Release approvals can be created.
- Approval status can be displayed in dashboards and reports.
- Missing approval can be shown as a warning or incomplete status.
- Pipelines do not fail because of missing approval unless enforcement is enabled.
- Multi-approver quorum is not required unless configured.

The first customer experience should feel like:

```text
"GitGov is showing me what evidence I have and what is missing."
```

It should not feel like:

```text
"GitGov suddenly blocked my release because I did not know a hidden rule existed."
```

## Configuration Shape

KAN-45 implements the first profile-level policy shape like this:

```json
{
  "release_governance": {
    "mode": "record-only",
    "environment": "production",
    "approval_required": false,
    "enforcement": "disabled",
    "quorum": {
      "enabled": false,
      "rules": []
    }
  }
}
```

An enforcement customer can explicitly choose:

```json
{
  "release_governance": {
    "mode": "quorum-required",
    "environment": "production",
    "approval_required": true,
    "enforcement": "blocking",
    "quorum": {
      "enabled": true,
      "rules": [
        {
          "role": "engineering",
          "required": 1
        },
        {
          "role": "security",
          "required": 1
        }
      ]
    }
  }
}
```

The profile-level shape is deliberately small. A later version can expand it to per-environment or per-risk-level policy, for example:

```json
{
  "release_governance": {
    "mode": "quorum-required",
    "environments": {
      "production": {
        "approval_required": true,
        "enforcement": "blocking",
        "quorum": {
          "enabled": true,
          "rules": [
            { "role": "engineering", "required": 1 },
            { "role": "security", "required": 1 }
          ]
        }
      }
    }
  }
}
```

The first shape is now used by adoption profile validation, dashboard exports, adoption packs, and workflow template manifests.

## UI Requirements

Any future UI for quorum or enforcement must make the selected behavior obvious.

The UI should clearly distinguish:

- `Record only`: save evidence and approval history, do not block.
- `Advisory`: warn when approval is missing, do not block.
- `Blocking`: fail release gates when configured approval evidence is missing.
- `Quorum`: require specific approver roles or counts.

The UI should avoid vague labels like "strict" without showing the consequence.

Good UI copy:

```text
Blocking is off. GitGov will record approval evidence but will not fail releases.
```

Good UI copy:

```text
Blocking is on for production. A production release will fail if approval evidence is missing or expired.
```

Bad UI copy:

```text
Enterprise governance enabled.
```

The bad copy hides whether releases will actually be blocked.

## Workflow Template Requirements

Generated workflow templates should default to non-blocking behavior unless the adoption profile explicitly requests blocking enforcement.

Default generated workflow behavior:

- collect evidence.
- upload or send governance evidence to GitGov.
- report advisory status.
- avoid failing the customer pipeline solely because release approval is missing.

Blocking workflow behavior is allowed only when customer configuration says so.

When blocking is enabled, the workflow should:

- print a clear non-secret reason for failure.
- identify which release, environment, and policy failed.
- link to the GitGov evidence or approval page when possible.
- avoid printing tokens, secret values, raw Authorization headers, or sensitive payloads.

## API And Backend Requirements

Future backend enforcement should be policy-driven.

The backend should not infer blocking behavior only because approval records exist.

Examples:

- Existing approval records do not imply that all releases require approval.
- Existing high-risk accepted-risk records do not imply that all high-risk releases are automatically blocked.
- Existing quorum records do not imply quorum is enabled for every environment.

There must be an explicit policy source that answers:

- Is release approval required?
- Is enforcement disabled, advisory, or blocking?
- Which environments are affected?
- Which repositories or branches are affected?
- Is quorum enabled?
- Which approver roles or counts are required?
- How long can an approval remain valid?
- Are accepted risks allowed, and for how long?

## Customer Examples

Small team default:

```text
Mode: record-only
Meaning: GitGov records release approvals and evidence. The team can inspect readiness, but GitGov does not block deploys.
```

Growing team:

```text
Mode: advisory
Meaning: GitGov shows warnings when production releases lack formal approval, but deploys continue.
```

Regulated enterprise:

```text
Mode: quorum-required
Meaning: GitGov blocks production release unless required engineering and security approvals exist and are still valid.
```

Emergency hotfix:

```text
Mode: approval-required with accepted-risk
Meaning: GitGov allows a risky release only when someone records an accepted-risk decision with reason and expiration.
```

## Relationship To Existing Work

KAN-37 created the backend release approval record.

KAN-43 added the dashboard create/list flow.

KAN-44 clarifies future defaults:

- KAN-37 and KAN-43 do not enforce release blocking by themselves.
- Future quorum support must be opt-in.
- Future release gate enforcement must be opt-in.
- Adoption profile and workflow template work must preserve non-blocking defaults unless customer policy says otherwise.
- KAN-45 adds the first explicit `release_governance` profile policy so customer intent can travel with adoption packs and generated workflow templates.

## Non-Goals

- This document does not implement quorum.
- This document does not implement release gate enforcement.
- This document does not change current backend behavior.
- This document does not change current workflow templates.
- This document does not make approval mandatory for all GitGov customers.

## Acceptance Criteria For Future Implementation

Future quorum or enforcement work should meet these criteria:

- Default customer policy remains non-blocking.
- Blocking behavior requires explicit configuration.
- Quorum behavior requires explicit configuration.
- Generated workflows show whether they are advisory or blocking.
- Dashboard UI makes the consequence of each mode obvious.
- Policy decisions are auditable.
- Error messages explain missing approval evidence without exposing secrets.
- Customers can start in advisory mode before turning on blocking mode.
