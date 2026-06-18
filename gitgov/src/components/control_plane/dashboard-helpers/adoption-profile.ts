export type AdoptionPolicyPreset = 'audit-only' | 'moderate' | 'strict'
export type AdoptionReleaseGovernanceMode = 'record-only' | 'advisory' | 'approval-required' | 'quorum-required'
export type AdoptionReleaseGovernanceEnforcement = 'disabled' | 'advisory' | 'blocking'
export type AdoptionProvider = 'github' | 'jira' | 'jenkins' | 'sonarqube' | 'render' | 'vercel'
export type AdoptionModule =
  | 'traceability'
  | 'github-evidence'
  | 'release-readiness'
  | 'quality-gates'
  | 'evidence-packets'
  | 'vulnerability-review'
  | 'artifact-monitoring'
  | 'trend-enforcement'
  | 'formal-approval'

export interface AdoptionOption<T extends string> {
  id: T
  label: string
}

export const ADOPTION_PROVIDER_OPTIONS: AdoptionOption<AdoptionProvider>[] = [
  { id: 'github', label: 'GitHub' },
  { id: 'jira', label: 'Jira' },
  { id: 'jenkins', label: 'Jenkins' },
  { id: 'sonarqube', label: 'SonarQube' },
  { id: 'render', label: 'Render' },
  { id: 'vercel', label: 'Vercel' },
]

export const ADOPTION_MODULE_OPTIONS: AdoptionOption<AdoptionModule>[] = [
  { id: 'traceability', label: 'Traceability' },
  { id: 'github-evidence', label: 'GitHub evidence' },
  { id: 'release-readiness', label: 'Release readiness' },
  { id: 'quality-gates', label: 'Quality gates' },
  { id: 'evidence-packets', label: 'Evidence packets' },
  { id: 'vulnerability-review', label: 'Vulnerability review' },
  { id: 'artifact-monitoring', label: 'Artifact monitoring' },
  { id: 'trend-enforcement', label: 'Trend enforcement' },
  { id: 'formal-approval', label: 'Formal approval' },
]

export const ADOPTION_POLICY_PRESET_OPTIONS: AdoptionOption<AdoptionPolicyPreset>[] = [
  { id: 'audit-only', label: 'Audit-only' },
  { id: 'moderate', label: 'Moderate' },
  { id: 'strict', label: 'Strict' },
]

export const ADOPTION_RELEASE_GOVERNANCE_MODE_OPTIONS: AdoptionOption<AdoptionReleaseGovernanceMode>[] = [
  { id: 'record-only', label: 'Record only' },
  { id: 'advisory', label: 'Advisory' },
  { id: 'approval-required', label: 'Approval required' },
  { id: 'quorum-required', label: 'Quorum required' },
]

export interface EnterpriseReleaseGovernanceQuorumRule {
  role: string
  required: number
}

export interface EnterpriseReleaseGovernancePolicy {
  mode: AdoptionReleaseGovernanceMode
  environment: string
  approval_required: boolean
  enforcement: AdoptionReleaseGovernanceEnforcement
  quorum: {
    enabled: boolean
    rules: EnterpriseReleaseGovernanceQuorumRule[]
  }
  environment_overrides?: EnterpriseReleaseGovernancePolicy[]
}

export type EnterpriseReleaseGovernanceEnvironmentSource = 'base' | 'override'

export interface EnterpriseReleaseGovernanceEnvironmentRow {
  source: EnterpriseReleaseGovernanceEnvironmentSource
  environment: string
  mode: AdoptionReleaseGovernanceMode
  approval_required: boolean
  enforcement: AdoptionReleaseGovernanceEnforcement
  quorum_summary: string
  override_index?: number
}

export interface EnterpriseAdoptionProfile {
  customer_name: string
  repository_full_name: string
  default_branch: string
  jira_project_key: string
  policy_preset: AdoptionPolicyPreset
  providers: AdoptionProvider[]
  modules: AdoptionModule[]
  release_governance?: EnterpriseReleaseGovernancePolicy
}

export interface EnterpriseAdoptionWorkflowPlan {
  file: string
  reason: string
}

export interface EnterpriseAdoptionVariable {
  name: string
  scope: string
  purpose: string
  example: string
}

