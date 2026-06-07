import type {
  EnterpriseAdoptionPack,
  EnterpriseAdoptionProfile,
  EnterpriseAdoptionValidation,
  EnterpriseOnboardingGuide,
  EnterpriseOnboardingReadinessReport,
  EnterpriseOnboardingRemediationPlan,
  EnterpriseProviderHealthCheck,
} from '@/components/control_plane/dashboard-helpers'

export type ActionCenterGoal = 'quick-onboarding' | 'prepare-release' | 'export-evidence'
export type ActionCenterLens = 'founder' | 'developer' | 'executive' | 'platform' | 'auditor'
export type ActionCenterConfidence = 'high' | 'medium' | 'low'
export type ActionCenterRecommendationStatus = 'ready' | 'needs-action' | 'blocked'
export type ActionCenterActionKind = 'navigate' | 'review' | 'export'

export interface ActionCenterGoalOption {
  id: ActionCenterGoal
  label: string
  description: string
}

export interface ActionCenterLensOption {
  id: ActionCenterLens
  label: string
  description: string
}

export const ACTION_CENTER_GOALS: ActionCenterGoalOption[] = [
  {
    id: 'quick-onboarding',
    label: 'Onboarding',
    description: 'Get the customer setup path into a known-good state.',
  },
  {
    id: 'prepare-release',
    label: 'Release',
    description: 'Prepare release evidence and approval context.',
  },
  {
    id: 'export-evidence',
    label: 'Evidence',
    description: 'Export traceable packets, readiness, and remediation evidence.',
  },
]

export const ACTION_CENTER_LENSES: ActionCenterLensOption[] = [
  {
    id: 'founder',
    label: 'Founder',
    description: 'Prioritizes the shortest customer-success path with visible risk.',
  },
  {
    id: 'developer',
    label: 'Developer',
    description: 'Prioritizes concrete repo, workflow, and CLI tasks.',
  },
  {
    id: 'executive',
    label: 'Executive',
    description: 'Prioritizes readiness, accountable owner, and release impact.',
  },
  {
    id: 'platform',
    label: 'Platform',
    description: 'Prioritizes providers, workflow installation, and telemetry.',
  },
  {
    id: 'auditor',
    label: 'Auditor',
    description: 'Prioritizes evidence completeness and exportable records.',
  },
]

export const ACTION_CENTER_TARGETS = {
  workspace: '/',
  controlPlane: '/control-plane',
  enterpriseAdoption: '/control-plane#enterprise-adoption',
  evidencePacket: '/control-plane#evidence-packet',
  releaseApprovals: '/control-plane#release-approvals',
  governanceCopilot: '/control-plane#governance-copilot',
  settings: '/settings',
} as const

export interface ActionCenterPermission {
  label: string
  detail: string
  canAct: boolean
  requiredRole: 'Any connected user' | 'Admin'
}

export interface ActionCenterAction {
  label: string
  to: string
  kind: ActionCenterActionKind
}

export interface ActionCenterEvidenceLine {
  label: string
  value: string
  state: ActionCenterRecommendationStatus
}

export interface ActionCenterRecommendation {
  id: string
  title: string
  outcome: string
  reason: string
  status: ActionCenterRecommendationStatus
  confidence: ActionCenterConfidence
  permission: ActionCenterPermission
  primaryAction: ActionCenterAction
  evidence: ActionCenterEvidenceLine[]
  advisory: true
}

export interface ActionCenterSummary {
  customerName: string
  repositoryFullName: string
  readinessScore: number
  readinessStatus: string
  providersReady: number
  providersTotal: number
  ticketCoveragePercentage: number | null
  pipelineSuccessRate: number | null
  releaseGovernanceMode: string
  workflowTemplateCount: number
}

export interface ActionCenterPipelineInput {
  total_7d?: number
  success_7d?: number
  failure_7d?: number
}

export interface ActionCenterTicketCoverageInput {
  total_commits?: number
  commits_with_ticket?: number
  coverage_percentage?: number
  commits_without_ticket?: unknown[]
  tickets_without_commits?: unknown[]
}

