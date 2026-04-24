# Quality Gate Policy Matrix Validation Report

Generated (UTC): 2026-04-20 07:59:43
Status: **PASS**

## Context

- GitGov URL: http://127.0.0.1:3001
- Repo: yohandry10/Git-Gov
- Branch: main
- Failing commit (non-green): fd3fb268dc4c34aad9f01aec5e8da3f69017be74
- Green commit: 3a5ddde5c616706e52b5c0ed2ff4e587c6863870
- Policy restore: RESTORED

## Step 1 - quality_gates=warn + failing commit

~~~json
{
    "advisory":  true,
    "allowed":  true,
    "reasons":  [

                ],
    "warnings":  [
                     "Branch \u0027main\u0027 is protected; direct push not allowed",
                     "Sonar quality gate not green for commit fd3fb268dc4c34aad9f01aec5e8da3f69017be74 (job \u0027sonar-governance-test\u0027, status \u0027failure\u0027)"
                 ],
    "evaluated_rules":  [
                            "repo_exists",
                            "policy_exists",
                            "branch_matches_policy",
                            "branch_name_valid",
                            "not_protected_branch",
                            "no_force_push",
                            "conventional_commit",
                            "require_pull_request",
                            "min_approvals_1",
                            "quality_gate_green"
                        ],
    "enforcement_applied":  "warn",
    "violations":  [
                       {
                           "rule":  "not_protected_branch",
                           "category":  "branches",
                           "enforcement":  "warn",
                           "message":  "Branch \u0027main\u0027 is protected; direct push not allowed"
                       },
                       {
                           "rule":  "quality_gate_green",
                           "category":  "quality_gates",
                           "enforcement":  "warn",
                           "message":  "Sonar quality gate not green for commit fd3fb268dc4c34aad9f01aec5e8da3f69017be74 (job \u0027sonar-governance-test\u0027, status \u0027failure\u0027)"
                       }
                   ]
}
~~~

## Step 2 - quality_gates=block + failing commit

~~~json
{
    "advisory":  false,
    "allowed":  false,
    "reasons":  [
                    "Sonar quality gate not green for commit fd3fb268dc4c34aad9f01aec5e8da3f69017be74 (job \u0027sonar-governance-test\u0027, status \u0027failure\u0027)"
                ],
    "warnings":  [
                     "Branch \u0027main\u0027 is protected; direct push not allowed"
                 ],
    "evaluated_rules":  [
                            "repo_exists",
                            "policy_exists",
                            "branch_matches_policy",
                            "branch_name_valid",
                            "not_protected_branch",
                            "no_force_push",
                            "conventional_commit",
                            "require_pull_request",
                            "min_approvals_1",
                            "quality_gate_green"
                        ],
    "enforcement_applied":  "block",
    "violations":  [
                       {
                           "rule":  "not_protected_branch",
                           "category":  "branches",
                           "enforcement":  "warn",
                           "message":  "Branch \u0027main\u0027 is protected; direct push not allowed"
                       },
                       {
                           "rule":  "quality_gate_green",
                           "category":  "quality_gates",
                           "enforcement":  "block",
                           "message":  "Sonar quality gate not green for commit fd3fb268dc4c34aad9f01aec5e8da3f69017be74 (job \u0027sonar-governance-test\u0027, status \u0027failure\u0027)"
                       }
                   ]
}
~~~

## Step 3 - quality_gates=block + green commit

~~~json
{
    "advisory":  false,
    "allowed":  true,
    "reasons":  [

                ],
    "warnings":  [
                     "Branch \u0027main\u0027 is protected; direct push not allowed"
                 ],
    "evaluated_rules":  [
                            "repo_exists",
                            "policy_exists",
                            "branch_matches_policy",
                            "branch_name_valid",
                            "not_protected_branch",
                            "no_force_push",
                            "conventional_commit",
                            "require_pull_request",
                            "min_approvals_1",
                            "quality_gate_green"
                        ],
    "enforcement_applied":  "block",
    "violations":  [
                       {
                           "rule":  "not_protected_branch",
                           "category":  "branches",
                           "enforcement":  "warn",
                           "message":  "Branch \u0027main\u0027 is protected; direct push not allowed"
                       }
                   ]
}
~~~

## Validation errors

- none
