import {
  ONBOARDING_STAGE_IDS,
  ONBOARDING_TRACKING_STATUSES,
  normalizeEnterpriseAdoptionProfile,
  validateEnterpriseAdoptionProfile,
  type EnterpriseAdoptionPack,
  type EnterpriseAdoptionProfile,
  type EnterpriseOnboardingChecklistTracking,
  type EnterpriseOnboardingChecklistTrackingItem,
  type EnterpriseOnboardingConfigurationCommand,
  type EnterpriseOnboardingGuide,
  type EnterpriseOnboardingGuideStep,
  type EnterpriseOnboardingGuideStepStatus,
  type EnterpriseOnboardingReadinessReport,
  type EnterpriseOnboardingReadinessStage,
  type EnterpriseOnboardingReadinessStageId,
  type EnterpriseOnboardingReadinessStatus,
  type EnterpriseOnboardingRemediationAction,
  type EnterpriseOnboardingRemediationPlan,
  type EnterpriseProviderHealthCheck,
  type EnterpriseWorkflowInstallationReadinessInput,
} from './adoption-profile'
import { buildEnterpriseAdoptionPack } from './adoption-pack'
import { buildEnterpriseProviderHealth } from './provider-health'

function readinessStageWeight(status: EnterpriseOnboardingReadinessStatus): number {
  if (status === 'ready') return 1
  if (status === 'needs-action') return 0.5
  return 0
}

function numericReadinessTotal(value?: number): number {
  return Number.isFinite(value) ? Math.max(0, Math.trunc(value ?? 0)) : 0
}

function workflowReadinessStatus(
  workflowReadiness?: EnterpriseWorkflowInstallationReadinessInput | null,
): EnterpriseOnboardingReadinessStatus {
  if (!workflowReadiness) return 'needs-action'
  return workflowReadiness.status === 'ready' ? 'ready' : 'needs-action'
}

function workflowReadinessTotals(workflowReadiness?: EnterpriseWorkflowInstallationReadinessInput | null) {
  return {
    workflowsMissing: numericReadinessTotal(workflowReadiness?.totals?.workflows_missing),
    workflowsDifferent: numericReadinessTotal(workflowReadiness?.totals?.workflows_different),
    variablesMissing: numericReadinessTotal(workflowReadiness?.totals?.variables_missing),
    secretsMissing: numericReadinessTotal(workflowReadiness?.totals?.secrets_missing),
  }
}