export interface ActionCenterEvidencePacketInput {
  subject?: string | null
  content_hash?: string | null
  completeness?: {
    ticket_found?: boolean
    commits?: number
    pull_requests?: number
    pipelines?: number
    quality_gates?: number
    missing?: string[]
  } | null
}

export interface ActionCenterBuildInput {
  goal: ActionCenterGoal
  lens: ActionCenterLens
  isConnected: boolean
  userRole?: string | null
  profile: EnterpriseAdoptionProfile
  pack: EnterpriseAdoptionPack
  validation: EnterpriseAdoptionValidation
  providerHealth: EnterpriseProviderHealthCheck[]
  readiness: EnterpriseOnboardingReadinessReport
  remediationPlan: EnterpriseOnboardingRemediationPlan
  guide: EnterpriseOnboardingGuide
  pipeline?: ActionCenterPipelineInput | null
  ticketCoverage?: ActionCenterTicketCoverageInput | null
  evidencePacket?: ActionCenterEvidencePacketInput | null
  releaseApprovalsTotal?: number | null
}

export interface ActionCenterGuidance {
  goal: ActionCenterGoal
  lens: ActionCenterLens
  lensNote: string
  primary: ActionCenterRecommendation
  secondary: ActionCenterRecommendation[]
  summary: ActionCenterSummary
}

const LENS_NOTES: Record<ActionCenterLens, string> = {
  founder: 'Use this lens to keep the next customer-success move visible without hiding alternatives.',
  developer: 'Use this lens to turn guidance into repo, workflow, CLI, or evidence tasks.',
  executive: 'Use this lens to explain release posture, owner, and impact without changing policy.',
  platform: 'Use this lens to focus on provider readiness, telemetry, and workflow installation.',
  auditor: 'Use this lens to keep exportable evidence and traceability completeness first.',
}

function permissionForAdminAction(userRole?: string | null): ActionCenterPermission {
  const normalizedRole = userRole?.trim() || 'Unknown'
  const canAct = normalizedRole === 'Admin'
  return {
    label: canAct ? 'Admin action available' : 'Admin action',
    detail: canAct
      ? 'Your current Control Plane role can execute this workflow.'
      : `${normalizedRole} can use the guidance; an Admin must execute the target workflow.`,
    canAct,
    requiredRole: 'Admin',
  }
}

function permissionForNavigation(isConnected: boolean): ActionCenterPermission {
  return {
    label: isConnected ? 'Open navigation' : 'Connect first',
    detail: isConnected
      ? 'This is a navigation step and does not mutate provider state.'
      : 'Connect the Control Plane before GitGov can read current evidence.',
    canAct: isConnected,
    requiredRole: 'Any connected user',
  }
}

function action(label: string, to: string, kind: ActionCenterActionKind): ActionCenterAction {
  return { label, to, kind }
}

function evidence(
  label: string,
  value: string,
  state: ActionCenterRecommendationStatus,
): ActionCenterEvidenceLine {
  return { label, value, state }
}

function recommendation(params: {
  id: string
  title: string
  outcome: string
  reason: string
  status: ActionCenterRecommendationStatus
  confidence: ActionCenterConfidence
  permission: ActionCenterPermission
  primaryAction: ActionCenterAction
  evidence: ActionCenterEvidenceLine[]
}): ActionCenterRecommendation {
  return {
    ...params,
    advisory: true,
  }
}

function pipelineSuccessRate(pipeline?: ActionCenterPipelineInput | null): number | null {
  const total = pipeline?.total_7d ?? 0
  if (total <= 0) return null
  return Math.round(((pipeline?.success_7d ?? 0) / total) * 1000) / 10
}

function evidencePacketMissingCount(packet?: ActionCenterEvidencePacketInput | null): number | null {
  if (!packet?.content_hash) return null
  return packet.completeness?.missing?.length ?? 0
}

