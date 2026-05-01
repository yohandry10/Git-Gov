import type { CombinedEvent } from '@/lib/types'

export function readDetailString(log: CombinedEvent, key: string): string | null {
  const value = log.details?.[key]
  if (typeof value === 'string' && value.trim().length > 0) return value
  const metadata = log.details && typeof log.details === 'object' ? (log.details['metadata'] as Record<string, unknown> | undefined) : undefined
  const nested = metadata?.[key]
  if (typeof nested === 'string' && nested.trim().length > 0) return nested
  const legacyDetails = log.details && typeof log.details === 'object' ? (log.details['legacy_details'] as Record<string, unknown> | undefined) : undefined
  const legacyMetadata = legacyDetails && typeof legacyDetails === 'object' ? (legacyDetails['metadata'] as Record<string, unknown> | undefined) : undefined
  const nestedLegacy = legacyMetadata?.[key]
  return typeof nestedLegacy === 'string' && nestedLegacy.trim().length > 0 ? nestedLegacy : null
}

export function getLogDetailPreview(log: CombinedEvent): string | null {
  if (log.event_type === 'commit') return readDetailString(log, 'commit_message')
  if (log.status === 'failed' || log.status === 'blocked') return readDetailString(log, 'reason')
  return null
}

export function getShortCommitSha(log: CombinedEvent): string | null {
  const sha = readDetailString(log, 'commit_sha')
  return sha ? sha.slice(0, 7) : null
}

export function extractTicketIdsFromCommitLog(log: CombinedEvent): string[] {
  const values = [readDetailString(log, 'commit_message'), log.branch ?? null].filter((v): v is string => typeof v === 'string' && v.trim().length > 0)
  const regex = /\b([A-Z][A-Z0-9]{1,15}-\d{1,9})\b/g
  const result: string[] = []
  const seen = new Set<string>()
  for (const value of values) {
    let match: RegExpExecArray | null
    regex.lastIndex = 0
    while ((match = regex.exec(value)) !== null) {
      const ticket = match[1].toUpperCase()
      if (!seen.has(ticket)) { seen.add(ticket); result.push(ticket) }
    }
  }
  return result
}

export function formatDurationMs(ms?: number): string {
  if (!ms || ms <= 0) return '-'
  const totalSeconds = Math.floor(ms / 1000)
  const minutes = Math.floor(totalSeconds / 60)
  const seconds = totalSeconds % 60
  if (minutes <= 0) return `${seconds}s`
  return `${minutes}m ${seconds}s`
}

export interface OperationalPipelineEvidence {
  commit_created_at: number
  pipeline?: {
    pipeline_event_id?: string | null
    pipeline_id?: string | null
    job_name: string
    status: string
    ingested_at: number
  } | null
}

export interface OperationalEvidenceMetrics {
  timeToEvidenceMs: number | null
  timeToEvidenceSamples: number
  mttrMs: number | null
  mttrSamples: number
}

const SUCCESS_PIPELINE_STATUSES = new Set(['success', 'ok', 'passed'])
const RECOVERABLE_FAILURE_PIPELINE_STATUSES = new Set([
  'failure',
  'failed',
  'error',
  'unstable',
  'aborted',
  'cancelled',
  'canceled',
])

function normalizePipelineStatus(status?: string | null): string {
  return (status ?? '').trim().toLowerCase()
}

function isSuccessPipelineStatus(status?: string | null): boolean {
  return SUCCESS_PIPELINE_STATUSES.has(normalizePipelineStatus(status))
}

function isRecoverableFailurePipelineStatus(status?: string | null): boolean {
  return RECOVERABLE_FAILURE_PIPELINE_STATUSES.has(normalizePipelineStatus(status))
}

function averageMs(values: number[]): number | null {
  if (values.length === 0) return null
  return Math.round(values.reduce((total, value) => total + value, 0) / values.length)
}

function pipelineEvidenceKey(evidence: OperationalPipelineEvidence, index: number): string {
  const pipeline = evidence.pipeline
  return (
    pipeline?.pipeline_event_id ||
    pipeline?.pipeline_id ||
    `${pipeline?.job_name ?? 'unknown'}:${pipeline?.ingested_at ?? index}:${pipeline?.status ?? 'unknown'}`
  )
}

export function formatOperationalMetricDuration(ms: number | null, samples: number): string {
  if (samples <= 0 || ms === null) return 'N/A'
  if (ms <= 0) return '0s'
  return formatDurationMs(ms)
}

export function buildOperationalEvidenceMetrics(
  correlations: OperationalPipelineEvidence[],
): OperationalEvidenceMetrics {
  const seenPipelines = new Set<string>()
  const pipelines = correlations.flatMap((entry, index) => {
    const pipeline = entry.pipeline
    if (!pipeline) return []
    if (!Number.isFinite(entry.commit_created_at) || !Number.isFinite(pipeline.ingested_at)) return []

    const key = pipelineEvidenceKey(entry, index)
    if (seenPipelines.has(key)) return []
    seenPipelines.add(key)

    return [{
      commitCreatedAt: entry.commit_created_at,
      ingestedAt: pipeline.ingested_at,
      jobName: pipeline.job_name.trim() || 'unknown',
      status: normalizePipelineStatus(pipeline.status),
    }]
  })

  const timeToEvidenceDeltas = pipelines
    .map((pipeline) => pipeline.ingestedAt - pipeline.commitCreatedAt)
    .filter((delta) => Number.isFinite(delta) && delta >= 0)

  const pipelinesByJob = new Map<string, typeof pipelines>()
  for (const pipeline of pipelines) {
    const jobKey = pipeline.jobName.toLowerCase()
    const jobRuns = pipelinesByJob.get(jobKey) ?? []
    jobRuns.push(pipeline)
    pipelinesByJob.set(jobKey, jobRuns)
  }

  const recoveryDeltas: number[] = []
  for (const jobRuns of pipelinesByJob.values()) {
    const orderedRuns = [...jobRuns].sort((a, b) => a.ingestedAt - b.ingestedAt)
    for (let index = 0; index < orderedRuns.length; index++) {
      const run = orderedRuns[index]
      if (!isRecoverableFailurePipelineStatus(run.status)) continue
      const recovery = orderedRuns
        .slice(index + 1)
        .find((nextRun) => nextRun.ingestedAt >= run.ingestedAt && isSuccessPipelineStatus(nextRun.status))
      if (recovery) {
        recoveryDeltas.push(recovery.ingestedAt - run.ingestedAt)
      }
    }
  }

  return {
    timeToEvidenceMs: averageMs(timeToEvidenceDeltas),
    timeToEvidenceSamples: timeToEvidenceDeltas.length,
    mttrMs: averageMs(recoveryDeltas),
    mttrSamples: recoveryDeltas.length,
  }
}

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

