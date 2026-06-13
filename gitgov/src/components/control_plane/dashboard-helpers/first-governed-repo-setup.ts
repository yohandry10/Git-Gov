export type FirstGovernedRepoSetupGoal =
  | 'govern_release'
  | 'generate_audit_evidence'
  | 'standardize_workflows'
  | 'assess_governance_gaps'

export type FirstGovernedRepoSetupStatus = 'draft' | 'ready' | 'blocked' | 'completed'
export type FirstGovernedRepoPolicyPreset = 'audit-only' | 'moderate' | 'strict'
export type FirstGovernedRepoProvider = 'github' | 'jira' | 'jenkins' | 'sonarqube' | 'render' | 'vercel'
export type FirstGovernedRepoModule =
  | 'traceability'
  | 'github-evidence'
  | 'release-readiness'
  | 'quality-gates'
  | 'evidence-packets'
  | 'formal-approval'
export type FirstGovernedRepoGateReadiness = 'needs_repo' | 'needs_preview' | 'baseline_ready'

export interface FirstGovernedRepoOption<T extends string> {
  id: T
  label: string
  description: string
  required?: boolean
}

export interface FirstGovernedRepoSetupBaseline extends Record<string, unknown> {
  version: 1
  policy_workflow_preview_acknowledged: boolean
  gate_readiness: FirstGovernedRepoGateReadiness
  setup_summary: {
    repository_full_name: string
    default_branch: string
    goal: FirstGovernedRepoSetupGoal
    policy_preset: FirstGovernedRepoPolicyPreset
    provider_count: number
    module_count: number
    github_selected: boolean
    policy_workflow_preview_acknowledged: boolean
  }
  action_center_gaps: string[]
  first_result: {
    status: 'ready_for_advisory_gate' | 'needs_setup'
    deployment_gate_mode: 'advisory'
    cta: 'simulate_deployment_gate'
    evidence_contract: {
      repo: string
      branch: string
      providers: FirstGovernedRepoProvider[]
      modules: FirstGovernedRepoModule[]
    }
  }
}

export interface FirstGovernedRepoSetupDraft {
  status: FirstGovernedRepoSetupStatus
  goal: FirstGovernedRepoSetupGoal
  repository_full_name: string
  default_branch: string
  selected_providers: FirstGovernedRepoProvider[]
  selected_modules: FirstGovernedRepoModule[]
  policy_preset: FirstGovernedRepoPolicyPreset
  baseline: FirstGovernedRepoSetupBaseline
}

export interface FirstGovernedRepoSetupValidation {
  ready: boolean
  gateReadiness: FirstGovernedRepoGateReadiness
  gaps: string[]
  errors: string[]
}

export const FIRST_GOVERNED_REPO_GOAL_OPTIONS: Array<FirstGovernedRepoOption<FirstGovernedRepoSetupGoal>> = [
  {
    id: 'govern_release',
    label: 'Govern release',
    description: 'Prepare the first repo for advisory deployment gate evidence.',
  },
  {
    id: 'generate_audit_evidence',
    label: 'Audit evidence',
    description: 'Prioritize traceable evidence packets and provider proof.',
  },
  {
    id: 'standardize_workflows',
    label: 'Workflow baseline',
    description: 'Standardize reviewed workflow and policy installation steps.',
  },
  {
    id: 'assess_governance_gaps',
    label: 'Gap assessment',
    description: 'Expose missing controls before gate simulation.',
  },
]

export const FIRST_GOVERNED_REPO_PROVIDER_OPTIONS: Array<FirstGovernedRepoOption<FirstGovernedRepoProvider>> = [
  {
    id: 'github',
    label: 'GitHub',
    description: 'Repo, PR, status, and webhook evidence.',
    required: true,
  },
  {
    id: 'jira',
    label: 'Jira',
    description: 'Ticket traceability and release context.',
  },
  {
    id: 'jenkins',
    label: 'Jenkins',
    description: 'Pipeline run and build outcome evidence.',
  },
  {
    id: 'sonarqube',
    label: 'SonarQube',
    description: 'Quality gate evidence inferred from pipeline signals.',
  },
  {
    id: 'render',
    label: 'Render',
    description: 'Deployment target evidence when configured later.',
  },
  {
    id: 'vercel',
    label: 'Vercel',
    description: 'Deployment target evidence when configured later.',
  },
]