export interface EnterpriseAdoptionSecret {
  name: string
  scope: string
  purpose: string
  value_policy: string
}

export interface EnterpriseAdoptionPolicyRule {
  rule: string
  setting: string
}

export interface EnterpriseAdoptionManualStep {
  step: string
  detail: string
}

export interface EnterpriseAdoptionProductGap {
  gap: string
  detail: string
}

export interface EnterpriseAdoptionPack {
  generated_at: string
  customer_name: string
  repository_full_name: string
  default_branch: string
  jira_project_key: string
  policy_preset: AdoptionPolicyPreset
  release_governance: EnterpriseReleaseGovernancePolicy
  providers: AdoptionProvider[]
  modules: AdoptionModule[]
  workflow_plan: EnterpriseAdoptionWorkflowPlan[]
  variables: EnterpriseAdoptionVariable[]
  secrets: EnterpriseAdoptionSecret[]
  policy_rules: EnterpriseAdoptionPolicyRule[]
  manual_steps: EnterpriseAdoptionManualStep[]
  open_product_gaps: EnterpriseAdoptionProductGap[]
}

export interface EnterpriseWorkflowTemplateSummary {
  file: string
  reason: string
  requires_review_before_install: boolean
}

export interface EnterpriseWorkflowTemplateManifest {
  generated_at: string
  customer_name: string
  repository_full_name: string
  default_branch: string
  jira_project_key: string
  policy_preset: AdoptionPolicyPreset
  release_governance: EnterpriseReleaseGovernancePolicy
  providers: AdoptionProvider[]
  modules: AdoptionModule[]
  workflow_templates: EnterpriseWorkflowTemplateSummary[]
  variables: EnterpriseAdoptionVariable[]
  secrets: EnterpriseAdoptionSecret[]
  manual_steps: EnterpriseAdoptionManualStep[]
  open_product_gaps: EnterpriseAdoptionProductGap[]
  safety: {
    contains_secret_values: false
    mutates_customer_repository: false
    requires_manual_install_review: true
  }
}

export interface EnterpriseWorkflowTemplateFile {
  file: string
  reason: string
  content: string
}

export interface EnterpriseWorkflowTemplatePack {
  generated_at: string
  manifest: EnterpriseWorkflowTemplateManifest
  files: EnterpriseWorkflowTemplateFile[]
  readme: string
}

export interface EnterpriseAdoptionValidation {
  valid: boolean
  errors: string[]
}

export type EnterpriseProviderHealthStatus = 'ready' | 'needs-evidence' | 'needs-config'

export interface EnterpriseProviderHealthEvidence {
  githubEventsTotal?: number
  githubEventTypes?: Record<string, number>
  jiraCommitsWithTicket?: number
  jiraCoveragePercentage?: number
  pipelineRuns7d?: number
  pipelineSuccess7d?: number
  sonarRuns?: number
  sonarSuccessful?: number
  activeRepos?: number
}

export interface EnterpriseProviderHealthCheck {
  provider: AdoptionProvider
  label: string
  status: EnterpriseProviderHealthStatus
  evidence: string
  next_step: string
}

export type EnterpriseProviderSetupStatus = EnterpriseProviderHealthStatus | 'skipped'
export type EnterpriseProviderSetupAction = 'connect' | 'retry' | 'skip' | 'review'
export type EnterpriseProviderSetupTargetKind = 'settings' | 'evidence' | 'action-center' | 'adoption-profile'

export interface EnterpriseProviderSetupTarget {
  kind: EnterpriseProviderSetupTargetKind
  label: string
  to: string
  navigation_only: true
}

export interface EnterpriseProviderSetupStep {
  provider: AdoptionProvider
  label: string
  selected: boolean
  status: EnterpriseProviderSetupStatus
  action: EnterpriseProviderSetupAction
  action_label: string
  reason: string
  validation: string
  target: EnterpriseProviderSetupTarget
}

export interface EnterpriseProviderSetupGuidance {
  selected_count: number
  skipped_count: number
  ready_count: number
  needs_config_count: number
  needs_evidence_count: number
  next_step: EnterpriseProviderSetupStep | null
  steps: EnterpriseProviderSetupStep[]
  safety: {
    contains_secret_values: false
    reads_secret_values: false
    mutates_customer_repository: false
    mutates_provider_state: false
    calls_provider_api: false
    starts_oauth_flow: false
    release_blocking_default: false
    agent_governance_used: false
  }
}