function summarize(input: ActionCenterBuildInput): ActionCenterSummary {
  return {
    customerName: input.profile.customer_name,
    repositoryFullName: input.profile.repository_full_name,
    readinessScore: input.readiness.readiness_score,
    readinessStatus: input.readiness.status,
    providersReady: input.providerHealth.filter((check) => check.status === 'ready').length,
    providersTotal: input.providerHealth.length,
    ticketCoveragePercentage: typeof input.ticketCoverage?.coverage_percentage === 'number'
      ? input.ticketCoverage.coverage_percentage
      : null,
    pipelineSuccessRate: pipelineSuccessRate(input.pipeline),
    releaseGovernanceMode: input.pack.release_governance.mode,
    workflowTemplateCount: input.pack.workflow_plan.length,
  }
}

function profileRecommendation(input: ActionCenterBuildInput): ActionCenterRecommendation {
  const errors = input.validation.errors.slice(0, 3)
  return recommendation({
    id: 'complete-profile',
    title: 'Complete the enterprise adoption profile',
    outcome: 'GitGov can scope provider checks, workflow packs, release policy, and evidence exports to the right repo.',
    reason: 'The profile is the root context for the guided path. Without it, downstream recommendations are lower confidence.',
    status: 'blocked',
    confidence: 'high',
    permission: permissionForAdminAction(input.userRole),
    primaryAction: action('Open adoption profile', ACTION_CENTER_TARGETS.enterpriseAdoption, 'review'),
    evidence: [
      evidence('Profile validation', `${errors.length || input.validation.errors.length} issue(s)`, 'blocked'),
      evidence('Next detail', errors.join(' ') || 'Review required customer, repository, branch, Jira key, providers, and modules.', 'needs-action'),
    ],
  })
}

function providerConfigRecommendation(input: ActionCenterBuildInput): ActionCenterRecommendation {
  const firstConfigIssue = input.providerHealth.find((check) => check.status === 'needs-config')
  return recommendation({
    id: 'complete-provider-config',
    title: 'Complete provider configuration names',
    outcome: 'The selected providers can be checked without reading or storing secret values.',
    reason: 'Provider checks need declared variable, secret, repository, and policy context before evidence can be trusted.',
    status: 'needs-action',
    confidence: 'high',
    permission: permissionForAdminAction(input.userRole),
    primaryAction: action('Review provider setup', ACTION_CENTER_TARGETS.enterpriseAdoption, 'review'),
    evidence: [
      evidence('Provider', firstConfigIssue?.label ?? 'Selected provider', 'needs-action'),
      evidence('Required next step', firstConfigIssue?.next_step ?? 'Complete required provider configuration.', 'needs-action'),
    ],
  })
}

function providerEvidenceRecommendation(input: ActionCenterBuildInput): ActionCenterRecommendation {
  const firstEvidenceIssue = input.providerHealth.find((check) => check.status === 'needs-evidence')
  return recommendation({
    id: 'collect-provider-evidence',
    title: 'Collect provider health evidence',
    outcome: 'GitGov can distinguish a configured provider from a provider that has actually produced usable evidence.',
    reason: 'The Action Center should guide by observed evidence, not by assuming a provider is healthy because it was selected.',
    status: 'needs-action',
    confidence: input.providerHealth.length > 0 ? 'medium' : 'low',
    permission: permissionForAdminAction(input.userRole),
    primaryAction: action('Open provider health', ACTION_CENTER_TARGETS.enterpriseAdoption, 'review'),
    evidence: [
      evidence('Provider', firstEvidenceIssue?.label ?? 'Provider evidence', 'needs-action'),
      evidence('Observed evidence', firstEvidenceIssue?.evidence ?? 'No selected provider evidence is loaded.', 'needs-action'),
    ],
  })
}

function onboardingNextStepRecommendation(input: ActionCenterBuildInput): ActionCenterRecommendation {
  const nextStep = input.guide.next_step
  const isBlocked = nextStep?.status === 'blocked'
  return recommendation({
    id: 'continue-onboarding-checklist',
    title: nextStep ? `Continue: ${nextStep.label}` : 'Export the reviewed workflow pack',
    outcome: nextStep
      ? 'The next onboarding move is visible, owned, and validated without blocking other actions.'
      : 'The onboarding path is ready to hand off as reviewed customer setup evidence.',
    reason: nextStep
      ? nextStep.action
      : `${input.pack.workflow_plan.length} workflow templates are available for review and export.`,
    status: isBlocked ? 'blocked' : nextStep ? 'needs-action' : 'ready',
    confidence: 'high',
    permission: permissionForAdminAction(input.userRole),
    primaryAction: action(
      nextStep ? 'Open guided checklist' : 'Open workflow pack',
      ACTION_CENTER_TARGETS.enterpriseAdoption,
      nextStep ? 'review' : 'export',
    ),
    evidence: [
      evidence('Readiness score', `${input.guide.readiness_score}/100`, input.readiness.status === 'ready' ? 'ready' : 'needs-action'),
      evidence('Checklist progress', `${input.guide.completed_steps}/${input.guide.total_steps} complete`, nextStep ? 'needs-action' : 'ready'),
    ],
  })
}

