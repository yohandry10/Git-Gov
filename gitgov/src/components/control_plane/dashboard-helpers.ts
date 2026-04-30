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

export interface EnterpriseAdoptionProfile {
  customer_name: string
  repository_full_name: string
  default_branch: string
  jira_project_key: string
  policy_preset: AdoptionPolicyPreset
  providers: AdoptionProvider[]
  modules: AdoptionModule[]
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
  providers: AdoptionProvider[]
  modules: AdoptionModule[]
  workflow_plan: EnterpriseAdoptionWorkflowPlan[]
  variables: EnterpriseAdoptionVariable[]
  secrets: EnterpriseAdoptionSecret[]
  policy_rules: EnterpriseAdoptionPolicyRule[]
  manual_steps: EnterpriseAdoptionManualStep[]
  open_product_gaps: EnterpriseAdoptionProductGap[]
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

export const DEFAULT_ENTERPRISE_ADOPTION_PROFILE: EnterpriseAdoptionProfile = {
  customer_name: 'ExampleCo',
  repository_full_name: 'example-org/example-repo',
  default_branch: 'main',
  jira_project_key: 'EX',
  policy_preset: 'moderate',
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

const ADOPTION_PROVIDER_IDS = ADOPTION_PROVIDER_OPTIONS.map((option) => option.id)
const ADOPTION_MODULE_IDS = ADOPTION_MODULE_OPTIONS.map((option) => option.id)

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
    addUniqueByKey(openProductGaps, {
      gap: 'Formal release approval',
      detail: 'GitGov has PR review evidence and policy decisions, but a full enterprise release approval model still needs approvers, expiration, risk acceptance, and evidence binding.',
    }, 'gap')
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
