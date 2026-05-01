# KAN-48 Environment-Scoped Release Governance Policy MVP

Updated: 2026-05-01

## Summary

KAN-48 adds opt-in environment overrides for release governance.

The default remains `record-only`. A customer can keep the base profile non-blocking while explicitly making a selected environment, such as `production`, use `advisory`, `approval-required`, or `quorum-required`.

## Profile Shape

The existing `release_governance` object now accepts `environment_overrides`:

```json
{
  "release_governance": {
    "mode": "record-only",
    "environment": "staging",
    "approval_required": false,
    "enforcement": "disabled",
    "quorum": {
      "enabled": false,
      "rules": []
    },
    "environment_overrides": [
      {
        "mode": "approval-required",
        "environment": "production",
        "approval_required": true,
        "enforcement": "blocking",
        "quorum": {
          "enabled": false,
          "rules": []
        }
      }
    ]
  }
}
```

## Resolution Rule

When GitGov evaluates release governance:

1. Match `release_governance.environment_overrides[*].environment` against the requested release environment.
2. If an override matches, evaluate that override as the effective policy.
3. If no override matches, evaluate the base `release_governance` policy.
4. If no policy exists, use the safe `record-only` default.

## Validation

- `environment_overrides` is optional.
- It must be an array when present.
- It can contain at most 10 entries.
- Each entry must be a release governance policy object.
- Each override must have a non-empty `environment`.
- Override environments are unique case-insensitively.
- Any non-`record-only` base policy or override requires the `formal-approval` module.
- Blocking behavior remains customer-selected only.

## Generated Packs

Adoption packs and workflow template packs now carry the override list.

The release governance gate is generated when either:

- the base policy is non-`record-only`, or
- any environment override is non-`record-only`,

and the `formal-approval` module is enabled.

If the first blocking policy is an override, generated gate templates default to that override environment. Operators can still choose another environment through `workflow_dispatch`.

## Non-Goals

- No default release blocking.
- No database migration.
- No provider API mutation.
- No automatic remote repository workflow installation.
- No secret storage or secret printing.