function quickOnboardingPrimary(input: ActionCenterBuildInput): ActionCenterRecommendation {
  if (!input.validation.valid) return profileRecommendation(input)
  if (input.providerHealth.some((check) => check.status === 'needs-config')) {
    return providerConfigRecommendation(input)
  }
  if (input.providerHealth.some((check) => check.status === 'needs-evidence')) {
    return providerEvidenceRecommendation(input)
  }
  return onboardingNextStepRecommendation(input)
}

function pipelineRecommendation(input: ActionCenterBuildInput): ActionCenterRecommendation {
  const pipeline = input.pipeline
  const total = pipeline?.total_7d ?? 0
  const failures = pipeline?.failure_7d ?? 0
  const successRate = pipelineSuccessRate(pipeline)
  const needsAction = total === 0 || failures > 0 || (successRate !== null && successRate < 90)
  return recommendation({
    id: total === 0 ? 'collect-pipeline-evidence' : 'review-pipeline-health',
    title: total === 0 ? 'Collect pipeline evidence before release' : 'Review pipeline health before release',
    outcome: 'Release guidance is tied to observed CI evidence instead of a manual confidence guess.',
    reason: total === 0
      ? 'No pipeline run is loaded for the current 7 day window.'
      : `${failures} failing run(s) and ${successRate ?? 0}% success in the current 7 day window.`,
    status: needsAction ? 'needs-action' : 'ready',
    confidence: total === 0 ? 'low' : 'high',
    permission: permissionForNavigation(input.isConnected),
    primaryAction: action('Open pipeline dashboard', ACTION_CENTER_TARGETS.controlPlane, 'review'),
    evidence: [
      evidence('Pipeline runs 7d', String(total), total > 0 ? 'ready' : 'needs-action'),
      evidence('Success rate', successRate === null ? 'N/A' : `${successRate}%`, needsAction ? 'needs-action' : 'ready'),
    ],
  })
}

function traceabilityRecommendation(input: ActionCenterBuildInput): ActionCenterRecommendation {
  const coverage = input.ticketCoverage?.coverage_percentage
  const total = input.ticketCoverage?.total_commits ?? 0
  const missingCommits = input.ticketCoverage?.commits_without_ticket?.length ?? 0
  return recommendation({
    id: 'repair-traceability-coverage',
    title: 'Repair Jira traceability before release',
    outcome: 'The release has a clearer ticket-to-code story before approval or evidence export.',
    reason: total > 0
      ? `${missingCommits} commit(s) are missing ticket evidence in the current coverage window.`
      : 'Ticket coverage is not loaded yet, so release confidence should stay conservative.',
    status: coverage !== undefined && coverage >= 85 ? 'ready' : 'needs-action',
    confidence: total > 0 ? 'high' : 'low',
    permission: permissionForNavigation(input.isConnected),
    primaryAction: action('Open ticket coverage', ACTION_CENTER_TARGETS.controlPlane, 'review'),
    evidence: [
      evidence('Coverage', coverage === undefined ? 'N/A' : `${coverage.toFixed(2)}%`, coverage !== undefined && coverage >= 85 ? 'ready' : 'needs-action'),
      evidence('Window commits', String(total), total > 0 ? 'ready' : 'needs-action'),
    ],
  })
}