export const FIRST_GOVERNED_REPO_MODULE_OPTIONS: Array<FirstGovernedRepoOption<FirstGovernedRepoModule>> = [
  {
    id: 'traceability',
    label: 'Traceability',
    description: 'Ticket-to-code coverage.',
    required: true,
  },
  {
    id: 'release-readiness',
    label: 'Release readiness',
    description: 'Advisory readiness score input.',
    required: true,
  },
  {
    id: 'evidence-packets',
    label: 'Evidence packets',
    description: 'Downloadable audit packet proof.',
    required: true,
  },
  {
    id: 'quality-gates',
    label: 'Quality gates',
    description: 'CI quality pass/fail evidence.',
  },
  {
    id: 'github-evidence',
    label: 'GitHub evidence',
    description: 'PR review and check-run signal depth.',
  },
  {
    id: 'formal-approval',
    label: 'Formal approval',
    description: 'Release approval record before hard enforcement.',
  },
]

export const FIRST_GOVERNED_REPO_POLICY_PRESET_OPTIONS: Array<FirstGovernedRepoOption<FirstGovernedRepoPolicyPreset>> = [
  {
    id: 'audit-only',
    label: 'Audit only',
    description: 'Collect evidence without advisory warnings.',
  },
  {
    id: 'moderate',
    label: 'Moderate',
    description: 'Recommended advisory deployment gate baseline.',
  },
  {
    id: 'strict',
    label: 'Strict',
    description: 'Prepare for future blocking controls.',
  },
]

export const DEFAULT_FIRST_GOVERNED_REPO_SETUP: FirstGovernedRepoSetupDraft = {
  status: 'draft',
  goal: 'govern_release',
  repository_full_name: '',
  default_branch: 'main',
  selected_providers: ['github'],
  selected_modules: ['traceability', 'release-readiness', 'evidence-packets'],
  policy_preset: 'moderate',
  baseline: buildFirstGovernedRepoSetupBaseline({
    repository_full_name: '',
    default_branch: 'main',
    goal: 'govern_release',
    selected_providers: ['github'],
    selected_modules: ['traceability', 'release-readiness', 'evidence-packets'],
    policy_preset: 'moderate',
    policyWorkflowPreviewAcknowledged: false,
  }),
}

function dedupeAllowed<T extends string>(values: unknown, allowed: readonly T[], fallback: T[]): T[] {
  if (!Array.isArray(values)) return fallback
  const allowedSet = new Set(allowed)
  const seen = new Set<T>()
  const result: T[] = []
  for (const value of values) {
    if (typeof value !== 'string') continue
    const trimmed = value.trim() as T
    if (!allowedSet.has(trimmed) || seen.has(trimmed)) continue
    seen.add(trimmed)
    result.push(trimmed)
  }
  return result.length > 0 ? result : fallback
}

export function isFirstGovernedRepoNameValid(repositoryFullName: string): boolean {
  const value = repositoryFullName.trim()
  if (value.length === 0 || value.length > 200 || /\s/.test(value)) return false
  const parts = value.split('/')
  if (parts.length !== 2) return false
  return parts.every((part) => /^[A-Za-z0-9_.-]{1,100}$/.test(part) && !part.startsWith('.') && !part.endsWith('.'))
}

export function buildFirstGovernedRepoSetupBaseline(input: {
  repository_full_name: string
  default_branch: string
  goal: FirstGovernedRepoSetupGoal
  selected_providers: FirstGovernedRepoProvider[]
  selected_modules: FirstGovernedRepoModule[]
  policy_preset: FirstGovernedRepoPolicyPreset
  policyWorkflowPreviewAcknowledged: boolean
}): FirstGovernedRepoSetupBaseline {
  const repositoryFullName = input.repository_full_name.trim()
  const defaultBranch = input.default_branch.trim() || 'main'
  const providers = input.selected_providers.includes('github')
    ? input.selected_providers
    : (['github', ...input.selected_providers] as FirstGovernedRepoProvider[])
  const modules = input.selected_modules
  const repoReady = isFirstGovernedRepoNameValid(repositoryFullName)
  const previewReady = input.policyWorkflowPreviewAcknowledged
  const gateReadiness: FirstGovernedRepoGateReadiness =
    repoReady && previewReady ? 'baseline_ready' : repoReady ? 'needs_preview' : 'needs_repo'
  const gaps: string[] = []
  if (!repoReady) gaps.push('repository_full_name')
  if (!previewReady) gaps.push('policy_workflow_preview')
  if (!modules.includes('quality-gates')) gaps.push('quality_gate_evidence')
  if (!modules.includes('formal-approval')) gaps.push('formal_approval_policy')

  return {
    version: 1,
    policy_workflow_preview_acknowledged: previewReady,
    gate_readiness: gateReadiness,
    setup_summary: {
      repository_full_name: repositoryFullName,
      default_branch: defaultBranch,
      goal: input.goal,
      policy_preset: input.policy_preset,
      provider_count: providers.length,
      module_count: modules.length,
      github_selected: providers.includes('github'),
      policy_workflow_preview_acknowledged: previewReady,
    },
    action_center_gaps: gaps,
    first_result: {
      status: gateReadiness === 'baseline_ready' ? 'ready_for_advisory_gate' : 'needs_setup',
      deployment_gate_mode: 'advisory',
      cta: 'simulate_deployment_gate',
      evidence_contract: {
        repo: repositoryFullName,
        branch: defaultBranch,
        providers,
        modules,
      },
    },
  }
}

