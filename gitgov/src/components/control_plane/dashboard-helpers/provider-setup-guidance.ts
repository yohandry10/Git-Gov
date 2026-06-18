import {
  ADOPTION_PROVIDER_OPTIONS,
  uniqueKnownValues,
  type AdoptionProvider,
  type EnterpriseAdoptionProfile,
  type EnterpriseProviderHealthCheck,
  type EnterpriseProviderSetupAction,
  type EnterpriseProviderSetupGuidance,
  type EnterpriseProviderSetupStep,
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

function healthByProvider(
  providerHealth: EnterpriseProviderHealthCheck[],
): Map<AdoptionProvider, EnterpriseProviderHealthCheck> {
  return new Map(providerHealth.map((check) => [check.provider, check]))
}

function selectedProviderSet(profile: EnterpriseAdoptionProfile): Set<AdoptionProvider> {
  return new Set(uniqueKnownValues(profile.providers, ADOPTION_PROVIDER_OPTIONS.map((option) => option.id)))
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
  const steps = ADOPTION_PROVIDER_OPTIONS.map((option): EnterpriseProviderSetupStep => {
    const selectedProvider = selected.has(option.id)
    const check = health.get(option.id)
    const status = selectedProvider ? check?.status ?? 'needs-evidence' : 'skipped'
    const action = setupActionForStatus(status)
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