function evidencePacketRecommendation(input: ActionCenterBuildInput): ActionCenterRecommendation {
  const missingCount = evidencePacketMissingCount(input.evidencePacket)
  const hasPacket = Boolean(input.evidencePacket?.content_hash)
  const isComplete = hasPacket && missingCount === 0
  return recommendation({
    id: hasPacket ? 'review-current-evidence-packet' : 'generate-evidence-packet',
    title: hasPacket ? 'Review the current Evidence Packet' : 'Generate a ticket Evidence Packet',
    outcome: hasPacket
      ? 'Release and audit conversations can cite a concrete packet hash.'
      : 'The release path gets a traceable ticket-scoped evidence record.',
    reason: hasPacket
      ? `${input.evidencePacket?.subject ?? 'Current ticket'} has ${missingCount ?? 0} missing evidence area(s).`
      : 'No ticket Evidence Packet is currently loaded in the dashboard state.',
    status: isComplete ? 'ready' : 'needs-action',
    confidence: hasPacket ? 'high' : 'medium',
    permission: permissionForAdminAction(input.userRole),
    primaryAction: action(hasPacket ? 'Open packet' : 'Generate packet', ACTION_CENTER_TARGETS.evidencePacket, hasPacket ? 'review' : 'export'),
    evidence: [
      evidence('Packet hash', hasPacket ? String(input.evidencePacket?.content_hash).slice(0, 12) : 'Not loaded', hasPacket ? 'ready' : 'needs-action'),
      evidence('Missing areas', missingCount === null ? 'N/A' : String(missingCount), isComplete ? 'ready' : 'needs-action'),
    ],
  })
}

function releaseApprovalRecommendation(input: ActionCenterBuildInput): ActionCenterRecommendation {
  const approvalsTotal = input.releaseApprovalsTotal ?? 0
  return recommendation({
    id: 'record-release-decision',
    title: 'Record the release decision with evidence',
    outcome: 'A release approval or accepted-risk decision becomes traceable to the evidence packet hash.',
    reason: input.pack.release_governance.enforcement === 'disabled'
      ? 'Release governance is record-only by default, so this records evidence without blocking delivery.'
      : 'The configured release governance policy should be evaluated before treating the release as ready.',
    status: 'needs-action',
    confidence: input.evidencePacket?.content_hash ? 'high' : 'medium',
    permission: permissionForAdminAction(input.userRole),
    primaryAction: action('Open release approvals', ACTION_CENTER_TARGETS.releaseApprovals, 'review'),
    evidence: [
      evidence('Governance mode', input.pack.release_governance.mode, input.pack.release_governance.enforcement === 'disabled' ? 'ready' : 'needs-action'),
      evidence('Stored approvals', String(approvalsTotal), approvalsTotal > 0 ? 'ready' : 'needs-action'),
    ],
  })
}

function prepareReleasePrimary(input: ActionCenterBuildInput): ActionCenterRecommendation {
  if (!input.validation.valid) return profileRecommendation(input)

  const pipeline = input.pipeline
  const pipelineRate = pipelineSuccessRate(pipeline)
  if ((pipeline?.total_7d ?? 0) === 0 || (pipeline?.failure_7d ?? 0) > 0 || (pipelineRate !== null && pipelineRate < 90)) {
    return pipelineRecommendation(input)
  }

  const coverage = input.ticketCoverage?.coverage_percentage
  if (coverage !== undefined && coverage < 85) {
    return traceabilityRecommendation(input)
  }

  if (!input.evidencePacket?.content_hash || evidencePacketMissingCount(input.evidencePacket) !== 0) {
    return evidencePacketRecommendation(input)
  }

  return releaseApprovalRecommendation(input)
}

function exportEvidencePrimary(input: ActionCenterBuildInput): ActionCenterRecommendation {
  if (!input.validation.valid) return profileRecommendation(input)
  if (input.evidencePacket?.content_hash) return evidencePacketRecommendation(input)

  if (input.readiness.status !== 'ready') {
    return recommendation({
      id: 'export-readiness-remediation',
      title: 'Export readiness and remediation evidence',
      outcome: 'The customer gets a concrete list of what is ready, what is missing, and who owns each next step.',
      reason: `${input.remediationPlan.action_count} remediation action(s) remain before onboarding is fully ready.`,
      status: 'needs-action',
      confidence: 'high',
      permission: permissionForAdminAction(input.userRole),
      primaryAction: action('Open readiness exports', ACTION_CENTER_TARGETS.enterpriseAdoption, 'export'),
      evidence: [
        evidence('Readiness score', `${input.readiness.readiness_score}/100`, 'needs-action'),
        evidence('Remediation actions', String(input.remediationPlan.action_count), input.remediationPlan.action_count === 0 ? 'ready' : 'needs-action'),
      ],
    })
  }

  return evidencePacketRecommendation(input)
}