export function normalizeFirstGovernedRepoSetupDraft(input?: Partial<FirstGovernedRepoSetupDraft> | null): FirstGovernedRepoSetupDraft {
  const allowedGoals = FIRST_GOVERNED_REPO_GOAL_OPTIONS.map((option) => option.id)
  const allowedProviders = FIRST_GOVERNED_REPO_PROVIDER_OPTIONS.map((option) => option.id)
  const allowedModules = FIRST_GOVERNED_REPO_MODULE_OPTIONS.map((option) => option.id)
  const allowedPresets = FIRST_GOVERNED_REPO_POLICY_PRESET_OPTIONS.map((option) => option.id)
  const selectedProviders = dedupeAllowed(
    input?.selected_providers,
    allowedProviders,
    DEFAULT_FIRST_GOVERNED_REPO_SETUP.selected_providers,
  )
  const providers = selectedProviders.includes('github')
    ? selectedProviders
    : (['github', ...selectedProviders] as FirstGovernedRepoProvider[])
  const modules = dedupeAllowed(
    input?.selected_modules,
    allowedModules,
    DEFAULT_FIRST_GOVERNED_REPO_SETUP.selected_modules,
  )
  const goal = allowedGoals.includes(input?.goal as FirstGovernedRepoSetupGoal)
    ? (input?.goal as FirstGovernedRepoSetupGoal)
    : DEFAULT_FIRST_GOVERNED_REPO_SETUP.goal
  const policyPreset = allowedPresets.includes(input?.policy_preset as FirstGovernedRepoPolicyPreset)
    ? (input?.policy_preset as FirstGovernedRepoPolicyPreset)
    : DEFAULT_FIRST_GOVERNED_REPO_SETUP.policy_preset
  const previewAck =
    Boolean(input?.baseline?.policy_workflow_preview_acknowledged) ||
    input?.baseline?.gate_readiness === 'baseline_ready'
  const baseline = buildFirstGovernedRepoSetupBaseline({
    repository_full_name: input?.repository_full_name ?? '',
    default_branch: input?.default_branch ?? 'main',
    goal,
    selected_providers: providers,
    selected_modules: modules,
    policy_preset: policyPreset,
    policyWorkflowPreviewAcknowledged: previewAck,
  })

  return {
    status: baseline.gate_readiness === 'baseline_ready' ? 'ready' : 'draft',
    goal,
    repository_full_name: input?.repository_full_name?.trim() ?? '',
    default_branch: input?.default_branch?.trim() || 'main',
    selected_providers: providers,
    selected_modules: modules,
    policy_preset: policyPreset,
    baseline,
  }
}

export function validateFirstGovernedRepoSetupDraft(
  draft: FirstGovernedRepoSetupDraft,
): FirstGovernedRepoSetupValidation {
  const baseline = buildFirstGovernedRepoSetupBaseline({
    repository_full_name: draft.repository_full_name,
    default_branch: draft.default_branch,
    goal: draft.goal,
    selected_providers: draft.selected_providers,
    selected_modules: draft.selected_modules,
    policy_preset: draft.policy_preset,
    policyWorkflowPreviewAcknowledged: draft.baseline.policy_workflow_preview_acknowledged,
  })
  const errors: string[] = []
  if (!isFirstGovernedRepoNameValid(draft.repository_full_name)) {
    errors.push('Repository must use owner/repo format.')
  }
  if (!draft.selected_providers.includes('github')) {
    errors.push('GitHub is required for the first governed repo.')
  }
  if (draft.default_branch.trim().length === 0 || /\s/.test(draft.default_branch)) {
    errors.push('Default branch must not be empty or contain whitespace.')
  }

  return {
    ready: baseline.gate_readiness === 'baseline_ready' && errors.length === 0,
    gateReadiness: baseline.gate_readiness,
    gaps: baseline.action_center_gaps,
    errors,
  }
}
