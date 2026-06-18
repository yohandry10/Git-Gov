import {
  ADOPTION_PROVIDER_OPTIONS,
  normalizeEnterpriseProviderSetupDecisions,
  providerSetupDecisionLabel,
  uniqueKnownValues,
  type AdoptionProvider,
  type EnterpriseAdoptionProfile,
  type EnterpriseProviderHealthCheck,
  type EnterpriseProviderSetupDecision,
  type EnterpriseProviderSetupAction,
  type EnterpriseProviderSetupGuidance,
  type EnterpriseProviderSetupStep,
  type EnterpriseProviderSetupTarget,
} from './adoption-profile'
import { buildEnterpriseProviderHealth } from './provider-health'

function setupActionForStatus(status: EnterpriseProviderSetupStep['status']): EnterpriseProviderSetupAction {
  if (status === 'needs-config') return 'connect'
  if (status === 'needs-evidence') return 'retry'
  if (status === 'skipped') return 'skip'
  return 'review'
}

function setupActionLabel(action: EnterpriseProviderSetupAction): string {
  if (action === 'connect') return 'Connect'
  if (action === 'retry') return 'Retry'
  if (action === 'skip') return 'Skipped'
  return 'Review'
}

function setupTargetForAction(action: EnterpriseProviderSetupAction): EnterpriseProviderSetupTarget {
  if (action === 'connect') {
    return {
      kind: 'settings',
      label: 'Open Settings',
      to: '/settings#control-plane',
      navigation_only: true,
    }
  }
  if (action === 'retry') {
    return {
      kind: 'evidence',
      label: 'Open Evidence',
      to: '/governance/evidence',
      navigation_only: true,
    }
  }
  if (action === 'skip') {
    return {
      kind: 'adoption-profile',
      label: 'Review profile',
      to: '/governance/adoption#enterprise-adoption',
      navigation_only: true,
    }
  }
  return {
    kind: 'action-center',
    label: 'Open Action Center',
    to: '/action-center',
    navigation_only: true,
  }
}

function healthByProvider(
  providerHealth: EnterpriseProviderHealthCheck[],
): Map<AdoptionProvider, EnterpriseProviderHealthCheck> {
  return new Map(providerHealth.map((check) => [check.provider, check]))
}

function selectedProviderSet(profile: EnterpriseAdoptionProfile): Set<AdoptionProvider> {
  return new Set(uniqueKnownValues(profile.providers, ADOPTION_PROVIDER_OPTIONS.map((option) => option.id)))
}

function decisionByProvider(profile: EnterpriseAdoptionProfile): Map<AdoptionProvider, EnterpriseProviderSetupDecision> {
  return new Map(normalizeEnterpriseProviderSetupDecisions(profile.provider_setup_decisions).decisions.map((decision) => [
    decision.provider,
    decision,
  ]))
}

function decisionAppliesToStep(
  decision: EnterpriseProviderSetupDecision | undefined,
  step: Pick<EnterpriseProviderSetupStep, 'selected' | 'status'>,
): EnterpriseProviderSetupDecision | null {
  if (!decision) return null
  if (!step.selected && step.status === 'skipped' && decision.decision === 'intentionally-skipped') return decision
  if (step.selected && step.status === 'ready' && decision.decision === 'reviewed') return decision
  if (
    step.selected &&
    (step.status === 'needs-config' || step.status === 'needs-evidence') &&
    decision.decision === 'retry-later'
  ) {
    return decision
  }
  return null
}

function compareSetupSteps(left: EnterpriseProviderSetupStep, right: EnterpriseProviderSetupStep): number {
  const priority: Record<EnterpriseProviderSetupStep['status'], number> = {
    'needs-config': 0,
    'needs-evidence': 1,
    ready: 2,
    skipped: 3,
  }
  return priority[left.status] - priority[right.status] || left.label.localeCompare(right.label)
}

export function buildEnterpriseProviderSetupGuidance(
  profile: EnterpriseAdoptionProfile,
  providerHealth: EnterpriseProviderHealthCheck[] = buildEnterpriseProviderHealth(profile),
): EnterpriseProviderSetupGuidance {
  const selected = selectedProviderSet(profile)
  const health = healthByProvider(providerHealth)
  const decisions = decisionByProvider(profile)
  const steps = ADOPTION_PROVIDER_OPTIONS.map((option): EnterpriseProviderSetupStep => {
    const selectedProvider = selected.has(option.id)
    const check = health.get(option.id)
    const status = selectedProvider ? check?.status ?? 'needs-evidence' : 'skipped'
    const action = setupActionForStatus(status)
    const operatorDecision = decisionAppliesToStep(decisions.get(option.id), {
      selected: selectedProvider,
      status,
    })
    return {
      provider: option.id,
      label: option.label,
      selected: selectedProvider,
      status,
      action,
      action_label: setupActionLabel(action),
      reason: selectedProvider
        ? check?.evidence ?? 'No provider evidence is available yet'
        : 'Not selected for this customer onboarding profile',
      validation: selectedProvider
        ? check?.next_step ?? 'Run the approved provider validation flow and wait for GitGov evidence.'
        : 'Leave unselected unless the customer uses this provider.',
      target: setupTargetForAction(action),
      operator_decision: operatorDecision,
      operator_decision_label: operatorDecision ? providerSetupDecisionLabel(operatorDecision.decision) : null,
    }
  })
  const orderedSteps = [...steps].sort(compareSetupSteps)
  const nextStep = orderedSteps.find((step) => step.status === 'needs-config' || step.status === 'needs-evidence') ?? null

  return {
    selected_count: steps.filter((step) => step.selected).length,
    skipped_count: steps.filter((step) => step.status === 'skipped').length,
    ready_count: steps.filter((step) => step.status === 'ready').length,
    needs_config_count: steps.filter((step) => step.status === 'needs-config').length,
    needs_evidence_count: steps.filter((step) => step.status === 'needs-evidence').length,
    operator_decision_count: steps.filter((step) => step.operator_decision !== null).length,
    next_step: nextStep,
    steps: orderedSteps,
    safety: {
      contains_secret_values: false,
      reads_secret_values: false,
      mutates_customer_repository: false,
      mutates_provider_state: false,
      calls_provider_api: false,
      starts_oauth_flow: false,
      release_blocking_default: false,
      agent_governance_used: false,
    },
  }
}