const ADOPTION_PROVIDER_IDS = ADOPTION_PROVIDER_OPTIONS.map((option) => option.id)
const ADOPTION_MODULE_IDS = ADOPTION_MODULE_OPTIONS.map((option) => option.id)
const ADOPTION_RELEASE_GOVERNANCE_MODE_IDS = ADOPTION_RELEASE_GOVERNANCE_MODE_OPTIONS.map((option) => option.id)

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
    }
  }
  if (mode === 'approval-required') {
    return {
      mode,
      environment: normalizedEnvironment,
      approval_required: true,
      enforcement: 'blocking',
      quorum: { enabled: false, rules: [] },
    }
  }
  if (mode === 'quorum-required') {
    return {
      mode,
      environment: normalizedEnvironment,
      approval_required: true,
      enforcement: 'blocking',
      quorum: { enabled: true, rules: defaultQuorumRules() },
    }
  }
  return {
    mode: 'record-only',
    environment: normalizedEnvironment,
    approval_required: false,
    enforcement: 'disabled',
    quorum: { enabled: false, rules: [] },
  }
}

function normalizeReleaseGovernanceRule(rule: EnterpriseReleaseGovernanceQuorumRule): EnterpriseReleaseGovernanceQuorumRule | null {
  const role = typeof rule.role === 'string' ? rule.role.trim().toLowerCase() : ''
  const required = Number.isFinite(rule.required) ? Math.max(1, Math.min(20, Math.trunc(rule.required))) : 1
  if (!role) return null
  return { role, required }
}