export function buildEnterpriseOnboardingReadinessReport(
  profile: EnterpriseAdoptionProfile,
  providerHealth: EnterpriseProviderHealthCheck[] = buildEnterpriseProviderHealth(profile),
  workflowReadiness?: EnterpriseWorkflowInstallationReadinessInput | null,
  generatedAt = new Date().toISOString(),
): EnterpriseOnboardingReadinessReport {
  const normalizedProfile = normalizeEnterpriseAdoptionProfile(profile)
  const validation = validateEnterpriseAdoptionProfile(normalizedProfile)
  const pack = buildEnterpriseAdoptionPack(normalizedProfile, generatedAt)
  const workflowTotals = workflowReadinessTotals(workflowReadiness)
  const actionConfigMissing = workflowTotals.variablesMissing + workflowTotals.secretsMissing
  const hasRequiredActionConfig = pack.variables.length + pack.secrets.length > 0
  const readyProviders = providerHealth.filter((check) => check.status === 'ready').length
  const providerConfigIssues = providerHealth.filter((check) => check.status === 'needs-config').length
  const providerEvidenceIssues = providerHealth.filter((check) => check.status === 'needs-evidence').length
  const releaseGovernance = pack.release_governance
  const stages: EnterpriseOnboardingReadinessStage[] = [
    {
      id: 'profile',
      label: 'Adoption profile',
      status: validation.valid ? 'ready' : 'blocked',
      summary: validation.valid
        ? `${pack.customer_name} profile targets ${pack.repository_full_name}:${pack.default_branch}`
        : `${validation.errors.length} profile validation issue(s)`,
      next_action: validation.valid ? 'Keep profile saved before generating customer artifacts.' : validation.errors.join(' '),
    },
    {
      id: 'providers',
      label: 'Provider evidence',
      status: providerHealth.length > 0 && readyProviders === providerHealth.length ? 'ready' : 'needs-action',
      summary: `${readyProviders}/${providerHealth.length} selected provider(s) ready`,
      next_action: providerConfigIssues > 0
        ? 'Complete required provider configuration names and connection setup.'
        : providerEvidenceIssues > 0
          ? 'Run provider validation and wait for GitGov evidence ingestion.'
          : 'Select at least one provider for onboarding validation.',
    },
    {
      id: 'workflow-pack',
      label: 'Workflow template pack',
      status: pack.workflow_plan.length > 0 ? 'ready' : 'needs-action',
      summary: `${pack.workflow_plan.length} workflow template(s), ${pack.variables.length} variable name(s), ${pack.secrets.length} secret name(s)`,
      next_action: pack.workflow_plan.length > 0
        ? 'Review the generated workflow pack before installation.'
        : 'Enable at least one governance module that generates workflow evidence.',
    },
    {
      id: 'remote-workflows',
      label: 'Remote workflow readiness',
      status: workflowReadinessStatus(workflowReadiness),
      summary: workflowReadiness
        ? `${workflowTotals.workflowsMissing} missing, ${workflowTotals.workflowsDifferent} different workflow file(s)`
        : 'No remote workflow readiness report attached',
      next_action: workflowReadiness?.status === 'ready'
        ? 'Keep workflow readiness evidence with the customer onboarding record.'
        : 'Run the read-only remote workflow readiness validator after install or PR merge.',
    },
    {
      id: 'actions-config',
      label: 'GitHub Actions configuration',
      status: workflowReadiness
        ? actionConfigMissing === 0 ? 'ready' : 'needs-action'
        : hasRequiredActionConfig ? 'needs-action' : 'ready',
      summary: workflowReadiness
        ? `${workflowTotals.variablesMissing} missing variable name(s), ${workflowTotals.secretsMissing} missing secret name(s)`
        : `${pack.variables.length} variable name(s), ${pack.secrets.length} secret name(s) required by the pack`,
      next_action: workflowReadiness && actionConfigMissing === 0
        ? 'Required GitHub Actions configuration names are present.'
        : 'Create required GitHub Actions variables/secrets outside GitGov and re-run readiness validation.',
    },
    {
      id: 'release-governance',
      label: 'Release governance policy',
      status: validation.valid ? 'ready' : 'blocked',
      summary: `${releaseGovernance.mode} for ${releaseGovernance.environment}, enforcement ${releaseGovernance.enforcement}`,
      next_action: releaseGovernance.enforcement === 'disabled'
        ? 'Record-only remains the safe default and does not block releases.'
        : 'Confirm the customer explicitly selected this policy before treating it as release blocking.',
    },
  ]
  const stageCounts = {
    ready: stages.filter((stage) => stage.status === 'ready').length,
    'needs-action': stages.filter((stage) => stage.status === 'needs-action').length,
    blocked: stages.filter((stage) => stage.status === 'blocked').length,
  }
  const score = Math.round(
    (stages.reduce((total, stage) => total + readinessStageWeight(stage.status), 0) / stages.length) * 100,
  )
  const status: EnterpriseOnboardingReadinessStatus = stageCounts.blocked > 0
    ? 'blocked'
    : stageCounts['needs-action'] > 0
      ? 'needs-action'
      : 'ready'

  return {
    generated_at: generatedAt,
    customer_name: pack.customer_name,
    repository_full_name: pack.repository_full_name,
    default_branch: pack.default_branch,
    jira_project_key: pack.jira_project_key,
    policy_preset: pack.policy_preset,
    status,
    readiness_score: score,
    stage_counts: stageCounts,
    release_governance: releaseGovernance,
    providers: pack.providers,
    modules: pack.modules,
    stages,
    next_actions: stages
      .filter((stage) => stage.status !== 'ready')
      .map((stage) => `${stage.label}: ${stage.next_action}`),
    safety: {
      contains_secret_values: false,
      reads_secret_values: false,
      mutates_customer_repository: false,
      mutates_provider_state: false,
      release_blocking_default: false,
    },
  }
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

export function buildEnterpriseOnboardingReadinessReportFilename(profile: EnterpriseAdoptionProfile): string {
  const basis = `${profile.customer_name}-${profile.repository_full_name}`
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
  return `${basis || 'enterprise-adoption'}-onboarding-readiness.json`
}

function onboardingRemediationPriority(
  stageId: EnterpriseOnboardingReadinessStageId,
  status: EnterpriseOnboardingReadinessStatus,
): number {
  if (status === 'blocked') return 0
  if (stageId === 'profile') return 1
  if (stageId === 'providers') return 2
  if (stageId === 'workflow-pack') return 3
  if (stageId === 'remote-workflows') return 4
  if (stageId === 'actions-config') return 5
  if (stageId === 'release-governance') return 6
  return 50
}

function onboardingRemediationOwner(stageId: EnterpriseOnboardingReadinessStageId): string {
  if (stageId === 'profile') return 'GitGov admin'
  if (stageId === 'providers') return 'Platform owner'
  if (stageId === 'workflow-pack') return 'DevOps owner'
  if (stageId === 'remote-workflows') return 'Repository admin'
  if (stageId === 'actions-config') return 'Repository admin'
  if (stageId === 'release-governance') return 'Release governance owner'
  return 'GitGov operator'
}

function onboardingRemediationValidation(stageId: EnterpriseOnboardingReadinessStageId): string {
  if (stageId === 'profile') return 'Regenerate onboarding readiness and confirm the profile stage is ready.'
  if (stageId === 'providers') return 'Attach a sanitized provider connection report with ready provider checks.'
  if (stageId === 'workflow-pack') return 'Regenerate the workflow template pack and review the manifest.'
  if (stageId === 'remote-workflows') return 'Run remote workflow readiness validation after install or remote PR merge.'
  if (stageId === 'actions-config') return 'Re-run workflow readiness and confirm required variable and secret names are present.'
  if (stageId === 'release-governance') return 'Run the release governance evaluator or confirm record-only policy remains intentional.'
  return 'Regenerate onboarding readiness and confirm the stage is ready.'
}

function buildOnboardingConfigurationCommand(
  kind: 'variable' | 'secret',
  name: string,
  repositoryFullName: string,
): EnterpriseOnboardingConfigurationCommand {
  return {
    kind,
    name,
    command: kind === 'variable'
      ? `gh variable set ${name} --repo ${repositoryFullName} --body "<value>"`
      : `gh secret set ${name} --repo ${repositoryFullName}`,
    contains_secret_value: false,
  }
}

export function buildEnterpriseOnboardingRemediationPlan(
  readiness: EnterpriseOnboardingReadinessReport,
  pack?: EnterpriseAdoptionPack,
  generatedAt = new Date().toISOString(),
): EnterpriseOnboardingRemediationPlan {
  const actions = readiness.stages
    .filter((stage) => stage.status !== 'ready')
    .map((stage): EnterpriseOnboardingRemediationAction => ({
      priority: onboardingRemediationPriority(stage.id, stage.status),
      stage_id: stage.id,
      stage: stage.label,
      status: stage.status,
      owner: onboardingRemediationOwner(stage.id),
      action: stage.next_action,
      reason: stage.summary,
      validation: onboardingRemediationValidation(stage.id),
    }))
    .sort((left, right) => left.priority - right.priority || left.stage_id.localeCompare(right.stage_id))

  const variables = pack?.variables ?? []
  const secrets = pack?.secrets ?? []
  const commands = [
    ...variables.map((variable) => buildOnboardingConfigurationCommand('variable', variable.name, readiness.repository_full_name)),
    ...secrets.map((secret) => buildOnboardingConfigurationCommand('secret', secret.name, readiness.repository_full_name)),
  ]

  const remediationStatus: EnterpriseOnboardingReadinessStatus = readiness.status === 'ready'
    ? 'ready'
    : actions.some((action) => action.status === 'blocked')
      ? 'blocked'
      : 'needs-action'

  return {
    generated_at: generatedAt,
    customer_name: readiness.customer_name,
    repository_full_name: readiness.repository_full_name,
    default_branch: readiness.default_branch,
    policy_preset: readiness.policy_preset,
    readiness_status: readiness.status,
    readiness_score: readiness.readiness_score,
    remediation_status: remediationStatus,
    action_count: actions.length,
    actions,
    github_actions_configuration: {
      source: 'dashboard-adoption-pack',
      variables_count: variables.length,
      secrets_count: secrets.length,
      commands_are_placeholders: true,
      commands,
    },
    validation: {
      regenerate_readiness: 'Run scripts/control-plane/generate_enterprise_onboarding_readiness_report.ps1 after completing actions.',
      rerun_provider_checks: 'Run scripts/control-plane/validate_enterprise_provider_connections.ps1 only with customer-approved credentials.',
      rerun_workflow_readiness: 'Run scripts/control-plane/validate_enterprise_workflow_installation_readiness.ps1 after workflow installation or remote PR merge.',
    },
    safety: {
      contains_secret_values: false,
      reads_secret_values: false,
      mutates_customer_repository: false,
      mutates_provider_state: false,
      creates_github_actions_variables: false,
      creates_github_actions_secrets: false,
      release_blocking_default: false,
    },
  }
}

export function buildEnterpriseOnboardingGuide(
  readiness: EnterpriseOnboardingReadinessReport,
  remediationPlan: EnterpriseOnboardingRemediationPlan,
  generatedAt = new Date().toISOString(),
): EnterpriseOnboardingGuide {
  const actionByStage = new Map(
    remediationPlan.actions.map((action) => [action.stage_id, action]),
  )
  const nextAction = remediationPlan.actions[0] ?? null
  const steps = readiness.stages.map((stage, index): EnterpriseOnboardingGuideStep => {
    const remediationAction = actionByStage.get(stage.id)
    const status: EnterpriseOnboardingGuideStepStatus = stage.status === 'ready'
      ? 'complete'
      : stage.status === 'blocked'
        ? 'blocked'
        : nextAction?.stage_id === stage.id
          ? 'next'
          : 'todo'

    return {
      order: index + 1,
      stage_id: stage.id,
      label: stage.label,
      status,
      readiness_status: stage.status,
      owner: remediationAction?.owner ?? 'GitGov operator',
      summary: stage.summary,
      action: stage.status === 'ready'
        ? 'Keep this evidence current during onboarding.'
        : remediationAction?.action ?? stage.next_action,
      validation: remediationAction?.validation ?? 'Regenerate onboarding readiness and confirm the stage remains ready.',
    }
  })

  return {
    generated_at: generatedAt,
    customer_name: readiness.customer_name,
    repository_full_name: readiness.repository_full_name,
    readiness_status: readiness.status,
    readiness_score: readiness.readiness_score,
    completed_steps: steps.filter((step) => step.status === 'complete').length,
    total_steps: steps.length,
    next_step: steps.find((step) => step.status === 'next') ?? steps.find((step) => step.status === 'blocked') ?? null,
    steps,
    configuration_summary: {
      variable_names: remediationPlan.github_actions_configuration.commands
        .filter((command) => command.kind === 'variable')
        .map((command) => command.name),
      secret_names: remediationPlan.github_actions_configuration.commands
        .filter((command) => command.kind === 'secret')
        .map((command) => command.name),
      commands_are_placeholders: remediationPlan.github_actions_configuration.commands_are_placeholders,
      suggested_commands_count: remediationPlan.github_actions_configuration.commands.length,
    },
    safety: {
      contains_secret_values: false,
      reads_secret_values: false,
      mutates_customer_repository: false,
      mutates_provider_state: false,
      creates_github_actions_variables: false,
      creates_github_actions_secrets: false,
      release_blocking_default: false,
    },
  }
}

function trimTrackingText(value: unknown, maxLength: number): string | undefined {
  if (typeof value !== 'string') return undefined
  const trimmed = value.trim()
  if (!trimmed) return undefined
  return trimmed.slice(0, maxLength)
}

export function normalizeEnterpriseOnboardingChecklistTracking(
  tracking?: Partial<EnterpriseOnboardingChecklistTracking> | null,
): EnterpriseOnboardingChecklistTracking {
  const items = Array.isArray(tracking?.items) ? tracking.items : []
  const seen = new Set<EnterpriseOnboardingReadinessStageId>()
  const normalizedItems: EnterpriseOnboardingChecklistTrackingItem[] = []
  for (const item of items) {
    const stageId = item?.stage_id
    if (!ONBOARDING_STAGE_IDS.includes(stageId) || seen.has(stageId)) continue
    seen.add(stageId)
    const status = ONBOARDING_TRACKING_STATUSES.includes(item.status) ? item.status : 'open'
    normalizedItems.push({
      stage_id: stageId,
      status,
      owner: trimTrackingText(item.owner, 80),
      note: trimTrackingText(item.note, 1000),
      external_ref: trimTrackingText(item.external_ref, 120),
      target_date: trimTrackingText(item.target_date, 10),
      updated_at: trimTrackingText(item.updated_at, 40),
    })
  }

  return {
    version: 1,
    items: normalizedItems,
  }
}

export function upsertEnterpriseOnboardingChecklistTrackingItem(
  tracking: EnterpriseOnboardingChecklistTracking,
  item: EnterpriseOnboardingChecklistTrackingItem,
): EnterpriseOnboardingChecklistTracking {
  const normalized = normalizeEnterpriseOnboardingChecklistTracking(tracking)
  if (!ONBOARDING_STAGE_IDS.includes(item.stage_id)) return normalized
  const nextItem: EnterpriseOnboardingChecklistTrackingItem = {
    stage_id: item.stage_id,
    status: ONBOARDING_TRACKING_STATUSES.includes(item.status) ? item.status : 'open',
    owner: trimTrackingText(item.owner, 80),
    note: trimTrackingText(item.note, 1000),
    external_ref: trimTrackingText(item.external_ref, 120),
    target_date: trimTrackingText(item.target_date, 10),
    updated_at: item.updated_at ?? new Date().toISOString(),
  }
  const withoutStage = normalized.items.filter((candidate) => candidate.stage_id !== item.stage_id)
  return normalizeEnterpriseOnboardingChecklistTracking({
    version: 1,
    items: [...withoutStage, nextItem].sort(
      (left, right) => ONBOARDING_STAGE_IDS.indexOf(left.stage_id) - ONBOARDING_STAGE_IDS.indexOf(right.stage_id),
    ),
  })
}

export function buildEnterpriseOnboardingRemediationPlanFilename(profile: EnterpriseAdoptionProfile): string {
  const basis = `${profile.customer_name}-${profile.repository_full_name}`
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
  return `${basis || 'enterprise-adoption'}-onboarding-remediation-plan.json`
}
