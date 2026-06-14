import {
  DEFAULT_ENTERPRISE_ADOPTION_PROFILE,
  normalizeEnterpriseAdoptionProfile,
  type EnterpriseAdoptionProfile,
  type EnterpriseOnboardingChecklistTrackingStatus,
  type EnterpriseOnboardingGuideStepStatus,
  type EnterpriseOnboardingReadinessStatus,
  type EnterpriseProviderHealthStatus,
} from './dashboard-helpers'

export const ONBOARDING_TRACKING_STATUS_OPTIONS: Array<{ id: EnterpriseOnboardingChecklistTrackingStatus; label: string }> = [
  { id: 'open', label: 'Open' },
  { id: 'in-progress', label: 'Doing' },
  { id: 'waiting', label: 'Wait' },
  { id: 'done', label: 'Done' },
]

export function cloneDefaultProfile(): EnterpriseAdoptionProfile {
  return normalizeEnterpriseAdoptionProfile(DEFAULT_ENTERPRISE_ADOPTION_PROFILE)
}

export function toggleValue<T extends string>(values: T[], value: T): T[] {
  return values.includes(value)
    ? values.filter((candidate) => candidate !== value)
    : [...values, value]
}

export function selectedClass(selected: boolean): string {
  return selected
    ? 'border-brand-500/60 bg-brand-500/12 text-brand-200'
    : 'border-white/10 bg-white/[0.03] text-surface-300 hover:border-white/20 hover:bg-white/[0.05]'
}

export function providerHealthBadgeVariant(status: EnterpriseProviderHealthStatus): 'success' | 'warning' | 'info' {
  if (status === 'ready') return 'success'
  if (status === 'needs-config') return 'warning'
  return 'info'
}

export function providerHealthLabel(status: EnterpriseProviderHealthStatus): string {
  if (status === 'ready') return 'Ready'
  if (status === 'needs-config') return 'Config'
  return 'Evidence'
}

export function onboardingReadinessBadgeVariant(status: EnterpriseOnboardingReadinessStatus): 'success' | 'warning' | 'info' {
  if (status === 'ready') return 'success'
  if (status === 'blocked') return 'warning'
  return 'info'
}

export function onboardingReadinessLabel(status: EnterpriseOnboardingReadinessStatus): string {
  if (status === 'ready') return 'Ready'
  if (status === 'blocked') return 'Blocked'
  return 'Action'
}

export function onboardingGuideStepBadgeVariant(status: EnterpriseOnboardingGuideStepStatus): 'success' | 'warning' | 'info' | 'neutral' {
  if (status === 'complete') return 'success'
  if (status === 'blocked') return 'warning'
  if (status === 'next') return 'info'
  return 'neutral'
}

export function onboardingGuideStepLabel(status: EnterpriseOnboardingGuideStepStatus): string {
  if (status === 'complete') return 'Done'
  if (status === 'blocked') return 'Blocked'
  if (status === 'next') return 'Next'
  return 'Todo'
}

export function onboardingGuideStepClass(status: EnterpriseOnboardingGuideStepStatus): string {
  if (status === 'complete') return 'border-success-500/20 bg-success-500/8'
  if (status === 'blocked') return 'border-warning-500/25 bg-warning-500/8'
  if (status === 'next') return 'border-brand-500/25 bg-brand-500/8'
  return 'border-white/8 bg-white/[0.03]'
}
