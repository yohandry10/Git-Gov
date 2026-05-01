# KAN-45 Release Governance Profile Policy MVP

Updated: 2026-05-01

## Summary

KAN-45 turns the KAN-44 product decision into customer configuration.

GitGov now has an explicit `release_governance` policy in the enterprise adoption profile. The default is still `record-only`, which means GitGov records release approval evidence but does not block releases and does not require multiple approvers.

Stricter modes exist only when the customer selects them:

| Mode | Enforcement | Requires approval | Requires quorum | Default |
| --- | --- | --- | --- | --- |
| `record-only` | `disabled` | No | No | Yes |
| `advisory` | `advisory` | No | No | No |
| `approval-required` | `blocking` | Yes | No | No |
| `quorum-required` | `blocking` | Yes | Yes | No |

This keeps onboarding low-friction while giving enterprise customers a clear path to stronger governance.

## Profile Shape

The profile field is intentionally small:

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

For quorum opt-in:

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
        { "role": "engineering", "required": 1 },
        { "role": "security", "required": 1 }
      ]
    }
  }
}
```

## Implementation Scope

- Dashboard: Enterprise Adoption profile editor exposes release governance mode and environment.
- Dashboard exports: adoption pack JSON and workflow template pack JSON include `release_governance`.
- CLI adoption pack: `scripts/control-plane/generate_enterprise_adoption_pack.ps1` includes the policy in Markdown and JSON.
- CLI workflow templates: `scripts/control-plane/generate_enterprise_workflow_templates.ps1` includes the policy in README and manifest.
- Backend persistence validation: `PUT /enterprise/adoption-profile` validates `release_governance` before saving.
- Example profile: `docs/examples/enterprise-adoption-profile.example.json` documents the safe default.

## Validation Rules

Backend and local helpers enforce these rules:

- Missing `release_governance` is treated as compatible with the safe `record-only` default.
- `record-only` must use `disabled` enforcement.
- `record-only` cannot require approval.
- `record-only` cannot enable quorum.
- Any non-`record-only` mode requires the `formal-approval` module.
- `approval-required` must require approval and use `blocking` enforcement.
- `quorum-required` must require approval, use `blocking` enforcement, enable quorum, and include at least one quorum rule.
- Quorum rule `required` values must stay bounded.

## Product Behavior

KAN-45 does not make release blocking active by itself.

It only makes the customer's intended policy explicit and portable across:

- stored adoption profile.
- dashboard state.
- generated adoption pack.
- generated workflow template pack.
- backend validation.

KAN-46 adds the first evaluator that reads this policy and compares it with formal release approval evidence. The evaluator returns `recorded`, `advisory-warning`, `approved`, `would-block`, or `blocked`, but it still does not mutate customer workflows or block deployments by itself.

Actual release gate enforcement remains a customer-selected feature. A future workflow gate can consume the KAN-46 evaluator result and fail only when the customer explicitly configured blocking enforcement.

## Secret Safety

The profile stores policy intent only. It does not store provider tokens, `.env` values, Authorization headers, webhook secrets, or raw customer credentials.

Generated packs mention variable and secret names only.

## Non-Goals

- No production release gate is blocked by this change.
- No customer repository is mutated remotely.
- No provider secret is read or written.
- Quorum approval evaluation is added later in KAN-46 without changing this default.
- No workflow auto-install is triggered.