function workspaceRecommendation(input: ActionCenterBuildInput): ActionCenterRecommendation {
  return recommendation({
    id: 'use-workspace-cli',
    title: 'Use the workspace CLI for manual repo work',
    outcome: 'Manual commands stay next to files, diff context, branch state, and commit/push controls.',
    reason: 'The Action Center should guide the next move, while the existing dashboard remains the hands-on execution surface.',
    status: 'ready',
    confidence: 'high',
    permission: permissionForNavigation(input.isConnected),
    primaryAction: action('Open workspace', ACTION_CENTER_TARGETS.workspace, 'navigate'),
    evidence: [
      evidence('Workspace role', 'CLI, pipeline visualizer, commit and push controls', 'ready'),
    ],
  })
}

function copilotExplanationRecommendation(input: ActionCenterBuildInput): ActionCenterRecommendation {
  return recommendation({
    id: 'explain-with-copilot',
    title: 'Ask the copilot to explain the evidence',
    outcome: 'AI can explain the recommendation with citations, while the deterministic Action Center keeps decision control.',
    reason: 'Use this when the next step needs a plain-language explanation for a stakeholder.',
    status: 'ready',
    confidence: 'medium',
    permission: permissionForAdminAction(input.userRole),
    primaryAction: action('Open copilot', ACTION_CENTER_TARGETS.governanceCopilot, 'review'),
    evidence: [
      evidence('Decision owner', 'Operator, not AI', 'ready'),
      evidence('Output style', 'Citation-grounded explanation', 'ready'),
    ],
  })
}

function secondaryRecommendations(input: ActionCenterBuildInput): ActionCenterRecommendation[] {
  if (input.goal === 'quick-onboarding') {
    return [
      traceabilityRecommendation(input),
      workspaceRecommendation(input),
      copilotExplanationRecommendation(input),
    ]
  }
  if (input.goal === 'prepare-release') {
    return [
      evidencePacketRecommendation(input),
      releaseApprovalRecommendation(input),
      copilotExplanationRecommendation(input),
    ]
  }
  return [
    evidencePacketRecommendation(input),
    onboardingNextStepRecommendation(input),
    workspaceRecommendation(input),
  ]
}

function disconnectedRecommendation(input: ActionCenterBuildInput): ActionCenterRecommendation {
  return recommendation({
    id: 'connect-control-plane',
    title: 'Connect the Control Plane',
    outcome: 'GitGov can load current evidence before recommending a next action.',
    reason: 'The Action Center is deterministic, so it needs current server evidence instead of guessing from stale UI state.',
    status: 'blocked',
    confidence: 'high',
    permission: permissionForNavigation(input.isConnected),
    primaryAction: action('Open connection settings', ACTION_CENTER_TARGETS.controlPlane, 'navigate'),
    evidence: [
      evidence('Connection', 'Disconnected', 'blocked'),
      evidence('Available guidance', 'Navigation only', 'needs-action'),
    ],
  })
}

export function buildActionCenterGuidance(input: ActionCenterBuildInput): ActionCenterGuidance {
  const primary = !input.isConnected
    ? disconnectedRecommendation(input)
    : input.goal === 'quick-onboarding'
      ? quickOnboardingPrimary(input)
      : input.goal === 'prepare-release'
        ? prepareReleasePrimary(input)
        : exportEvidencePrimary(input)

  const secondary = secondaryRecommendations(input)
    .filter((candidate) => candidate.id !== primary.id)
    .slice(0, 3)

  return {
    goal: input.goal,
    lens: input.lens,
    lensNote: LENS_NOTES[input.lens],
    primary,
    secondary,
    summary: summarize(input),
  }
}