export type EnterpriseOnboardingReadinessStatus = 'ready' | 'needs-action' | 'blocked'
export type EnterpriseOnboardingReadinessStageId =
  | 'profile'
  | 'providers'
  | 'workflow-pack'
  | 'remote-workflows'
  | 'actions-config'
  | 'release-governance'

export interface EnterpriseWorkflowInstallationReadinessInput {
  status?: string
  totals?: {
    workflows_missing?: number
    workflows_different?: number
    variables_missing?: number
    secrets_missing?: number
  }
}

export interface EnterpriseOnboardingReadinessStage {
  id: EnterpriseOnboardingReadinessStageId
  label: string
  status: EnterpriseOnboardingReadinessStatus
  summary: string
  next_action: string
}

export interface EnterpriseOnboardingReadinessReport {
  generated_at: string
  customer_name: string
  repository_full_name: string
  default_branch: string
  jira_project_key: string
  policy_preset: AdoptionPolicyPreset
  status: EnterpriseOnboardingReadinessStatus
  readiness_score: number
  stage_counts: Record<EnterpriseOnboardingReadinessStatus, number>
  release_governance: EnterpriseReleaseGovernancePolicy
  providers: AdoptionProvider[]
  modules: AdoptionModule[]
  stages: EnterpriseOnboardingReadinessStage[]
  next_actions: string[]
  safety: {
    contains_secret_values: false
    reads_secret_values: false
    mutates_customer_repository: false
    mutates_provider_state: false
    release_blocking_default: false
  }
}

export interface EnterpriseOnboardingRemediationAction {
  priority: number
  stage_id: EnterpriseOnboardingReadinessStageId
  stage: string
  status: EnterpriseOnboardingReadinessStatus
  owner: string
  action: string
  reason: string
  validation: string
}

export interface EnterpriseOnboardingConfigurationCommand {
  kind: 'variable' | 'secret'
  name: string
  command: string
  contains_secret_value: false
}

export interface EnterpriseOnboardingRemediationPlan {
  generated_at: string
  customer_name: string
  repository_full_name: string
  default_branch: string
  policy_preset: AdoptionPolicyPreset
  readiness_status: EnterpriseOnboardingReadinessStatus
  readiness_score: number
  remediation_status: EnterpriseOnboardingReadinessStatus
  action_count: number
  actions: EnterpriseOnboardingRemediationAction[]
  github_actions_configuration: {
    source: 'dashboard-adoption-pack'
    variables_count: number
    secrets_count: number
    commands_are_placeholders: true
    commands: EnterpriseOnboardingConfigurationCommand[]
  }
  validation: {
    regenerate_readiness: string
    rerun_provider_checks: string
    rerun_workflow_readiness: string
  }
  safety: {
    contains_secret_values: false
    reads_secret_values: false
    mutates_customer_repository: false
    mutates_provider_state: false
    creates_github_actions_variables: false
    creates_github_actions_secrets: false
    release_blocking_default: false
  }
}

export type EnterpriseOnboardingGuideStepStatus = 'complete' | 'next' | 'todo' | 'blocked'

export interface EnterpriseOnboardingGuideStep {
  order: number
  stage_id: EnterpriseOnboardingReadinessStageId
  label: string
  status: EnterpriseOnboardingGuideStepStatus
  readiness_status: EnterpriseOnboardingReadinessStatus
  owner: string
  summary: string
  action: string
  validation: string
}

export interface EnterpriseOnboardingGuide {
  generated_at: string
  customer_name: string
  repository_full_name: string
  readiness_status: EnterpriseOnboardingReadinessStatus
  readiness_score: number
  completed_steps: number
  total_steps: number
  next_step: EnterpriseOnboardingGuideStep | null
  steps: EnterpriseOnboardingGuideStep[]
  configuration_summary: {
    variable_names: string[]
    secret_names: string[]
    commands_are_placeholders: true
    suggested_commands_count: number
  }
  safety: {
    contains_secret_values: false
    reads_secret_values: false
    mutates_customer_repository: false
    mutates_provider_state: false
    creates_github_actions_variables: false
    creates_github_actions_secrets: false
    release_blocking_default: false
  }
}

