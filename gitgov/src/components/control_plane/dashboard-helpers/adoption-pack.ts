import {
  ADOPTION_MODULE_IDS,
  ADOPTION_PROVIDER_IDS,
  addUniqueByKey,
  adoptionReadinessTarget,
  criticalHighPolicy,
  normalizeReleaseGovernancePolicy,
  releaseGovernanceGateRequired,
  releaseGovernanceOverrideSummary,
  releaseGovernanceQuorumSummary,
  uniqueKnownValues,
  type EnterpriseAdoptionManualStep,
  type EnterpriseAdoptionPack,
  type EnterpriseAdoptionPolicyRule,
  type EnterpriseAdoptionProductGap,
  type EnterpriseAdoptionProfile,
  type EnterpriseAdoptionSecret,
  type EnterpriseAdoptionVariable,
  type EnterpriseAdoptionWorkflowPlan,
} from './adoption-profile'

export function buildEnterpriseAdoptionPack(
  profile: EnterpriseAdoptionProfile,
  generatedAt = new Date().toISOString(),
): EnterpriseAdoptionPack {
  const providers = uniqueKnownValues(profile.providers, ADOPTION_PROVIDER_IDS)
  const modules = uniqueKnownValues(profile.modules, ADOPTION_MODULE_IDS)
  const readinessTarget = adoptionReadinessTarget(profile.policy_preset)
  const trendEnforcementRequired =
    profile.policy_preset === 'strict' || modules.includes('trend-enforcement')
  const prReviewRequired = profile.policy_preset === 'strict'
  const freshArtifactRequired = profile.policy_preset !== 'audit-only'
  const jiraKey = profile.jira_project_key.trim()
  const ticketPrefix = jiraKey || 'KAN'
  const releaseGovernance = normalizeReleaseGovernancePolicy(profile.release_governance)
  const releaseGovernanceGateNeeded = releaseGovernanceGateRequired(releaseGovernance, modules)

  const workflowPlan: EnterpriseAdoptionWorkflowPlan[] = []
  const variables: EnterpriseAdoptionVariable[] = []
  const secrets: EnterpriseAdoptionSecret[] = []
  const manualSteps: EnterpriseAdoptionManualStep[] = []
  const openProductGaps: EnterpriseAdoptionProductGap[] = []

  addUniqueByKey(workflowPlan, { file: '.github/workflows/ci.yml', reason: 'core build, lint, typecheck, and tests' }, 'file')
  addUniqueByKey(workflowPlan, { file: '.github/workflows/secret-scan.yml', reason: 'publication guard, secret policy, and traceability hygiene' }, 'file')

  if (modules.includes('traceability')) {
    addUniqueByKey(workflowPlan, { file: '.github/workflows/public-naming-guard.yml', reason: 'public naming and repository hygiene' }, 'file')
    addUniqueByKey(manualSteps, {
      step: 'Set Jira-style ticket ID policy',
      detail: `Require branch names, PR titles, and commit messages to include ticket IDs such as ${ticketPrefix}-123.`,
    }, 'step')
  }

  if (modules.includes('github-evidence')) {
    addUniqueByKey(workflowPlan, { file: '.github/workflows/github-evidence-report.yml', reason: 'GitHub evidence executive report' }, 'file')
    addUniqueByKey(workflowPlan, { file: '.github/workflows/github-evidence-artifact-monitor.yml', reason: 'GitHub evidence artifact freshness' }, 'file')
    addUniqueByKey(workflowPlan, { file: '.github/workflows/github-evidence-trend-report.yml', reason: 'GitHub evidence trend history' }, 'file')
  }

  if (modules.includes('release-readiness')) {
    addUniqueByKey(workflowPlan, { file: '.github/workflows/release-readiness-gate.yml', reason: 'release readiness score and evidence artifact' }, 'file')
  }

  if (releaseGovernanceGateNeeded) {
    addUniqueByKey(workflowPlan, { file: '.github/workflows/release-governance-gate.yml', reason: 'optional release governance policy evaluator for customer-selected enforcement' }, 'file')
    if (modules.includes('artifact-monitoring')) {
      addUniqueByKey(workflowPlan, { file: '.github/workflows/release-governance-gate-artifact-monitor.yml', reason: 'release governance gate artifact freshness after explicit enforcement opt-in' }, 'file')
    }
    addUniqueByKey(variables, { name: 'GITGOV_URL', scope: 'GitHub Actions variable', purpose: 'GitGov API base URL', example: 'https://gitgov-api.example.com' }, 'name')
    addUniqueByKey(secrets, { name: 'GITGOV_API_KEY', scope: 'GitHub Actions secret', purpose: 'GitGov API authentication for release governance evaluation', value_policy: 'secret value only, never committed' }, 'name')
  }

  if (modules.includes('quality-gates')) {
    addUniqueByKey(workflowPlan, { file: '.github/workflows/quality-gate-policy-matrix.yml', reason: 'quality gate warn/block matrix validation' }, 'file')
    if (providers.includes('sonarqube')) {
      addUniqueByKey(workflowPlan, { file: '.github/workflows/sonar-governance.yml', reason: 'SonarQube governance telemetry when reachable' }, 'file')
    }
  }

  if (modules.includes('vulnerability-review')) {
    addUniqueByKey(workflowPlan, { file: '.github/workflows/product-vulnerability-review.yml', reason: 'product vulnerability review evidence' }, 'file')
    addUniqueByKey(workflowPlan, { file: '.github/workflows/product-vulnerability-review-trend-report.yml', reason: 'product vulnerability review trend report' }, 'file')
  }

  if (modules.includes('artifact-monitoring')) {
    addUniqueByKey(workflowPlan, { file: '.github/workflows/product-vulnerability-review-artifact-monitor.yml', reason: 'product vulnerability review artifact freshness' }, 'file')
  }

  if (trendEnforcementRequired) {
    addUniqueByKey(workflowPlan, { file: '.github/workflows/product-vulnerability-review-trend-enforcement.yml', reason: 'block regressions in vulnerability review trend' }, 'file')
  }

  if (modules.includes('formal-approval')) {
    addUniqueByKey(manualSteps, {
      step: 'Review release approval policy',
      detail: releaseGovernance.mode === 'record-only'
        ? `Default record-only mode stores release approval evidence and does not block customer releases. Environment overrides: ${releaseGovernanceOverrideSummary(releaseGovernance)}.`
        : `Customer selected ${releaseGovernance.mode} for ${releaseGovernance.environment}; review this explicit opt-in policy before installing any blocking workflow.`,
    }, 'step')
    if (releaseGovernanceGateNeeded) {
      addUniqueByKey(manualSteps, {
        step: 'Validate release governance gate manually',
        detail: 'Run release-governance-gate.yml with workflow_dispatch before using it as a blocking release check. Default enforcement follows the selected release_governance mode or matching environment override.',
      }, 'step')
      if (modules.includes('artifact-monitoring')) {
        addUniqueByKey(manualSteps, {
          step: 'Validate release governance gate artifact',
          detail: 'Run release-governance-gate-artifact-monitor.yml after at least one successful gate run to confirm the release governance evidence artifact exists and is still fresh.',
        }, 'step')
      }
    }
  }

  if (providers.includes('github')) {
    addUniqueByKey(variables, { name: 'GITGOV_URL', scope: 'GitHub Actions variable', purpose: 'GitGov API base URL', example: 'https://gitgov-api.example.com' }, 'name')
    addUniqueByKey(secrets, { name: 'GITGOV_API_KEY', scope: 'GitHub Actions secret', purpose: 'GitGov API authentication for workflow telemetry', value_policy: 'secret value only, never committed' }, 'name')
    addUniqueByKey(manualSteps, { step: 'Install GitHub webhook', detail: 'Configure signed GitHub webhook events for push, pull_request, pull_request_review, comments, checks, and status.' }, 'step')
  }

  if (providers.includes('jira')) {
    addUniqueByKey(manualSteps, { step: 'Connect Jira project', detail: 'Set Jira project key, enable signed Jira webhook, and verify ticket ingestion.' }, 'step')
  }

  if (providers.includes('jenkins')) {
    addUniqueByKey(manualSteps, { step: 'Connect Jenkins', detail: 'Configure authenticated Jenkins API access and GitGov telemetry publishing from pipeline jobs.' }, 'step')
  }

  if (providers.includes('sonarqube')) {
    addUniqueByKey(variables, { name: 'SONAR_HOST_URL', scope: 'GitHub Actions variable', purpose: 'SonarQube endpoint when reachable by runner', example: 'https://sonarqube.example.com' }, 'name')
    addUniqueByKey(variables, { name: 'SONAR_PROJECT_KEY', scope: 'GitHub Actions variable', purpose: 'SonarQube project key', example: 'example_org_example_repo' }, 'name')
    addUniqueByKey(secrets, { name: 'SONAR_TOKEN', scope: 'GitHub Actions secret', purpose: 'Optional SonarQube API token when runner can reach SonarQube', value_policy: 'secret value only, never committed' }, 'name')
    addUniqueByKey(manualSteps, { step: 'Validate Sonar runtime', detail: 'Use reachable SonarQube for customer environments; skip GitHub-hosted scans when Sonar is private/local.' }, 'step')
  }

  if (providers.includes('render')) {
    addUniqueByKey(manualSteps, { step: 'Connect deployment provider', detail: 'Record deployment health and service metadata without storing provider tokens in the repository.' }, 'step')
  }

  if (providers.includes('vercel')) {
    addUniqueByKey(manualSteps, { step: 'Connect Vercel deployment evidence', detail: 'Use deployment status and preview evidence as governance context when the customer deploys on Vercel.' }, 'step')
  }

  const policyRules: EnterpriseAdoptionPolicyRule[] = [
    { rule: 'Ticket traceability', setting: modules.includes('traceability') ? 'required' : 'optional' },
    { rule: 'Release readiness target', setting: String(readinessTarget) },
    { rule: 'Release approval governance', setting: releaseGovernance.mode },
    { rule: 'Release approval enforcement', setting: releaseGovernance.enforcement },
    { rule: 'Release governance gate', setting: releaseGovernanceGateNeeded ? 'manual opt-in workflow' : 'not generated for record-only' },
    { rule: 'Release governance artifact monitor', setting: releaseGovernanceGateNeeded && modules.includes('artifact-monitoring') ? 'manual opt-in freshness check' : 'not generated by default' },
    { rule: 'Release governance environment overrides', setting: releaseGovernanceOverrideSummary(releaseGovernance) },
    {
      rule: 'Release approval quorum',
      setting: releaseGovernanceQuorumSummary(releaseGovernance),
    },
    { rule: 'Critical/high vulnerability policy', setting: criticalHighPolicy(profile.policy_preset) },
    { rule: 'PR review evidence', setting: prReviewRequired ? 'required' : 'recommended' },
    { rule: 'Fresh evidence artifacts', setting: freshArtifactRequired ? 'required' : 'report-only' },
    { rule: 'Vulnerability trend enforcement', setting: trendEnforcementRequired ? 'enabled' : 'informational' },
  ]

  return {
    generated_at: generatedAt,
    customer_name: profile.customer_name.trim(),
    repository_full_name: profile.repository_full_name.trim(),
    default_branch: profile.default_branch.trim() || 'main',
    jira_project_key: jiraKey,
    policy_preset: profile.policy_preset,
    release_governance: releaseGovernance,
    providers,
    modules,
    workflow_plan: workflowPlan,
    variables,
    secrets,
    policy_rules: policyRules,
    manual_steps: manualSteps,
    open_product_gaps: openProductGaps,
  }
}