export function normalizeReleaseGovernancePolicy(
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

function uniqueKnownValues<T extends string>(values: readonly T[], knownValues: readonly T[]): T[] {
  const known = new Set(knownValues)
  const result: T[] = []
  for (const value of values) {
    if (!known.has(value)) continue
    if (!result.includes(value)) result.push(value)
  }
  return result
}

function addUniqueByKey<T extends Record<K, string>, K extends keyof T>(
  items: T[],
  item: T,
  key: K,
) {
  if (items.some((existing) => existing[key] === item[key])) return
  items.push(item)
}

function adoptionReadinessTarget(preset: AdoptionPolicyPreset): number {
  if (preset === 'audit-only') return 0
  if (preset === 'strict') return 85
  return 75
}

function criticalHighPolicy(preset: AdoptionPolicyPreset): string {
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
  if (releaseGovernance.mode !== 'record-only' && !profile.modules.includes('formal-approval')) {
    errors.push('Enable the Formal approval module before choosing advisory, approval-required, or quorum-required release governance.')
  }
  if (releaseGovernance.mode === 'quorum-required' && releaseGovernance.quorum.rules.length === 0) {
    errors.push('Quorum-required release governance needs at least one approver role.')
  }

  return { valid: errors.length === 0, errors }
}

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
  const releaseGovernanceGateRequired = releaseGovernance.mode !== 'record-only' && modules.includes('formal-approval')

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

  if (releaseGovernanceGateRequired) {
    addUniqueByKey(workflowPlan, { file: '.github/workflows/release-governance-gate.yml', reason: 'optional release governance policy evaluator for customer-selected enforcement' }, 'file')
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
        ? 'Default record-only mode stores release approval evidence and does not block customer releases.'
        : `Customer selected ${releaseGovernance.mode} for ${releaseGovernance.environment}; review this explicit opt-in policy before installing any blocking workflow.`,
    }, 'step')
    if (releaseGovernanceGateRequired) {
      addUniqueByKey(manualSteps, {
        step: 'Validate release governance gate manually',
        detail: 'Run release-governance-gate.yml with workflow_dispatch before using it as a blocking release check. Default enforcement follows the selected release_governance mode.',
      }, 'step')
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
    { rule: 'Release governance gate', setting: releaseGovernanceGateRequired ? 'manual opt-in workflow' : 'not generated for record-only' },
    {
      rule: 'Release approval quorum',
      setting: releaseGovernance.quorum.enabled
        ? releaseGovernance.quorum.rules.map((rule) => `${rule.role}:${rule.required}`).join(', ')
        : 'disabled',
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

function yamlQuoted(value: string): string {
  return `"${value.replace(/\\/g, '\\\\').replace(/"/g, '\\"')}"`
}

function joinWorkflow(lines: string[]): string {
  return `${lines.join('\n')}\n`
}

function buildCiWorkflowTemplate(profile: EnterpriseAdoptionProfile): string {
  return joinWorkflow([
    '# Generated by GitGov dashboard workflow template pack.',
    '# Review project commands before installing in a customer repository.',
    'name: GitGov Customer CI',
    '',
    'on:',
    '  pull_request:',
    '  push:',
    `    branches: [${yamlQuoted(profile.default_branch.trim() || 'main')}]`,
    '  workflow_dispatch:',
    '',
    'permissions:',
    '  contents: read',
    '',
    'jobs:',
    '  ci:',
    '    name: Build and test',
    '    runs-on: ubuntu-latest',
    '    timeout-minutes: 30',
    '    steps:',
    '      - name: Checkout',
    '        uses: actions/checkout@v6',
    '      - name: Setup Node.js',
    '        uses: actions/setup-node@v6',
    '        with:',
    '          node-version: 20',
    '      - name: Run detected checks',
    '        shell: pwsh',
    '        run: |',
    '          $ErrorActionPreference = "Stop"',
    '          if (Test-Path "package.json") {',
    '            if (Test-Path "package-lock.json") { npm ci } else { npm install }',
    '            npm run lint --if-present',
    '            npm run typecheck --if-present',
    '            npm test --if-present',
    '            npm run build --if-present',
    '          } else {',
    '            Write-Warning "Customize this CI template for the customer stack."',
    '          }',
  ])
}

function buildSecretScanWorkflowTemplate(profile: EnterpriseAdoptionProfile): string {
  return joinWorkflow([
    '# Generated by GitGov dashboard workflow template pack.',
    '# Baseline guard: block committed env/secret file paths.',
    'name: GitGov Secret And Publication Guard',
    '',
    'on:',
    '  pull_request:',
    '  push:',
    `    branches: [${yamlQuoted(profile.default_branch.trim() || 'main')}]`,
    '  workflow_dispatch:',
    '',
    'permissions:',
    '  contents: read',
    '',
    'jobs:',
    '  secret-publication-guard:',
    '    runs-on: ubuntu-latest',
    '    steps:',
    '      - name: Checkout',
    '        uses: actions/checkout@v6',
    '      - name: Block committed secret files',
    '        shell: pwsh',
    '        run: |',
    '          $ErrorActionPreference = "Stop"',
    '          $blocked = @(git ls-files | Where-Object { ($_ -match \'(^|/)\\.env($|\\.|/)\' -and $_ -notmatch \'\\.env\\.example$\') -or ($_ -match \'(^|/)secrets/\') })',
    '          if ($blocked.Count -gt 0) {',
    '            $blocked | ForEach-Object { Write-Host "Blocked: $_" }',
    '            throw "Secret-like files must not be committed."',
    '          }',
    '          Write-Host "PASS: no blocked secret file paths are tracked."',
  ])
}

function buildTraceabilityWorkflowTemplate(profile: EnterpriseAdoptionProfile): string {
  const jiraKey = profile.jira_project_key.trim() || 'KAN'
  return joinWorkflow([
    '# Generated by GitGov dashboard workflow template pack.',
    'name: GitGov Traceability Guard',
    '',
    'on:',
    '  pull_request:',
    '  push:',
    `    branches: [${yamlQuoted(profile.default_branch.trim() || 'main')}]`,
    '  workflow_dispatch:',
    '',
    'permissions:',
    '  contents: read',
    '  pull-requests: read',
    '',
    'env:',
    `  JIRA_PROJECT_KEY: ${yamlQuoted(jiraKey)}`,
    '',
    'jobs:',
    '  traceability:',
    '    runs-on: ubuntu-latest',
    '    steps:',
    '      - name: Checkout',
    '        uses: actions/checkout@v6',
    '        with:',
    '          fetch-depth: 0',
    '      - name: Check branch, PR title, and latest commit',
    '        shell: pwsh',
    '        env:',
    '          EVENT_NAME_VALUE: ${{ github.event_name }}',
    '          HEAD_REF_VALUE: ${{ github.head_ref }}',
    '          REF_NAME_VALUE: ${{ github.ref_name }}',
    '          PR_TITLE_VALUE: ${{ github.event.pull_request.title }}',
    '        run: |',
    '          $ErrorActionPreference = "Stop"',
    '          $pattern = "\\b$([regex]::Escape($env:JIRA_PROJECT_KEY))-\\d+\\b"',
    '          $branch = if ([string]::IsNullOrWhiteSpace($env:HEAD_REF_VALUE)) { $env:REF_NAME_VALUE } else { $env:HEAD_REF_VALUE }',
    '          $commitSubject = git log -1 --pretty=%s',
    '          $failures = New-Object System.Collections.Generic.List[string]',
    '          if ($branch -notmatch $pattern) { $failures.Add("branch name") }',
    '          if ($env:EVENT_NAME_VALUE -eq "pull_request" -and $env:PR_TITLE_VALUE -notmatch $pattern) { $failures.Add("PR title") }',
    '          if ($commitSubject -notmatch $pattern) { $failures.Add("latest commit subject") }',
    '          if ($failures.Count -gt 0) { throw "Missing ticket ID pattern in: $($failures -join \', \')." }',
  ])
}

function buildGitGovEvidenceWorkflowTemplate(): string {
  return joinWorkflow([
    '# Generated by GitGov dashboard workflow template pack.',
    'name: GitGov Evidence Report',
    '',
    'on:',
    '  workflow_dispatch:',
    '  schedule:',
    '    - cron: "23 13 * * 1"',
    '',
    'permissions:',
    '  contents: read',
    '',
    'jobs:',
    '  evidence-report:',
    '    runs-on: ubuntu-latest',
    '    steps:',
    '      - name: Fetch GitGov stats',
    '        shell: pwsh',
    '        env:',
    '          GITGOV_URL: ${{ vars.GITGOV_URL }}',
    '          GITGOV_API_KEY: ${{ secrets.GITGOV_API_KEY }}',
    '          RUN_ID_VALUE: ${{ github.run_id }}',
    '        run: |',
    '          $ErrorActionPreference = "Stop"',
    '          New-Item -ItemType Directory -Force -Path "gitgov-evidence" | Out-Null',
    '          $outputPath = "gitgov-evidence/github-evidence-report-$env:RUN_ID_VALUE.json"',
    '          if ([string]::IsNullOrWhiteSpace($env:GITGOV_URL) -or [string]::IsNullOrWhiteSpace($env:GITGOV_API_KEY)) {',
    '            @{ status = "skipped"; reason = "missing_gitgov_config"; generated_at = [DateTimeOffset]::UtcNow.ToString("o") } | ConvertTo-Json | Out-File $outputPath -Encoding UTF8',
    '            exit 0',
    '          }',
    '          $headers = @{ Authorization = "Bearer $env:GITGOV_API_KEY" }',
    '          $stats = Invoke-RestMethod -Method GET -Uri "$($env:GITGOV_URL.TrimEnd(\'/\'))/stats" -Headers $headers',
    '          @{ status = "ok"; stats = $stats; generated_at = [DateTimeOffset]::UtcNow.ToString("o") } | ConvertTo-Json -Depth 12 | Out-File $outputPath -Encoding UTF8',
    '      - name: Upload evidence artifact',
    '        uses: actions/upload-artifact@v7',
    '        with:',
    '          name: github-evidence-report-${{ github.run_id }}',
    '          path: gitgov-evidence',
    '          if-no-files-found: error',
  ])
}

function buildArtifactMonitorWorkflowTemplate(name: string, artifactPrefix: string, outputPrefix: string): string {
  return joinWorkflow([
    '# Generated by GitGov dashboard workflow template pack.',
    `name: ${name}`,
    '',
    'on:',
    '  workflow_dispatch:',
    '',
    'permissions:',
    '  actions: read',
    '  contents: read',
    '',
    'jobs:',
    '  artifact-monitor:',
    '    runs-on: ubuntu-latest',
    '    steps:',
    '      - name: Check artifact freshness',
    '        shell: pwsh',
    '        env:',
    '          GH_TOKEN: ${{ github.token }}',
    '          REPOSITORY_NAME: ${{ github.repository }}',
    `          ARTIFACT_PREFIX: ${yamlQuoted(artifactPrefix)}`,
    `          OUTPUT_PREFIX: ${yamlQuoted(outputPrefix)}`,
    '        run: |',
    '          $ErrorActionPreference = "Stop"',
    '          New-Item -ItemType Directory -Force -Path "gitgov-evidence" | Out-Null',
    '          $headers = @{ Authorization = "Bearer $env:GH_TOKEN"; Accept = "application/vnd.github+json" }',
    '          $uri = "https://api.github.com/repos/$env:REPOSITORY_NAME/actions/artifacts?per_page=100"',
    '          $response = Invoke-RestMethod -Method GET -Uri $uri -Headers $headers',
    '          $latest = @($response.artifacts | Where-Object { $_.name -like "$env:ARTIFACT_PREFIX*" -and $_.expired -ne $true } | Sort-Object created_at -Descending | Select-Object -First 1)',
    '          $status = if ($latest.Count -gt 0) { "pass" } else { "fail" }',
    '          @{ status = $status; latest_artifact_name = if ($latest.Count) { $latest[0].name } else { $null }; generated_at = [DateTimeOffset]::UtcNow.ToString("o") } | ConvertTo-Json -Depth 8 | Out-File "gitgov-evidence/$env:OUTPUT_PREFIX.json" -Encoding UTF8',
    '          if ($status -ne "pass") { throw "No fresh evidence artifact found." }',
    '      - name: Upload monitor artifact',
    '        uses: actions/upload-artifact@v7',
    '        with:',
    `          name: ${outputPrefix}`,
    '          path: gitgov-evidence',
    '          if-no-files-found: error',
  ])
}

function buildArtifactTrendWorkflowTemplate(name: string, artifactPrefix: string, outputPrefix: string): string {
  return joinWorkflow([
    '# Generated by GitGov dashboard workflow template pack.',
    `name: ${name}`,
    '',
    'on:',
    '  workflow_dispatch:',
    '',
    'permissions:',
    '  actions: read',
    '  contents: read',
    '',
    'jobs:',
    '  trend-report:',
    '    runs-on: ubuntu-latest',
    '    steps:',
    '      - name: Build trend inventory',
    '        shell: pwsh',
    '        env:',
    '          GH_TOKEN: ${{ github.token }}',
    '          REPOSITORY_NAME: ${{ github.repository }}',
    `          ARTIFACT_PREFIX: ${yamlQuoted(artifactPrefix)}`,
    `          OUTPUT_PREFIX: ${yamlQuoted(outputPrefix)}`,
    '        run: |',
    '          $ErrorActionPreference = "Stop"',
    '          New-Item -ItemType Directory -Force -Path "gitgov-evidence" | Out-Null',
    '          $headers = @{ Authorization = "Bearer $env:GH_TOKEN"; Accept = "application/vnd.github+json" }',
    '          $uri = "https://api.github.com/repos/$env:REPOSITORY_NAME/actions/artifacts?per_page=100"',
    '          $response = Invoke-RestMethod -Method GET -Uri $uri -Headers $headers',
    '          $artifacts = @($response.artifacts | Where-Object { $_.name -like "$env:ARTIFACT_PREFIX*" } | Sort-Object created_at -Descending | Select-Object -First 10)',
    '          @{ status = if ($artifacts.Count) { "pass" } else { "missing" }; artifact_count = $artifacts.Count; generated_at = [DateTimeOffset]::UtcNow.ToString("o") } | ConvertTo-Json -Depth 8 | Out-File "gitgov-evidence/$env:OUTPUT_PREFIX.json" -Encoding UTF8',
    '          if ($artifacts.Count -eq 0) { throw "No artifacts found for prefix $env:ARTIFACT_PREFIX." }',
    '      - name: Upload trend artifact',
    '        uses: actions/upload-artifact@v7',
    '        with:',
    `          name: ${outputPrefix}`,
    '          path: gitgov-evidence',
    '          if-no-files-found: error',
  ])
}

function buildReleaseReadinessWorkflowTemplate(profile: EnterpriseAdoptionProfile): string {
  const target = adoptionReadinessTarget(profile.policy_preset)
  const enforce = profile.policy_preset === 'audit-only' ? 'false' : 'true'
  return joinWorkflow([
    '# Generated by GitGov dashboard workflow template pack.',
    'name: GitGov Release Readiness Gate',
    '',
    'on:',
    '  push:',
    `    branches: [${yamlQuoted(profile.default_branch.trim() || 'main')}]`,
    '  workflow_dispatch:',
    '    inputs:',
    '      enforce_gate:',
    '        description: "Fail when readiness is below target"',
    '        required: false',
    `        default: ${enforce}`,
    '        type: boolean',
    '',
    'permissions:',
    '  contents: read',
    '',
    'jobs:',
    '  readiness:',
    '    runs-on: ubuntu-latest',
    '    steps:',
    '      - name: Compute readiness',
    '        shell: pwsh',
    '        env:',
    '          GITGOV_URL: ${{ vars.GITGOV_URL }}',
    '          GITGOV_API_KEY: ${{ secrets.GITGOV_API_KEY }}',
    '          REPOSITORY_NAME: ${{ github.repository }}',
    '          REF_NAME_VALUE: ${{ github.ref_name }}',
    '          INPUT_ENFORCE_GATE: ${{ inputs.enforce_gate }}',
    `          TARGET_READINESS: ${yamlQuoted(String(target))}`,
    '        run: |',
    '          $ErrorActionPreference = "Stop"',
    '          if ([string]::IsNullOrWhiteSpace($env:GITGOV_URL) -or [string]::IsNullOrWhiteSpace($env:GITGOV_API_KEY)) { exit 0 }',
    '          $headers = @{ Authorization = "Bearer $env:GITGOV_API_KEY" }',
    '          $baseUrl = $env:GITGOV_URL.TrimEnd("/")',
    '          $repo = [Uri]::EscapeDataString($env:REPOSITORY_NAME)',
    '          $branch = [Uri]::EscapeDataString($env:REF_NAME_VALUE)',
    '          $coverage = Invoke-RestMethod -Method GET -Uri "$baseUrl/integrations/jira/ticket-coverage?repo_full_name=$repo&branch=$branch&hours=720" -Headers $headers',
    '          $score = [int]([double]($coverage.coverage_percentage ?? 0))',
    '          if ($score -lt [int]$env:TARGET_READINESS -and $env:INPUT_ENFORCE_GATE -eq "true") { throw "Readiness below target." }',
  ])
}

function buildReleaseGovernanceGateWorkflowTemplate(profile: EnterpriseAdoptionProfile): string {
  const releaseGovernance = normalizeReleaseGovernancePolicy(profile.release_governance)
  const enforce = releaseGovernance.mode === 'approval-required' || releaseGovernance.mode === 'quorum-required'
    ? 'true'
    : 'false'
  return joinWorkflow([
    '# Generated by GitGov dashboard workflow template pack.',
    '# Manual release governance evaluation. Blocking is customer-selected, never default record-only behavior.',
    'name: GitGov Release Governance Gate',
    '',
    'on:',
    '  workflow_dispatch:',
    '    inputs:',
    '      org_name:',
    '        description: "GitGov organization scope"',
    '        required: false',
    '        default: ""',
    '        type: string',
    '      release_id:',
    '        description: "Release identifier to evaluate"',
    '        required: false',
    '        default: ""',
    '        type: string',
    '      environment:',
    '        description: "Release environment"',
    '        required: false',
    `        default: ${yamlQuoted(releaseGovernance.environment)}`,
    '        type: string',
    '      evidence_packet_hash:',
    '        description: "Optional SHA-256 evidence packet hash"',
    '        required: false',
    '        default: ""',
    '        type: string',
    '      enforce_gate:',
    '        description: "Fail only when explicitly blocking policy is not satisfied"',
    '        required: false',
    `        default: ${enforce}`,
    '        type: boolean',
    '',
    'permissions:',
    '  contents: read',
    '',
    'jobs:',
    '  release-governance:',
    '    runs-on: ubuntu-latest',
    '    steps:',
    '      - name: Evaluate release governance',
    '        shell: pwsh',
    '        env:',
    '          GITGOV_URL: ${{ vars.GITGOV_URL }}',
    '          GITGOV_API_KEY: ${{ secrets.GITGOV_API_KEY }}',
    '          REPOSITORY_NAME: ${{ github.repository }}',
    '          REF_NAME_VALUE: ${{ github.ref_name }}',
    '          RUN_ID_VALUE: ${{ github.run_id }}',
    '          INPUT_ORG_NAME: ${{ inputs.org_name }}',
    '          INPUT_RELEASE_ID: ${{ inputs.release_id }}',
    '          INPUT_ENVIRONMENT: ${{ inputs.environment }}',
    '          INPUT_EVIDENCE_PACKET_HASH: ${{ inputs.evidence_packet_hash }}',
    '          INPUT_ENFORCE_GATE: ${{ inputs.enforce_gate }}',
    '        run: |',
    '          $ErrorActionPreference = "Stop"',
    '          New-Item -ItemType Directory -Force -Path "gitgov-evidence" | Out-Null',
    '          $outputPath = "gitgov-evidence/release-governance-gate-$env:RUN_ID_VALUE.json"',
    '          if ([string]::IsNullOrWhiteSpace($env:GITGOV_URL) -or [string]::IsNullOrWhiteSpace($env:GITGOV_API_KEY)) {',
    '            @{ status = "skipped"; reason = "missing_gitgov_url_or_api_key"; generated_at = [DateTimeOffset]::UtcNow.ToString("o") } | ConvertTo-Json -Depth 6 | Out-File -FilePath $outputPath -Encoding UTF8',
    '            if ($env:INPUT_ENFORCE_GATE -eq "true") { throw "Missing GitGov configuration for enforced release governance gate." }',
    '            exit 0',
    '          }',
    '          $releaseId = $env:INPUT_RELEASE_ID',
    '          if ([string]::IsNullOrWhiteSpace($releaseId)) { $releaseId = $env:REF_NAME_VALUE }',
    '          $environment = $env:INPUT_ENVIRONMENT',
    `          if ([string]::IsNullOrWhiteSpace($environment)) { $environment = ${yamlQuoted(releaseGovernance.environment)} }`,
    '          $query = New-Object System.Collections.Generic.List[string]',
    '          if (-not [string]::IsNullOrWhiteSpace($env:INPUT_ORG_NAME)) { $query.Add("org_name=$([Uri]::EscapeDataString($env:INPUT_ORG_NAME))") | Out-Null }',
    '          $query.Add("repository_full_name=$([Uri]::EscapeDataString($env:REPOSITORY_NAME))") | Out-Null',
    '          $query.Add("release_id=$([Uri]::EscapeDataString($releaseId))") | Out-Null',
    '          $query.Add("environment=$([Uri]::EscapeDataString($environment))") | Out-Null',
    '          if (-not [string]::IsNullOrWhiteSpace($env:INPUT_EVIDENCE_PACKET_HASH)) { $query.Add("evidence_packet_hash=$([Uri]::EscapeDataString($env:INPUT_EVIDENCE_PACKET_HASH))") | Out-Null }',
    '          $baseUrl = $env:GITGOV_URL.TrimEnd("/")',
    '          $headers = @{ Authorization = "Bearer $env:GITGOV_API_KEY"; Accept = "application/json" }',
    `          $evaluation = Invoke-RestMethod -Method GET -Uri "$baseUrl/enterprise/release-governance/evaluate?$($query -join '&')" -Headers $headers`,
    '          $result = [ordered]@{ status = "evaluated"; release_id = $releaseId; environment = $environment; enforce = ($env:INPUT_ENFORCE_GATE -eq "true"); evaluation = @{ status = $evaluation.status; policy_mode = $evaluation.policy.mode; policy_enforcement = $evaluation.policy.enforcement; policy_satisfied = $evaluation.policy_satisfied; blocking = $evaluation.blocking; would_block = $evaluation.would_block; valid_approval_count = $evaluation.valid_approval_count; required_approval_count = $evaluation.required_approval_count }; safety = @{ contains_secret_values = $false; prints_authorization_header = $false }; generated_at = [DateTimeOffset]::UtcNow.ToString("o") }',
    '          $result | ConvertTo-Json -Depth 10 | Out-File -FilePath $outputPath -Encoding UTF8',
    '          if ($env:INPUT_ENFORCE_GATE -eq "true" -and $evaluation.blocking -eq $true) { throw "Release governance blocking policy is not satisfied." }',
    '      - name: Upload release governance evidence',
    '        uses: actions/upload-artifact@v7',
    '        with:',
    '          name: release-governance-gate-${{ github.run_id }}',
    '          path: gitgov-evidence',
    '          if-no-files-found: error',
  ])
}

function buildQualityGatePolicyWorkflowTemplate(profile: EnterpriseAdoptionProfile): string {
  return joinWorkflow([
    '# Generated by GitGov dashboard workflow template pack.',
    'name: GitGov Quality Gate Policy Matrix',
    '',
    'on:',
    '  pull_request:',
    '  push:',
    `    branches: [${yamlQuoted(profile.default_branch.trim() || 'main')}]`,
    '  workflow_dispatch:',
    '',
    'permissions:',
    '  contents: read',
    '',
    'jobs:',
    '  quality-gate-policy:',
    '    runs-on: ubuntu-latest',
    '    steps:',
    '      - name: Emit policy evidence',
    '        shell: pwsh',
    '        env:',
    `          POLICY_PRESET: ${yamlQuoted(profile.policy_preset)}`,
    `          READINESS_TARGET: ${yamlQuoted(String(adoptionReadinessTarget(profile.policy_preset)))}`,
    '        run: |',
    '          $ErrorActionPreference = "Stop"',
    '          if ($env:POLICY_PRESET -notin @("audit-only", "moderate", "strict")) { throw "Unsupported policy preset." }',
    '          Write-Host "PASS: quality gate policy preset is valid."',
  ])
}

function buildSonarGovernanceWorkflowTemplate(): string {
  return joinWorkflow([
    '# Generated by GitGov dashboard workflow template pack.',
    'name: GitGov SonarQube Governance',
    '',
    'on:',
    '  workflow_dispatch:',
    '',
    'permissions:',
    '  contents: read',
    '',
    'jobs:',
    '  sonarqube-governance:',
    '    runs-on: ubuntu-latest',
    '    steps:',
    '      - name: Check SonarQube quality gate',
    '        shell: pwsh',
    '        env:',
    '          SONAR_HOST_URL: ${{ vars.SONAR_HOST_URL }}',
    '          SONAR_PROJECT_KEY: ${{ vars.SONAR_PROJECT_KEY }}',
    '          SONAR_TOKEN: ${{ secrets.SONAR_TOKEN }}',
    '        run: |',
    '          $ErrorActionPreference = "Stop"',
    '          if ([string]::IsNullOrWhiteSpace($env:SONAR_HOST_URL) -or [string]::IsNullOrWhiteSpace($env:SONAR_PROJECT_KEY)) { exit 0 }',
    '          $headers = @{}',
    '          if (-not [string]::IsNullOrWhiteSpace($env:SONAR_TOKEN)) { $headers.Authorization = "Bearer $env:SONAR_TOKEN" }',
    '          $hostUrl = $env:SONAR_HOST_URL.TrimEnd("/")',
    '          Invoke-RestMethod -Method GET -Uri "$hostUrl/api/qualitygates/project_status?projectKey=$([Uri]::EscapeDataString($env:SONAR_PROJECT_KEY))" -Headers $headers | Out-Null',
  ])
}

function buildProductVulnerabilityReviewWorkflowTemplate(): string {
  return joinWorkflow([
    '# Generated by GitGov dashboard workflow template pack.',
    'name: GitGov Product Vulnerability Review',
    '',
    'on:',
    '  workflow_dispatch:',
    '  schedule:',
    '    - cron: "41 12 * * 4"',
    '',
    'permissions:',
    '  contents: read',
    '',
    'jobs:',
    '  product-vulnerability-review:',
    '    runs-on: ubuntu-latest',
    '    steps:',
    '      - name: Checkout',
    '        uses: actions/checkout@v6',
    '      - name: Run dependency review baseline',
    '        shell: pwsh',
    '        run: |',
    '          $ErrorActionPreference = "Continue"',
    '          New-Item -ItemType Directory -Force -Path "gitgov-evidence/product-vulnerability-review" | Out-Null',
    '          if (Test-Path "package-lock.json") { npm audit --json | Out-File "gitgov-evidence/product-vulnerability-review/npm-audit.json" -Encoding UTF8 }',
    '          if (Test-Path "Cargo.lock") { cargo install cargo-audit --locked; cargo audit --json | Out-File "gitgov-evidence/product-vulnerability-review/cargo-audit.json" -Encoding UTF8 }',
    '      - name: Upload review evidence',
    '        uses: actions/upload-artifact@v7',
    '        with:',
    '          name: product-vulnerability-review-${{ github.run_id }}',
    '          path: gitgov-evidence/product-vulnerability-review',
    '          if-no-files-found: warn',
  ])
}

function buildVulnerabilityTrendEnforcementWorkflowTemplate(profile: EnterpriseAdoptionProfile): string {
  const baseline = profile.policy_preset === 'audit-only' ? '999' : '1'
  return joinWorkflow([
    '# Generated by GitGov dashboard workflow template pack.',
    'name: GitGov Vulnerability Trend Enforcement',
    '',
    'on:',
    '  workflow_dispatch:',
    '',
    'permissions:',
    '  actions: read',
    '  contents: read',
    '',
    'jobs:',
    '  trend-enforcement:',
    '    runs-on: ubuntu-latest',
    '    steps:',
    '      - name: Enforce latest review artifact presence',
    '        shell: pwsh',
    '        env:',
    '          GH_TOKEN: ${{ github.token }}',
    '          REPOSITORY_NAME: ${{ github.repository }}',
    '          ARTIFACT_PREFIX: "product-vulnerability-review-"',
    `          ACCEPTED_FINDING_BASELINE: ${yamlQuoted(baseline)}`,
    '        run: |',
    '          $ErrorActionPreference = "Stop"',
    '          $headers = @{ Authorization = "Bearer $env:GH_TOKEN"; Accept = "application/vnd.github+json" }',
    '          $uri = "https://api.github.com/repos/$env:REPOSITORY_NAME/actions/artifacts?per_page=100"',
    '          $response = Invoke-RestMethod -Method GET -Uri $uri -Headers $headers',
    '          $latest = @($response.artifacts | Where-Object { $_.name -like "$env:ARTIFACT_PREFIX*" -and $_.expired -ne $true } | Select-Object -First 1)',
    '          if ($latest.Count -eq 0) { throw "No fresh product vulnerability review artifact found." }',
  ])
}

function workflowTemplateContent(file: string, profile: EnterpriseAdoptionProfile): string {
  switch (file) {
    case '.github/workflows/ci.yml':
      return buildCiWorkflowTemplate(profile)
    case '.github/workflows/secret-scan.yml':
      return buildSecretScanWorkflowTemplate(profile)
    case '.github/workflows/public-naming-guard.yml':
      return buildTraceabilityWorkflowTemplate(profile)
    case '.github/workflows/github-evidence-report.yml':
      return buildGitGovEvidenceWorkflowTemplate()
    case '.github/workflows/github-evidence-artifact-monitor.yml':
      return buildArtifactMonitorWorkflowTemplate('GitGov Evidence Artifact Monitor', 'github-evidence-report-', 'github-evidence-artifact-monitor')
    case '.github/workflows/github-evidence-trend-report.yml':
      return buildArtifactTrendWorkflowTemplate('GitGov Evidence Trend Report', 'github-evidence-report-', 'github-evidence-trend-report')
    case '.github/workflows/release-readiness-gate.yml':
      return buildReleaseReadinessWorkflowTemplate(profile)
    case '.github/workflows/release-governance-gate.yml':
      return buildReleaseGovernanceGateWorkflowTemplate(profile)
    case '.github/workflows/quality-gate-policy-matrix.yml':
      return buildQualityGatePolicyWorkflowTemplate(profile)
    case '.github/workflows/sonar-governance.yml':
      return buildSonarGovernanceWorkflowTemplate()
    case '.github/workflows/product-vulnerability-review.yml':
      return buildProductVulnerabilityReviewWorkflowTemplate()
    case '.github/workflows/product-vulnerability-review-artifact-monitor.yml':
      return buildArtifactMonitorWorkflowTemplate('GitGov Product Vulnerability Review Artifact Monitor', 'product-vulnerability-review-', 'product-vulnerability-review-artifact-monitor')
    case '.github/workflows/product-vulnerability-review-trend-report.yml':
      return buildArtifactTrendWorkflowTemplate('GitGov Product Vulnerability Review Trend Report', 'product-vulnerability-review-', 'product-vulnerability-review-trend-report')
    case '.github/workflows/product-vulnerability-review-trend-enforcement.yml':
      return buildVulnerabilityTrendEnforcementWorkflowTemplate(profile)
    default:
      return joinWorkflow([
        '# Generated by GitGov dashboard workflow template pack.',
        `name: ${file.split('/').pop()?.replace(/\.ya?ml$/, '') ?? 'GitGov workflow'}`,
        'on:',
        '  workflow_dispatch:',
        'permissions:',
        '  contents: read',
        'jobs:',
        '  placeholder:',
        '    runs-on: ubuntu-latest',
        '    steps:',
        '      - run: echo "Review and customize this generated workflow."',
      ])
  }
}

function buildWorkflowTemplateReadme(pack: EnterpriseAdoptionPack): string {
  const workflowLines = pack.workflow_plan.map((workflow) => `| \`${workflow.file}\` | ${workflow.reason.replace(/\|/g, '\\|')} |`)
  const variableLines = pack.variables.length > 0
    ? pack.variables.map((variable) => `| \`${variable.name}\` | ${variable.purpose.replace(/\|/g, '\\|')} | \`${variable.example}\` |`)
    : ['- None.']
  const secretLines = pack.secrets.length > 0
    ? pack.secrets.map((secret) => `| \`${secret.name}\` | ${secret.purpose.replace(/\|/g, '\\|')} | ${secret.value_policy.replace(/\|/g, '\\|')} |`)
    : ['- None.']
  const stepLines = pack.manual_steps.map((step) => `- **${step.step}:** ${step.detail}`)

  return [
    '# GitGov Workflow Template Pack',
    '',
    `Generated: \`${pack.generated_at}\``,
    '',
    `Customer: \`${pack.customer_name}\``,
    `Repository: \`${pack.repository_full_name}\``,
    `Default branch: \`${pack.default_branch}\``,
    `Policy preset: \`${pack.policy_preset}\``,
    `Release governance: \`${pack.release_governance.mode}\``,
    `Release enforcement: \`${pack.release_governance.enforcement}\``,
    pack.jira_project_key ? `Jira project key: \`${pack.jira_project_key}\`` : '',
    '',
    '## Generated Templates',
    '',
    '| Workflow | Why |',
    '|---|---|',
    ...workflowLines,
    '',
    '## Required Variables',
    '',
    ...(pack.variables.length > 0 ? ['| Name | Purpose | Example |', '|---|---|---|', ...variableLines] : variableLines),
    '',
    '## Required Secrets',
    '',
    ...(pack.secrets.length > 0 ? ['| Name | Purpose | Value Policy |', '|---|---|---|', ...secretLines] : secretLines),
    '',
    '## Manual Install Checklist',
    '',
    ...stepLines,
    '',
    '## Safety Notes',
    '',
    '- This pack contains workflow templates, variable names, and secret names only.',
    '- It does not contain secret values.',
    '- It does not mutate the customer repository automatically.',
    '- Review generated commands and permissions before copying templates into `.github/workflows`.',
  ].filter((line, index, lines) => line !== '' || lines[index - 1] !== '').join('\n')
}

export function buildEnterpriseWorkflowTemplatePack(
  profile: EnterpriseAdoptionProfile,
  generatedAt = new Date().toISOString(),
): EnterpriseWorkflowTemplatePack {
  const pack = buildEnterpriseAdoptionPack(profile, generatedAt)
  const files = pack.workflow_plan.map((workflow) => ({
    file: workflow.file,
    reason: workflow.reason,
    content: workflowTemplateContent(workflow.file, {
      ...profile,
      default_branch: pack.default_branch,
      jira_project_key: pack.jira_project_key,
      release_governance: pack.release_governance,
    }),
  }))

  return {
    generated_at: generatedAt,
    manifest: {
      generated_at: generatedAt,
      customer_name: pack.customer_name,
      repository_full_name: pack.repository_full_name,
      default_branch: pack.default_branch,
      jira_project_key: pack.jira_project_key,
      policy_preset: pack.policy_preset,
      release_governance: pack.release_governance,
      providers: pack.providers,
      modules: pack.modules,
      workflow_templates: files.map((file) => ({
        file: file.file,
        reason: file.reason,
        requires_review_before_install: true,
      })),
      variables: pack.variables,
      secrets: pack.secrets,
      manual_steps: [
        ...pack.manual_steps,
        {
          step: 'Review generated YAML',
          detail: 'Install templates only after customer owners review commands, schedules, permissions, and branch names.',
        },
        {
          step: 'Run workflow_dispatch first',
          detail: 'Validate each workflow manually before relying on schedules or blocking behavior.',
        },
      ],
      open_product_gaps: pack.open_product_gaps,
      safety: {
        contains_secret_values: false,
        mutates_customer_repository: false,
        requires_manual_install_review: true,
      },
    },
    files,
    readme: buildWorkflowTemplateReadme(pack),
  }
}

function hasPackVariable(pack: EnterpriseAdoptionPack, name: string): boolean {
  return pack.variables.some((variable) => variable.name === name)
}

function hasPackSecret(pack: EnterpriseAdoptionPack, name: string): boolean {
  return pack.secrets.some((secret) => secret.name === name)
}

function providerLabel(provider: AdoptionProvider): string {
  return ADOPTION_PROVIDER_OPTIONS.find((option) => option.id === provider)?.label ?? provider
}

function selectedProviders(profile: EnterpriseAdoptionProfile): AdoptionProvider[] {
  return uniqueKnownValues(profile.providers, ADOPTION_PROVIDER_IDS)
}

export function buildEnterpriseProviderHealth(
  profile: EnterpriseAdoptionProfile,
  evidence: EnterpriseProviderHealthEvidence = {},
  pack = buildEnterpriseAdoptionPack(profile),
): EnterpriseProviderHealthCheck[] {
  const checks: EnterpriseProviderHealthCheck[] = []
  const modules = uniqueKnownValues(profile.modules, ADOPTION_MODULE_IDS)
  const jiraKey = profile.jira_project_key.trim()
  const githubEventsTotal = evidence.githubEventsTotal ?? 0
  const jiraCommitsWithTicket = evidence.jiraCommitsWithTicket ?? 0
  const jiraCoveragePercentage = evidence.jiraCoveragePercentage ?? 0
  const pipelineRuns7d = evidence.pipelineRuns7d ?? 0
  const pipelineSuccess7d = evidence.pipelineSuccess7d ?? 0
  const sonarRuns = evidence.sonarRuns ?? 0
  const sonarSuccessful = evidence.sonarSuccessful ?? 0
  const activeRepos = evidence.activeRepos ?? 0

  for (const provider of selectedProviders(profile)) {
    if (provider === 'github') {
      const hasTelemetryConfig = hasPackVariable(pack, 'GITGOV_URL') && hasPackSecret(pack, 'GITGOV_API_KEY')
      checks.push({
        provider,
        label: providerLabel(provider),
        status: !hasTelemetryConfig ? 'needs-config' : githubEventsTotal > 0 ? 'ready' : 'needs-evidence',
        evidence: githubEventsTotal > 0
          ? `${githubEventsTotal} GitHub events observed`
          : 'No GitHub webhook or workflow evidence observed yet',
        next_step: hasTelemetryConfig
          ? 'Confirm signed webhook events and GitGov workflow telemetry are installed.'
          : 'Add GITGOV_URL and GITGOV_API_KEY to the adoption pack.',
      })
      continue
    }

    if (provider === 'jira') {
      const hasTraceabilityEvidence = jiraCommitsWithTicket > 0 || jiraCoveragePercentage > 0
      checks.push({
        provider,
        label: providerLabel(provider),
        status: !jiraKey ? 'needs-config' : hasTraceabilityEvidence ? 'ready' : 'needs-evidence',
        evidence: hasTraceabilityEvidence
          ? `${jiraCommitsWithTicket} ticket-linked commits, ${jiraCoveragePercentage.toFixed(2)}% coverage`
          : 'No Jira ticket correlation evidence observed yet',
        next_step: jiraKey
          ? 'Run Jira ingest/correlation and confirm ticket IDs appear in PRs, branches, or commits.'
          : 'Set the Jira project key for traceability validation.',
      })
      continue
    }

    if (provider === 'jenkins') {
      checks.push({
        provider,
        label: providerLabel(provider),
        status: pipelineRuns7d > 0 ? 'ready' : 'needs-evidence',
        evidence: pipelineRuns7d > 0
          ? `${pipelineRuns7d} pipeline runs observed in 7d, ${pipelineSuccess7d} successful`
          : 'No Jenkins pipeline evidence observed in the current 7d window',
        next_step: 'Publish Jenkins job telemetry to GitGov and verify pipeline evidence appears.',
      })
      continue
    }

    if (provider === 'sonarqube') {
      const hasSonarConfig = hasPackVariable(pack, 'SONAR_HOST_URL') && hasPackVariable(pack, 'SONAR_PROJECT_KEY')
      checks.push({
        provider,
        label: providerLabel(provider),
        status: !modules.includes('quality-gates') || !hasSonarConfig
          ? 'needs-config'
          : sonarRuns > 0
            ? 'ready'
            : 'needs-evidence',
        evidence: sonarRuns > 0
          ? `${sonarRuns} Sonar/quality runs observed, ${sonarSuccessful} successful`
          : 'No quality gate evidence observed in current dashboard stats',
        next_step: modules.includes('quality-gates')
          ? 'Validate SonarQube runtime reachability from the chosen runner.'
          : 'Enable the Quality gates module before validating SonarQube.',
      })
      continue
    }

    if (provider === 'render') {
      checks.push({
        provider,
        label: providerLabel(provider),
        status: activeRepos > 0 ? 'ready' : 'needs-evidence',
        evidence: activeRepos > 0
          ? `${activeRepos} active repositories observed by GitGov`
          : 'No deployment-provider evidence is available in the current adoption profile',
        next_step: 'Record deployment health and release metadata without storing provider tokens.',
      })
      continue
    }

    checks.push({
      provider,
      label: providerLabel(provider),
      status: 'needs-evidence',
      evidence: 'No Vercel deployment evidence is available in the current adoption profile',
      next_step: 'Connect deployment status or preview evidence when Vercel is used by the customer.',
    })
  }

  return checks
}

export function buildEnterpriseAdoptionPackFilename(profile: EnterpriseAdoptionProfile): string {
  const basis = `${profile.customer_name}-${profile.repository_full_name}`
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
  return `${basis || 'enterprise-adoption'}-pack.json`
}

export function buildEnterpriseWorkflowTemplatePackFilename(profile: EnterpriseAdoptionProfile): string {
  const basis = `${profile.customer_name}-${profile.repository_full_name}`
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
  return `${basis || 'enterprise-adoption'}-workflow-template-pack.json`
}

export function readDetailFiles(log: CombinedEvent): string[] {
  const direct = log.details?.['files']
  if (Array.isArray(direct)) return direct.filter((v): v is string => typeof v === 'string')
  return []
}

export interface DashboardRow { log: CombinedEvent; attachedFiles: string[] }

export interface GitHubEvidenceSummary {
  prLifecycleCount: number
  prReviewCount: number
  prCommentCount: number
  statusCheckCount: number
  activeSignals: number
  totalSignals: number
  executiveStatus: 'Completo' | 'Parcial' | 'Sin evidencia'
  missingSignals: string[]
}

export interface GitHubEvidenceTrendPoint {
  capturedAt: string
  activeSignals: number
  totalSignals: number
  executiveStatus: GitHubEvidenceSummary['executiveStatus']
  missingSignals: string[]
}

interface AuditExportResponse {
  id: string
  export_type: string
  record_count: number
  content_hash: string
  data?: unknown
  created_at: number
}

export interface AuditExportPackage {
  export_id: string
  export_type: string
  record_count: number
  source_content_hash: string
  created_at: number
  packaged_at: string
  executive_summary: {
    github_evidence: GitHubEvidenceSummary
    scope_note: string
  }
  data: unknown
}

export function buildGitHubEvidenceSummary(githubByType: Record<string, number>): GitHubEvidenceSummary {
  const prLifecycleCount = githubByType.pull_request ?? 0
  const prReviewCount = githubByType.pull_request_review ?? 0
  const prCommentCount =
    (githubByType.pull_request_review_comment ?? 0) +
    (githubByType.issue_comment ?? 0)
  const statusCheckCount =
    (githubByType.check_run ?? 0) +
    (githubByType.check_suite ?? 0) +
    (githubByType.status ?? 0)

  const signals = [
    ['PR lifecycle', prLifecycleCount],
    ['Reviews', prReviewCount],
    ['Comentarios PR', prCommentCount],
    ['Checks/status', statusCheckCount],
  ] as const
  const activeSignals = signals.filter(([, count]) => count > 0).length
  const executiveStatus =
    activeSignals === signals.length
      ? 'Completo'
      : activeSignals > 0
        ? 'Parcial'
        : 'Sin evidencia'

  return {
    prLifecycleCount,
    prReviewCount,
    prCommentCount,
    statusCheckCount,
    activeSignals,
    totalSignals: signals.length,
    executiveStatus,
    missingSignals: signals
      .filter(([, count]) => count === 0)
      .map(([label]) => label),
  }
}

export function buildGitHubEvidenceTrendPoint(
  summary: GitHubEvidenceSummary,
  capturedAt = new Date().toISOString(),
): GitHubEvidenceTrendPoint {
  return {
    capturedAt,
    activeSignals: summary.activeSignals,
    totalSignals: summary.totalSignals,
    executiveStatus: summary.executiveStatus,
    missingSignals: summary.missingSignals,
  }
}

export function appendGitHubEvidenceTrendPoint(
  previous: GitHubEvidenceTrendPoint[],
  next: GitHubEvidenceTrendPoint,
  maxPoints = 12,
): GitHubEvidenceTrendPoint[] {
  const latest = previous[previous.length - 1]
  const shouldReplaceLatest =
    latest &&
    latest.activeSignals === next.activeSignals &&
    latest.totalSignals === next.totalSignals &&
    latest.executiveStatus === next.executiveStatus &&
    latest.missingSignals.join('|') === next.missingSignals.join('|')

  const merged = shouldReplaceLatest
    ? [...previous.slice(0, -1), next]
    : [...previous, next]

  return merged.slice(Math.max(0, merged.length - maxPoints))
}

export function buildAuditExportPackage(
  exportResponse: AuditExportResponse,
  githubByType: Record<string, number>,
  packagedAt = new Date().toISOString(),
): AuditExportPackage {
  return {
    export_id: exportResponse.id,
    export_type: exportResponse.export_type,
    record_count: exportResponse.record_count,
    source_content_hash: exportResponse.content_hash,
    created_at: exportResponse.created_at,
    packaged_at: packagedAt,
    executive_summary: {
      github_evidence: buildGitHubEvidenceSummary(githubByType),
      scope_note: 'Dashboard snapshot at export time; raw audit records remain in data.',
    },
    data: exportResponse.data ?? null,
  }
}

export function buildDashboardRows(logs: CombinedEvent[]): DashboardRow[] {
  const WINDOW_MS = 10 * 60 * 1000
  const rowsAscending: DashboardRow[] = []
  const pendingStageByUser = new Map<string, Array<{ created_at: number; files: string[] }>>()

  // Process oldest -> newest so each commit can consume the closest prior stage_files.
  for (let idx = logs.length - 1; idx >= 0; idx--) {
    const log = logs[idx]
    const login = (log.user_login ?? '').trim()

    if (log.event_type === 'stage_files') {
      if (!login) continue
      const files = readDetailFiles(log)
      if (!files.length) continue
      const queue = pendingStageByUser.get(login) ?? []
      queue.push({ created_at: log.created_at, files })
      pendingStageByUser.set(login, queue)
      continue
    }

    if (log.event_type !== 'commit') continue

    let attachedFiles: string[] = []
    if (login) {
      const queue = pendingStageByUser.get(login)
      if (queue && queue.length > 0) {
        // Drop stale candidates that are too old for this commit.
        while (queue.length > 0 && (log.created_at - queue[0].created_at) > WINDOW_MS) {
          queue.shift()
        }
        if (queue.length > 0) {
          const candidate = queue.pop()
          if (candidate && log.created_at >= candidate.created_at && (log.created_at - candidate.created_at) <= WINDOW_MS) {
            attachedFiles = candidate.files
          }
        }
        if (!queue.length) pendingStageByUser.delete(login)
      }
    }

    rowsAscending.push({ log, attachedFiles })
  }

  return rowsAscending.reverse()
}