export type EnterpriseOnboardingChecklistTrackingStatus = 'open' | 'in-progress' | 'waiting' | 'done'

export interface EnterpriseOnboardingChecklistTrackingItem {
  stage_id: EnterpriseOnboardingReadinessStageId
  status: EnterpriseOnboardingChecklistTrackingStatus
  owner?: string
  note?: string
  external_ref?: string
  target_date?: string
  updated_at?: string
}

export interface EnterpriseOnboardingChecklistTracking {
  version: 1
  items: EnterpriseOnboardingChecklistTrackingItem[]
}

export const ADOPTION_PROVIDER_IDS = ADOPTION_PROVIDER_OPTIONS.map((option) => option.id)
export const ADOPTION_MODULE_IDS = ADOPTION_MODULE_OPTIONS.map((option) => option.id)
export const ADOPTION_RELEASE_GOVERNANCE_MODE_IDS = ADOPTION_RELEASE_GOVERNANCE_MODE_OPTIONS.map((option) => option.id)
export const ONBOARDING_STAGE_IDS: EnterpriseOnboardingReadinessStageId[] = [
  'profile',
  'providers',
  'workflow-pack',
  'remote-workflows',
  'actions-config',
  'release-governance',
]
export const ONBOARDING_TRACKING_STATUSES: EnterpriseOnboardingChecklistTrackingStatus[] = [
  'open',
  'in-progress',
  'waiting',
  'done',
]

function defaultQuorumRules(): EnterpriseReleaseGovernanceQuorumRule[] {
  return [
    { role: 'engineering', required: 1 },
    { role: 'security', required: 1 },
  ]
}

export function buildReleaseGovernancePolicy(
  mode: AdoptionReleaseGovernanceMode,
  environment = 'production',
): EnterpriseReleaseGovernancePolicy {
  const normalizedEnvironment = environment.trim() || 'production'
  if (mode === 'advisory') {
    return {
      mode,
      environment: normalizedEnvironment,
      approval_required: false,
      enforcement: 'advisory',
      quorum: { enabled: false, rules: [] },
      environment_overrides: [],
    }
  }
  if (mode === 'approval-required') {
    return {
      mode,
      environment: normalizedEnvironment,
      approval_required: true,
      enforcement: 'blocking',
      quorum: { enabled: false, rules: [] },
      environment_overrides: [],
    }
  }
  if (mode === 'quorum-required') {
    return {
      mode,
      environment: normalizedEnvironment,
      approval_required: true,
      enforcement: 'blocking',
      quorum: { enabled: true, rules: defaultQuorumRules() },
      environment_overrides: [],
    }
  }
  return {
    mode: 'record-only',
    environment: normalizedEnvironment,
    approval_required: false,
    enforcement: 'disabled',
    quorum: { enabled: false, rules: [] },
    environment_overrides: [],
  }
}

function normalizeReleaseGovernanceRule(rule: EnterpriseReleaseGovernanceQuorumRule): EnterpriseReleaseGovernanceQuorumRule | null {
  const role = typeof rule.role === 'string' ? rule.role.trim().toLowerCase() : ''
  const required = Number.isFinite(rule.required) ? Math.max(1, Math.min(20, Math.trunc(rule.required))) : 1
  if (!role) return null
  return { role, required }
}

function normalizeReleaseGovernancePolicyCore(
  policy?: EnterpriseReleaseGovernancePolicy | null,
): EnterpriseReleaseGovernancePolicy {
  const requestedMode = policy?.mode
  const mode = ADOPTION_RELEASE_GOVERNANCE_MODE_IDS.includes(requestedMode as AdoptionReleaseGovernanceMode)
    ? requestedMode as AdoptionReleaseGovernanceMode
    : 'record-only'
  const normalized = buildReleaseGovernancePolicy(mode, policy?.environment ?? 'production')

  if (mode !== 'quorum-required') return normalized

  const rules = Array.isArray(policy?.quorum?.rules)
    ? policy.quorum.rules
      .map((rule) => normalizeReleaseGovernanceRule(rule))
      .filter((rule): rule is EnterpriseReleaseGovernanceQuorumRule => rule !== null)
    : []

  return {
    ...normalized,
    quorum: {
      enabled: true,
      rules: rules.length > 0 ? rules : normalized.quorum.rules,
    },
  }
}

export function normalizeReleaseGovernancePolicy(
  policy?: EnterpriseReleaseGovernancePolicy | null,
): EnterpriseReleaseGovernancePolicy {
  const normalized = normalizeReleaseGovernancePolicyCore(policy)
  const overrides = Array.isArray(policy?.environment_overrides)
    ? policy.environment_overrides
      .map((override) => normalizeReleaseGovernancePolicyCore(override))
      .filter((override) => override.environment.trim().length > 0)
    : []
  const seen = new Set<string>()
  const uniqueOverrides: EnterpriseReleaseGovernancePolicy[] = []
  for (const override of overrides) {
    const environmentKey = override.environment.trim().toLowerCase()
    if (seen.has(environmentKey)) continue
    seen.add(environmentKey)
    uniqueOverrides.push({
      ...override,
      environment: override.environment.trim(),
      environment_overrides: [],
    })
  }
  return {
    ...normalized,
    environment_overrides: uniqueOverrides,
  }
}

export function releaseGovernancePolicies(policy: EnterpriseReleaseGovernancePolicy): EnterpriseReleaseGovernancePolicy[] {
  return [policy, ...(policy.environment_overrides ?? [])]
}

export function releaseGovernanceRequiresFormalApproval(policy: EnterpriseReleaseGovernancePolicy): boolean {
  return releaseGovernancePolicies(policy).some((candidate) => candidate.mode !== 'record-only')
}

export function releaseGovernanceGateRequired(policy: EnterpriseReleaseGovernancePolicy, modules: AdoptionModule[]): boolean {
  return modules.includes('formal-approval') && releaseGovernanceRequiresFormalApproval(policy)
}

export function releaseGovernanceGatePolicy(policy: EnterpriseReleaseGovernancePolicy): EnterpriseReleaseGovernancePolicy {
  return releaseGovernancePolicies(policy).find((candidate) => (
    candidate.mode === 'approval-required' || candidate.mode === 'quorum-required'
  )) ?? releaseGovernancePolicies(policy).find((candidate) => candidate.mode !== 'record-only') ?? policy
}

export function releaseGovernanceOverrideSummary(policy: EnterpriseReleaseGovernancePolicy): string {
  const overrides = policy.environment_overrides ?? []
  if (overrides.length === 0) return 'none'
  return overrides.map((override) => `${override.environment}:${override.mode}`).join(', ')
}

export function releaseGovernanceQuorumSummary(policy: EnterpriseReleaseGovernancePolicy): string {
  return policy.quorum.enabled
    ? policy.quorum.rules.map((rule) => `${rule.role}:${rule.required}`).join(', ')
    : 'disabled'
}

export function releaseGovernanceModeNeedsFormalApproval(mode: AdoptionReleaseGovernanceMode): boolean {
  return mode !== 'record-only'
}

export function buildReleaseGovernanceEnvironmentRows(
  policy?: EnterpriseReleaseGovernancePolicy | null,
): EnterpriseReleaseGovernanceEnvironmentRow[] {
  const normalized = normalizeReleaseGovernancePolicy(policy)
  return [
    {
      source: 'base',
      environment: normalized.environment,
      mode: normalized.mode,
      approval_required: normalized.approval_required,
      enforcement: normalized.enforcement,
      quorum_summary: releaseGovernanceQuorumSummary(normalized),
    },
    ...(normalized.environment_overrides ?? []).map((override, index) => ({
      source: 'override' as const,
      environment: override.environment,
      mode: override.mode,
      approval_required: override.approval_required,
      enforcement: override.enforcement,
      quorum_summary: releaseGovernanceQuorumSummary(override),
      override_index: index,
    })),
  ]
}

export function nextReleaseGovernanceOverrideEnvironment(
  policy?: EnterpriseReleaseGovernancePolicy | null,
): string {
  const normalized = normalizeReleaseGovernancePolicy(policy)
  const usedEnvironments = new Set(
    [normalized.environment, ...(normalized.environment_overrides ?? []).map((override) => override.environment)]
      .map((environment) => environment.trim().toLowerCase())
      .filter(Boolean),
  )
  return ['production', 'staging', 'development'].find((candidate) => !usedEnvironments.has(candidate))
    ?? `environment-${(normalized.environment_overrides ?? []).length + 1}`
}

export function updateReleaseGovernanceBaseMode(
  policy: EnterpriseReleaseGovernancePolicy | undefined,
  mode: AdoptionReleaseGovernanceMode,
): EnterpriseReleaseGovernancePolicy {
  const current = normalizeReleaseGovernancePolicy(policy)
  return {
    ...buildReleaseGovernancePolicy(mode, current.environment),
    environment_overrides: current.environment_overrides ?? [],
  }
}

export function updateReleaseGovernanceBaseEnvironment(
  policy: EnterpriseReleaseGovernancePolicy | undefined,
  environment: string,
): EnterpriseReleaseGovernancePolicy {
  const current = normalizeReleaseGovernancePolicy(policy)
  return {
    ...current,
    environment: environment.trim() || 'production',
  }
}

export function addReleaseGovernanceEnvironmentOverride(
  policy: EnterpriseReleaseGovernancePolicy | undefined,
): EnterpriseReleaseGovernancePolicy {
  const current = normalizeReleaseGovernancePolicy(policy)
  const environment = nextReleaseGovernanceOverrideEnvironment(current)
  return {
    ...current,
    environment_overrides: [
      ...(current.environment_overrides ?? []),
      buildReleaseGovernancePolicy('approval-required', environment),
    ],
  }
}

export function updateReleaseGovernanceEnvironmentOverrideEnvironment(
  policy: EnterpriseReleaseGovernancePolicy | undefined,
  index: number,
  environment: string,
): EnterpriseReleaseGovernancePolicy {
  const current = normalizeReleaseGovernancePolicy(policy)
  return {
    ...current,
    environment_overrides: (current.environment_overrides ?? []).map((override, overrideIndex) => (
      overrideIndex === index
        ? { ...override, environment: environment.trim() || 'production' }
        : override
    )),
  }
}

export function updateReleaseGovernanceEnvironmentOverrideMode(
  policy: EnterpriseReleaseGovernancePolicy | undefined,
  index: number,
  mode: AdoptionReleaseGovernanceMode,
): EnterpriseReleaseGovernancePolicy {
  const current = normalizeReleaseGovernancePolicy(policy)
  return {
    ...current,
    environment_overrides: (current.environment_overrides ?? []).map((override, overrideIndex) => (
      overrideIndex === index
        ? buildReleaseGovernancePolicy(mode, override.environment || 'production')
        : override
    )),
  }
}

export function removeReleaseGovernanceEnvironmentOverride(
  policy: EnterpriseReleaseGovernancePolicy | undefined,
  index: number,
): EnterpriseReleaseGovernancePolicy {
  const current = normalizeReleaseGovernancePolicy(policy)
  return {
    ...current,
    environment_overrides: (current.environment_overrides ?? []).filter((_, overrideIndex) => overrideIndex !== index),
  }
}

export function normalizeEnterpriseAdoptionProfile(profile: EnterpriseAdoptionProfile): EnterpriseAdoptionProfile {
  return {
    customer_name: profile.customer_name ?? DEFAULT_ENTERPRISE_ADOPTION_PROFILE.customer_name,
    repository_full_name: profile.repository_full_name ?? DEFAULT_ENTERPRISE_ADOPTION_PROFILE.repository_full_name,
    default_branch: profile.default_branch ?? DEFAULT_ENTERPRISE_ADOPTION_PROFILE.default_branch,
    jira_project_key: profile.jira_project_key ?? DEFAULT_ENTERPRISE_ADOPTION_PROFILE.jira_project_key,
    policy_preset: profile.policy_preset ?? DEFAULT_ENTERPRISE_ADOPTION_PROFILE.policy_preset,
    providers: Array.isArray(profile.providers) ? [...profile.providers] : [...DEFAULT_ENTERPRISE_ADOPTION_PROFILE.providers],
    modules: Array.isArray(profile.modules) ? [...profile.modules] : [...DEFAULT_ENTERPRISE_ADOPTION_PROFILE.modules],
    release_governance: normalizeReleaseGovernancePolicy(profile.release_governance),
  }
}

export const DEFAULT_ENTERPRISE_ADOPTION_PROFILE: EnterpriseAdoptionProfile = {
  customer_name: 'ExampleCo',
  repository_full_name: 'example-org/example-repo',
  default_branch: 'main',
  jira_project_key: 'EX',
  policy_preset: 'moderate',
  release_governance: buildReleaseGovernancePolicy('record-only'),
  providers: ['github', 'jira', 'jenkins', 'sonarqube'],
  modules: [
    'traceability',
    'github-evidence',
    'release-readiness',
    'quality-gates',
    'evidence-packets',
    'vulnerability-review',
    'artifact-monitoring',
    'trend-enforcement',
  ],
}

export function uniqueKnownValues<T extends string>(values: readonly T[], knownValues: readonly T[]): T[] {
  const known = new Set(knownValues)
  const result: T[] = []
  for (const value of values) {
    if (!known.has(value)) continue
    if (!result.includes(value)) result.push(value)
  }
  return result
}

export function addUniqueByKey<T extends Record<K, string>, K extends keyof T>(
  items: T[],
  item: T,
  key: K,
) {
  if (items.some((existing) => existing[key] === item[key])) return
  items.push(item)
}

export function adoptionReadinessTarget(preset: AdoptionPolicyPreset): number {
  if (preset === 'audit-only') return 0
  if (preset === 'strict') return 85
  return 75
}

export function criticalHighPolicy(preset: AdoptionPolicyPreset): string {
  if (preset === 'audit-only') return 'report-only'
  if (preset === 'strict') return 'block reachable critical/high vulnerabilities and require documented medium-risk acceptance'
  return 'block reachable critical/high vulnerabilities'
}

export function validateEnterpriseAdoptionProfile(profile: EnterpriseAdoptionProfile): EnterpriseAdoptionValidation {
  const errors: string[] = []
  const repo = profile.repository_full_name.trim()
  const branch = profile.default_branch.trim()
  const jiraKey = profile.jira_project_key.trim()
  const releaseGovernance = normalizeReleaseGovernancePolicy(profile.release_governance)

  if (!profile.customer_name.trim()) errors.push('Customer name is required.')
  if (!repo) {
    errors.push('Repository is required.')
  } else if (!/^[^/\s]+\/[^/\s]+$/.test(repo)) {
    errors.push('Repository must look like owner/repo.')
  }
  if (!branch) errors.push('Default branch is required.')
  if (profile.modules.includes('traceability') && !jiraKey) {
    errors.push('Jira project key is required when traceability is selected.')
  }
  if (jiraKey && !/^[A-Z][A-Z0-9]{1,15}$/.test(jiraKey)) {
    errors.push('Jira project key should be uppercase letters/numbers, like KAN.')
  }
  if (profile.providers.length === 0) errors.push('Select at least one provider.')
  if (profile.modules.length === 0) errors.push('Select at least one module.')
  if (!releaseGovernance.environment.trim()) {
    errors.push('Release governance environment is required.')
  }
  const rawEnvironmentOverrides = Array.isArray(profile.release_governance?.environment_overrides)
    ? profile.release_governance.environment_overrides
    : []
  for (const override of releaseGovernance.environment_overrides ?? []) {
    if (!override.environment.trim()) {
      errors.push('Release governance override environment is required.')
    }
  }
  const duplicateOverrideEnvironments = new Set<string>()
  for (const override of rawEnvironmentOverrides) {
    const environmentKey = override.environment.trim().toLowerCase()
    if (!environmentKey) continue
    if (duplicateOverrideEnvironments.has(environmentKey)) {
      errors.push(`Release governance override environment '${override.environment}' is duplicated.`)
    }
    duplicateOverrideEnvironments.add(environmentKey)
  }
  if (releaseGovernanceRequiresFormalApproval(releaseGovernance) && !profile.modules.includes('formal-approval')) {
    errors.push('Enable the Formal approval module before choosing advisory, approval-required, or quorum-required release governance.')
  }
  if (releaseGovernancePolicies(releaseGovernance).some((policy) => policy.mode === 'quorum-required' && policy.quorum.rules.length === 0)) {
    errors.push('Quorum-required release governance needs at least one approver role.')
  }

  return { valid: errors.length === 0, errors }
}
