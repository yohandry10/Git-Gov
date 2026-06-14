import { useEffect, useMemo, useState } from 'react'
import { Activity, AlertTriangle, CheckCircle2, Circle, CircleAlert, CircleDot, ClipboardCheck, Download, KeyRound, ListChecks, PackageCheck, Save, ShieldCheck, Workflow } from 'lucide-react'
import { Badge } from '@/components/shared/Badge'
import { Button } from '@/components/shared/Button'
import { useControlPlaneStore } from '@/store/useControlPlaneStore'
import { ReleaseGovernanceEnvironmentPolicyPanel } from './ReleaseGovernanceEnvironmentPolicyPanel'
import {
  ONBOARDING_TRACKING_STATUS_OPTIONS,
  cloneDefaultProfile,
  onboardingGuideStepBadgeVariant,
  onboardingGuideStepClass,
  onboardingGuideStepLabel,
  onboardingReadinessBadgeVariant,
  onboardingReadinessLabel,
  providerHealthBadgeVariant,
  providerHealthLabel,
  selectedClass,
  toggleValue,
} from './enterprise-adoption-panel-helpers'
import {
  ADOPTION_MODULE_OPTIONS,
  ADOPTION_POLICY_PRESET_OPTIONS,
  ADOPTION_PROVIDER_OPTIONS,
  addReleaseGovernanceEnvironmentOverride,
  buildEnterpriseAdoptionPack,
  buildEnterpriseAdoptionPackFilename,
  buildEnterpriseOnboardingGuide,
  buildEnterpriseOnboardingReadinessReport,
  buildEnterpriseOnboardingReadinessReportFilename,
  buildEnterpriseOnboardingRemediationPlan,
  buildEnterpriseOnboardingRemediationPlanFilename,
  buildEnterpriseWorkflowTemplatePack,
  buildEnterpriseWorkflowTemplatePackFilename,
  buildEnterpriseProviderHealth,
  normalizeEnterpriseOnboardingChecklistTracking,
  normalizeEnterpriseAdoptionProfile,
  removeReleaseGovernanceEnvironmentOverride,
  releaseGovernanceModeNeedsFormalApproval,
  updateReleaseGovernanceBaseEnvironment,
  updateReleaseGovernanceBaseMode,
  updateReleaseGovernanceEnvironmentOverrideEnvironment,
  updateReleaseGovernanceEnvironmentOverrideMode,
  upsertEnterpriseOnboardingChecklistTrackingItem,
  validateEnterpriseAdoptionProfile,
  type AdoptionModule,
  type AdoptionPolicyPreset,
  type AdoptionProvider,
  type AdoptionReleaseGovernanceMode,
  type EnterpriseAdoptionProfile,
  type EnterpriseOnboardingChecklistTracking,
  type EnterpriseOnboardingChecklistTrackingItem,
} from './dashboard-helpers'

export function EnterpriseAdoptionPanel() {
  const selectedOrgName = useControlPlaneStore((state) => state.selectedOrgName)
  const serverStats = useControlPlaneStore((state) => state.serverStats)
  const ticketCoverage = useControlPlaneStore((state) => state.ticketCoverage)
  const jenkinsCorrelations = useControlPlaneStore((state) => state.jenkinsCorrelations)
  const persistedProfile = useControlPlaneStore((state) => state.enterpriseAdoptionProfile)
  const persistedProfileUpdatedAt = useControlPlaneStore((state) => state.enterpriseAdoptionProfileUpdatedAt)
  const isProfileLoading = useControlPlaneStore((state) => state.isEnterpriseAdoptionProfileLoading)
  const isProfileSaving = useControlPlaneStore((state) => state.isEnterpriseAdoptionProfileSaving)
  const profileError = useControlPlaneStore((state) => state.enterpriseAdoptionProfileError)
  const persistedChecklistTracking = useControlPlaneStore((state) => state.enterpriseOnboardingChecklistTracking)
  const persistedChecklistTrackingUpdatedAt = useControlPlaneStore((state) => state.enterpriseOnboardingChecklistTrackingUpdatedAt)
  const isChecklistTrackingLoading = useControlPlaneStore((state) => state.isEnterpriseOnboardingChecklistTrackingLoading)
  const isChecklistTrackingSaving = useControlPlaneStore((state) => state.isEnterpriseOnboardingChecklistTrackingSaving)
  const checklistTrackingError = useControlPlaneStore((state) => state.enterpriseOnboardingChecklistTrackingError)
  const loadEnterpriseAdoptionProfile = useControlPlaneStore((state) => state.loadEnterpriseAdoptionProfile)
  const saveEnterpriseAdoptionProfile = useControlPlaneStore((state) => state.saveEnterpriseAdoptionProfile)
  const loadEnterpriseOnboardingChecklistTracking = useControlPlaneStore((state) => state.loadEnterpriseOnboardingChecklistTracking)
  const saveEnterpriseOnboardingChecklistTracking = useControlPlaneStore((state) => state.saveEnterpriseOnboardingChecklistTracking)
  const [profile, setProfile] = useState<EnterpriseAdoptionProfile>(() => cloneDefaultProfile())
  const [checklistTracking, setChecklistTracking] = useState<EnterpriseOnboardingChecklistTracking>(() => normalizeEnterpriseOnboardingChecklistTracking())
  const pack = useMemo(() => buildEnterpriseAdoptionPack(profile), [profile])
  const validation = useMemo(() => validateEnterpriseAdoptionProfile(profile), [profile])
  const sonarRuns = useMemo(
    () => jenkinsCorrelations.filter((entry) => entry.pipeline?.job_name.toLowerCase().includes('sonar')).length,
    [jenkinsCorrelations],
  )
  const sonarSuccessful = useMemo(
    () => jenkinsCorrelations.filter(
      (entry) => entry.pipeline?.job_name.toLowerCase().includes('sonar') && entry.pipeline.status === 'success',
    ).length,
    [jenkinsCorrelations],
  )
  const providerHealth = useMemo(() => buildEnterpriseProviderHealth(profile, {
    githubEventsTotal: serverStats?.github_events.total,
    githubEventTypes: serverStats?.github_events.by_type,
    jiraCommitsWithTicket: ticketCoverage?.commits_with_ticket,
    jiraCoveragePercentage: ticketCoverage?.coverage_percentage,
    pipelineRuns7d: serverStats?.pipeline?.total_7d,
    pipelineSuccess7d: serverStats?.pipeline?.success_7d,
    sonarRuns,
    sonarSuccessful,
    activeRepos: serverStats?.active_repos,
  }, pack), [pack, profile, serverStats, sonarRuns, sonarSuccessful, ticketCoverage])
  const onboardingReadiness = useMemo(
    () => buildEnterpriseOnboardingReadinessReport(profile, providerHealth),
    [profile, providerHealth],
  )
  const onboardingRemediationPlan = useMemo(
    () => buildEnterpriseOnboardingRemediationPlan(onboardingReadiness, pack),
    [onboardingReadiness, pack],
  )
  const onboardingGuide = useMemo(
    () => buildEnterpriseOnboardingGuide(onboardingReadiness, onboardingRemediationPlan),
    [onboardingReadiness, onboardingRemediationPlan],
  )
  const readyProviders = providerHealth.filter((check) => check.status === 'ready').length
  const readinessTarget = pack.policy_rules.find((rule) => rule.rule === 'Release readiness target')?.setting ?? '0'
  const trendRule = pack.policy_rules.find((rule) => rule.rule === 'Vulnerability trend enforcement')?.setting ?? 'informational'
  const releaseGovernance = pack.release_governance
  const releaseGovernanceBadgeVariant = releaseGovernance.enforcement === 'blocking'
    ? 'warning'
    : releaseGovernance.enforcement === 'advisory'
      ? 'info'
      : 'neutral'
  const savedAtLabel = persistedProfileUpdatedAt
    ? new Date(persistedProfileUpdatedAt).toLocaleString()
    : null
  const checklistSavedAtLabel = persistedChecklistTrackingUpdatedAt
    ? new Date(persistedChecklistTrackingUpdatedAt).toLocaleString()
    : null

  useEffect(() => {
    void loadEnterpriseAdoptionProfile(selectedOrgName || undefined)
    void loadEnterpriseOnboardingChecklistTracking(selectedOrgName || undefined)
  }, [loadEnterpriseAdoptionProfile, loadEnterpriseOnboardingChecklistTracking, selectedOrgName])

  useEffect(() => {
    if (!persistedProfile) return
    setProfile(normalizeEnterpriseAdoptionProfile(persistedProfile))
  }, [persistedProfile])

  useEffect(() => {
    setChecklistTracking(normalizeEnterpriseOnboardingChecklistTracking(persistedChecklistTracking))
  }, [persistedChecklistTracking])

  const updateText = (
    field: 'customer_name' | 'repository_full_name' | 'default_branch' | 'jira_project_key',
    value: string,
  ) => {
    setProfile((current) => ({ ...current, [field]: value }))
  }

  const updatePolicyPreset = (policyPreset: AdoptionPolicyPreset) => {
    setProfile((current) => ({ ...current, policy_preset: policyPreset }))
  }

  const updateReleaseGovernanceMode = (mode: AdoptionReleaseGovernanceMode) => {
    setProfile((current) => ({
      ...current,
      release_governance: updateReleaseGovernanceBaseMode(current.release_governance, mode),
      modules: !releaseGovernanceModeNeedsFormalApproval(mode) || current.modules.includes('formal-approval')
        ? current.modules
        : [...current.modules, 'formal-approval'],
    }))
  }

  const updateReleaseGovernanceEnvironment = (environment: string) => {
    setProfile((current) => ({
      ...current,
      release_governance: updateReleaseGovernanceBaseEnvironment(current.release_governance, environment),
    }))
  }

  const addReleaseGovernanceOverride = () => {
    setProfile((current) => ({
      ...current,
      modules: current.modules.includes('formal-approval') ? current.modules : [...current.modules, 'formal-approval'],
      release_governance: addReleaseGovernanceEnvironmentOverride(current.release_governance),
    }))
  }

  const updateReleaseGovernanceOverrideEnvironment = (index: number, environment: string) => {
    setProfile((current) => ({
      ...current,
      release_governance: updateReleaseGovernanceEnvironmentOverrideEnvironment(
        current.release_governance,
        index,
        environment,
      ),
    }))
  }

  const updateReleaseGovernanceOverrideMode = (index: number, mode: AdoptionReleaseGovernanceMode) => {
    setProfile((current) => ({
      ...current,
      modules: !releaseGovernanceModeNeedsFormalApproval(mode) || current.modules.includes('formal-approval')
        ? current.modules
        : [...current.modules, 'formal-approval'],
      release_governance: updateReleaseGovernanceEnvironmentOverrideMode(current.release_governance, index, mode),
    }))
  }

  const removeReleaseGovernanceOverride = (index: number) => {
    setProfile((current) => ({
      ...current,
      release_governance: removeReleaseGovernanceEnvironmentOverride(current.release_governance, index),
    }))
  }

  const toggleProvider = (provider: AdoptionProvider) => {
    setProfile((current) => ({ ...current, providers: toggleValue(current.providers, provider) }))
  }

  const toggleModule = (module: AdoptionModule) => {
    setProfile((current) => ({ ...current, modules: toggleValue(current.modules, module) }))
  }

  const downloadPack = () => {
    if (!validation.valid) return
    const blob = new Blob([JSON.stringify(pack, null, 2)], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    try {
      const link = document.createElement('a')
      link.href = url
      link.download = buildEnterpriseAdoptionPackFilename(profile)
      link.click()
    } finally {
      URL.revokeObjectURL(url)
    }
  }

  const downloadWorkflowTemplates = () => {
    if (!validation.valid) return
    const workflowTemplatePack = buildEnterpriseWorkflowTemplatePack(profile)
    const blob = new Blob([JSON.stringify(workflowTemplatePack, null, 2)], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    try {
      const link = document.createElement('a')
      link.href = url
      link.download = buildEnterpriseWorkflowTemplatePackFilename(profile)
      link.click()
    } finally {
      URL.revokeObjectURL(url)
    }
  }

  const downloadOnboardingReadiness = () => {
    const blob = new Blob([JSON.stringify(onboardingReadiness, null, 2)], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    try {
      const link = document.createElement('a')
      link.href = url
      link.download = buildEnterpriseOnboardingReadinessReportFilename(profile)
      link.click()
    } finally {
      URL.revokeObjectURL(url)
    }
  }

  const downloadOnboardingRemediationPlan = () => {
    const blob = new Blob([JSON.stringify(onboardingRemediationPlan, null, 2)], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    try {
      const link = document.createElement('a')
      link.href = url
      link.download = buildEnterpriseOnboardingRemediationPlanFilename(profile)
      link.click()
    } finally {
      URL.revokeObjectURL(url)
    }
  }

  const trackingItemForStage = (stageId: EnterpriseOnboardingChecklistTrackingItem['stage_id']) => (
    checklistTracking.items.find((item) => item.stage_id === stageId)
  )

  const updateChecklistTrackingItem = (
    stageId: EnterpriseOnboardingChecklistTrackingItem['stage_id'],
    patch: Partial<EnterpriseOnboardingChecklistTrackingItem>,
  ) => {
    const currentItem = trackingItemForStage(stageId)
    const guideStep = onboardingGuide.steps.find((step) => step.stage_id === stageId)
    const nextItem: EnterpriseOnboardingChecklistTrackingItem = {
      stage_id: stageId,
      status: patch.status ?? currentItem?.status ?? 'open',
      owner: patch.owner ?? currentItem?.owner ?? guideStep?.owner,
      note: patch.note ?? currentItem?.note,
      external_ref: patch.external_ref ?? currentItem?.external_ref,
      target_date: patch.target_date ?? currentItem?.target_date,
      updated_at: new Date().toISOString(),
    }
    setChecklistTracking((current) => upsertEnterpriseOnboardingChecklistTrackingItem(current, nextItem))
  }

  const saveProfile = async () => {
    if (!validation.valid) return
    await saveEnterpriseAdoptionProfile(profile, selectedOrgName || undefined)
  }

  const saveChecklistTracking = async () => {
    await saveEnterpriseOnboardingChecklistTracking(checklistTracking, selectedOrgName || undefined)
  }

  return (
    <section id="enterprise-adoption" className="glass-panel p-5 scroll-mt-4">
      <div className="card-header mb-4">
        <div>
          <div className="flex items-center gap-2">
            <PackageCheck size={16} className="text-brand-400" />
            <h2>Enterprise Adoption</h2>
            <Badge variant={validation.valid ? 'success' : 'warning'}>
              {validation.valid ? 'Ready' : 'Needs input'}
            </Badge>
          </div>
          <p>
            Customer profile, governance modules, workflow plan, and secret-safe adoption pack.
            {savedAtLabel ? <span className="ml-2 text-surface-500">Saved {savedAtLabel}</span> : null}
          </p>
        </div>
        <div className="flex flex-wrap justify-end gap-2">
          <Button
            size="sm"
            variant="primary"
            onClick={saveProfile}
            disabled={!validation.valid || isProfileSaving}
            title="Save adoption profile"
          >
            <Save size={14} />
            {isProfileSaving ? 'Saving' : 'Save'}
          </Button>
          <Button
            size="sm"
            variant="outline"
            onClick={downloadPack}
            disabled={!validation.valid}
            title="Download adoption pack JSON"
          >
            <Download size={14} />
            JSON
          </Button>
          <Button
            size="sm"
            variant="outline"
            onClick={downloadWorkflowTemplates}
            disabled={!validation.valid}
            title="Download workflow template pack JSON"
          >
            <Workflow size={14} />
            Workflows
          </Button>
          <Button
            size="sm"
            variant="outline"
            onClick={downloadOnboardingReadiness}
            title="Download onboarding readiness JSON"
          >
            <ClipboardCheck size={14} />
            Readiness
          </Button>
          <Button
            size="sm"
            variant="outline"
            onClick={downloadOnboardingRemediationPlan}
            title="Download onboarding remediation plan JSON"
          >
            <ListChecks size={14} />
            Plan
          </Button>
        </div>
      </div>

      {(isProfileLoading || profileError) && (
        <div className={`mb-4 rounded border p-3 text-xs ${profileError ? 'border-warning-500/20 bg-warning-500/8 text-warning-100' : 'border-white/8 bg-white/[0.03] text-surface-300'}`}>
          {profileError ?? 'Loading saved profile...'}
        </div>
      )}

      {(isChecklistTrackingLoading || checklistTrackingError) && (
        <div className={`mb-4 rounded border p-3 text-xs ${checklistTrackingError ? 'border-warning-500/20 bg-warning-500/8 text-warning-100' : 'border-white/8 bg-white/[0.03] text-surface-300'}`}>
          {checklistTrackingError ?? 'Loading checklist tracking...'}
        </div>
      )}

      <div className="space-y-4">
        <div className="grid grid-cols-1 2xl:grid-cols-[minmax(0,1fr)_minmax(340px,0.42fr)] gap-4 items-start">
          <div className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
            <label className="space-y-1">
              <span className="text-[10px] text-surface-500 uppercase tracking-widest">Customer</span>
              <input
                value={profile.customer_name}
                onChange={(event) => updateText('customer_name', event.target.value)}
                className="w-full bg-surface-800 border border-surface-600 rounded px-2 py-1.5 text-xs text-surface-200 focus:outline-none focus:border-surface-400"
              />
            </label>
            <label className="space-y-1">
              <span className="text-[10px] text-surface-500 uppercase tracking-widest">Repository</span>
              <input
                value={profile.repository_full_name}
                onChange={(event) => updateText('repository_full_name', event.target.value)}
                className="w-full bg-surface-800 border border-surface-600 rounded px-2 py-1.5 text-xs text-surface-200 focus:outline-none focus:border-surface-400"
              />
            </label>
            <label className="space-y-1">
              <span className="text-[10px] text-surface-500 uppercase tracking-widest">Default branch</span>
              <input
                value={profile.default_branch}
                onChange={(event) => updateText('default_branch', event.target.value)}
                className="w-full bg-surface-800 border border-surface-600 rounded px-2 py-1.5 text-xs text-surface-200 focus:outline-none focus:border-surface-400"
              />
            </label>
            <label className="space-y-1">
              <span className="text-[10px] text-surface-500 uppercase tracking-widest">Jira key</span>
              <input
                value={profile.jira_project_key}
                onChange={(event) => updateText('jira_project_key', event.target.value.toUpperCase())}
                className="w-full bg-surface-800 border border-surface-600 rounded px-2 py-1.5 text-xs text-surface-200 focus:outline-none focus:border-surface-400"
              />
            </label>
          </div>

          <div className="space-y-2">
            <span className="text-[10px] text-surface-500 uppercase tracking-widest">Policy preset</span>
            <div className="grid grid-cols-3 gap-2">
              {ADOPTION_POLICY_PRESET_OPTIONS.map((option) => {
                const selected = profile.policy_preset === option.id
                return (
                  <button
                    key={option.id}
                    type="button"
                    aria-pressed={selected}
                    onClick={() => updatePolicyPreset(option.id)}
                    className={`rounded border px-3 py-2 text-xs font-medium transition-colors ${selectedClass(selected)}`}
                  >
                    {option.label}
                  </button>
                )
              })}
            </div>
          </div>

          <ReleaseGovernanceEnvironmentPolicyPanel
            policy={releaseGovernance}
            badgeVariant={releaseGovernanceBadgeVariant}
            onBaseModeChange={updateReleaseGovernanceMode}
            onBaseEnvironmentChange={updateReleaseGovernanceEnvironment}
            onAddOverride={addReleaseGovernanceOverride}
            onOverrideEnvironmentChange={updateReleaseGovernanceOverrideEnvironment}
            onOverrideModeChange={updateReleaseGovernanceOverrideMode}
            onRemoveOverride={removeReleaseGovernanceOverride}
            selectedClass={selectedClass}
          />

          <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
            <div className="space-y-2">
              <span className="text-[10px] text-surface-500 uppercase tracking-widest">Providers</span>
              <div className="grid grid-cols-2 gap-2">
                {ADOPTION_PROVIDER_OPTIONS.map((option) => {
                  const selected = profile.providers.includes(option.id)
                  return (
                    <button
                      key={option.id}
                      type="button"
                      aria-pressed={selected}
                      onClick={() => toggleProvider(option.id)}
                      className={`flex items-center justify-between rounded border px-3 py-2 text-xs transition-colors ${selectedClass(selected)}`}
                    >
                      <span>{option.label}</span>
                      {selected && <ShieldCheck size={14} className="text-brand-300" />}
                    </button>
                  )
                })}
              </div>
            </div>

            <div className="space-y-2">
              <span className="text-[10px] text-surface-500 uppercase tracking-widest">Modules</span>
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
                {ADOPTION_MODULE_OPTIONS.map((option) => {
                  const selected = profile.modules.includes(option.id)
                  return (
                    <button
                      key={option.id}
                      type="button"
                      aria-pressed={selected}
                      onClick={() => toggleModule(option.id)}
                      className={`flex items-center justify-between rounded border px-3 py-2 text-xs transition-colors ${selectedClass(selected)}`}
                    >
                      <span>{option.label}</span>
                      {selected && <ShieldCheck size={14} className="text-brand-300" />}
                    </button>
                  )
                })}
              </div>
            </div>
          </div>

          {!validation.valid && (
            <div className="rounded border border-warning-500/20 bg-warning-500/8 p-3">
              <div className="flex items-center gap-2 text-xs font-medium text-warning-300">
                <AlertTriangle size={14} />
                Validation
              </div>
              <ul className="mt-2 space-y-1 text-[11px] text-warning-100/90">
                {validation.errors.map((error) => (
                  <li key={error}>{error}</li>
                ))}
              </ul>
            </div>
          )}
        </div>

          <div className="space-y-4">
            <div className="rounded border border-white/8 bg-white/[0.03] p-3">
            <div className="flex items-center justify-between gap-2">
              <div className="flex items-center gap-2 text-[10px] uppercase tracking-widest text-surface-500">
                <ClipboardCheck size={13} />
                Onboarding
              </div>
              <Badge variant={onboardingReadinessBadgeVariant(onboardingReadiness.status)}>
                {onboardingReadinessLabel(onboardingReadiness.status)}
              </Badge>
            </div>
            <div className="mt-2 flex items-end justify-between gap-3">
              <div className="mono-data text-2xl text-surface-100">{onboardingReadiness.readiness_score}</div>
              <div className="text-right text-[11px] text-surface-500">
                {onboardingReadiness.stage_counts.ready}/{onboardingReadiness.stages.length} stages ready
              </div>
            </div>
            <div className="mt-2 text-[11px] leading-5 text-surface-400">
              {onboardingReadiness.next_actions[0] ?? 'Onboarding evidence is ready for customer review.'}
            </div>
            <div className="mt-2 text-[10px] uppercase tracking-widest text-surface-500">
              {onboardingRemediationPlan.action_count} remediation action{onboardingRemediationPlan.action_count === 1 ? '' : 's'}
            </div>
            </div>

            <div className="grid grid-cols-2 gap-2">
              <div className="rounded border border-white/8 bg-white/[0.03] p-3">
                <div className="flex items-center gap-2 text-[10px] uppercase tracking-widest text-surface-500">
                  <Workflow size={13} />
                  Workflows
                </div>
                <div className="mt-2 mono-data text-xl text-surface-100">{pack.workflow_plan.length}</div>
              </div>
              <div className="rounded border border-white/8 bg-white/[0.03] p-3">
                <div className="flex items-center gap-2 text-[10px] uppercase tracking-widest text-surface-500">
                  <KeyRound size={13} />
                  Secrets
                </div>
                <div className="mt-2 mono-data text-xl text-surface-100">{pack.secrets.length}</div>
              </div>
              <div className="rounded border border-white/8 bg-white/[0.03] p-3">
                <div className="text-[10px] uppercase tracking-widest text-surface-500">Readiness</div>
                <div className="mt-2 mono-data text-xl text-surface-100">{readinessTarget}</div>
              </div>
              <div className="rounded border border-white/8 bg-white/[0.03] p-3">
                <div className="text-[10px] uppercase tracking-widest text-surface-500">Trend gate</div>
                <div className="mt-2 text-xs font-medium text-surface-100">{trendRule}</div>
              </div>
            </div>
          </div>
        </div>

        <div className="space-y-3 rounded border border-white/8 bg-white/[0.03] p-3">
            <div className="flex items-center justify-between gap-2">
              <div>
                <h3 className="flex items-center gap-2 text-xs font-semibold text-surface-200">
                  <ListChecks size={14} className="text-brand-300" />
                  Guided checklist
                </h3>
                {checklistSavedAtLabel && (
                  <div className="mt-1 text-[10px] text-surface-500">Saved {checklistSavedAtLabel}</div>
                )}
              </div>
              <div className="flex items-center gap-2">
                <Badge variant={onboardingGuide.completed_steps === onboardingGuide.total_steps ? 'success' : 'info'}>
                  {onboardingGuide.completed_steps}/{onboardingGuide.total_steps}
                </Badge>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={saveChecklistTracking}
                  disabled={isChecklistTrackingSaving}
                  title="Save checklist tracking"
                >
                  <Save size={13} />
                  {isChecklistTrackingSaving ? 'Saving' : 'Save'}
                </Button>
              </div>
            </div>
            {onboardingGuide.next_step && (
              <div className="rounded border border-brand-500/20 bg-brand-500/8 p-2">
                <div className="flex items-center justify-between gap-2">
                  <span className="text-[10px] uppercase tracking-widest text-brand-200">Next onboarding task</span>
                  <span className="text-[10px] text-surface-500">{onboardingGuide.next_step.owner}</span>
                </div>
                <div className="mt-1 text-xs font-medium text-surface-100">{onboardingGuide.next_step.label}</div>
                <div className="mt-1 text-[11px] leading-5 text-surface-300">{onboardingGuide.next_step.action}</div>
              </div>
            )}
            <div className="grid grid-cols-1 2xl:grid-cols-2 gap-2">
              {onboardingGuide.steps.map((step) => {
                const trackingItem = trackingItemForStage(step.stage_id)
                const StepIcon = step.status === 'complete'
                  ? CheckCircle2
                  : step.status === 'blocked'
                    ? CircleAlert
                    : step.status === 'next'
                      ? CircleDot
                      : Circle
                return (
                  <div key={step.stage_id} className={`rounded border p-2 ${onboardingGuideStepClass(step.status)}`}>
                    <div className="flex items-start gap-2">
                      <StepIcon size={14} className={step.status === 'complete' ? 'mt-0.5 text-success-300' : step.status === 'blocked' ? 'mt-0.5 text-warning-300' : step.status === 'next' ? 'mt-0.5 text-brand-300' : 'mt-0.5 text-surface-500'} />
                      <div className="min-w-0 flex-1">
                        <div className="flex items-start justify-between gap-2">
                          <div className="text-xs font-medium text-surface-100">
                            {step.order}. {step.label}
                          </div>
                          <Badge variant={onboardingGuideStepBadgeVariant(step.status)}>
                            {onboardingGuideStepLabel(step.status)}
                          </Badge>
                        </div>
                        <div className="mt-1 text-[11px] leading-5 text-surface-400">{step.summary}</div>
                        <div className="mt-1 text-[11px] leading-5 text-surface-300">{step.action}</div>
                        <div className="mt-1 text-[10px] leading-4 text-surface-500">
                          {step.owner} - {step.validation}
                        </div>
                        <div className="mt-2 space-y-2 rounded border border-white/8 bg-surface-950/30 p-2">
                          <div className="grid grid-cols-4 gap-1">
                            {ONBOARDING_TRACKING_STATUS_OPTIONS.map((option) => {
                              const selected = (trackingItem?.status ?? 'open') === option.id
                              return (
                                <button
                                  key={option.id}
                                  type="button"
                                  aria-pressed={selected}
                                  onClick={() => updateChecklistTrackingItem(step.stage_id, { status: option.id })}
                                  className={`rounded border px-2 py-1 text-[10px] font-medium transition-colors ${selectedClass(selected)}`}
                                >
                                  {option.label}
                                </button>
                              )
                            })}
                          </div>
                          <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
                            <input
                              value={trackingItem?.owner ?? ''}
                              onChange={(event) => updateChecklistTrackingItem(step.stage_id, { owner: event.target.value })}
                              className="w-full bg-surface-800 border border-surface-600 rounded px-2 py-1.5 text-[11px] text-surface-200 focus:outline-none focus:border-surface-400"
                              placeholder={step.owner}
                            />
                            <input
                              value={trackingItem?.target_date ?? ''}
                              onChange={(event) => updateChecklistTrackingItem(step.stage_id, { target_date: event.target.value })}
                              className="w-full bg-surface-800 border border-surface-600 rounded px-2 py-1.5 text-[11px] text-surface-200 focus:outline-none focus:border-surface-400"
                              placeholder="YYYY-MM-DD"
                            />
                          </div>
                          <input
                            value={trackingItem?.external_ref ?? ''}
                            onChange={(event) => updateChecklistTrackingItem(step.stage_id, { external_ref: event.target.value })}
                            className="w-full bg-surface-800 border border-surface-600 rounded px-2 py-1.5 text-[11px] text-surface-200 focus:outline-none focus:border-surface-400"
                            placeholder="Ticket or reference"
                          />
                          <textarea
                            value={trackingItem?.note ?? ''}
                            onChange={(event) => updateChecklistTrackingItem(step.stage_id, { note: event.target.value })}
                            className="min-h-16 w-full resize-y bg-surface-800 border border-surface-600 rounded px-2 py-1.5 text-[11px] text-surface-200 focus:outline-none focus:border-surface-400"
                            placeholder="Notes"
                          />
                        </div>
                      </div>
                    </div>
                  </div>
                )
              })}
            </div>
            <div className="grid grid-cols-3 gap-2 text-center">
              <div className="rounded border border-white/8 bg-surface-900/40 p-2">
                <div className="text-[10px] uppercase tracking-widest text-surface-500">Vars</div>
                <div className="mono-data mt-1 text-sm text-surface-100">{onboardingGuide.configuration_summary.variable_names.length}</div>
              </div>
              <div className="rounded border border-white/8 bg-surface-900/40 p-2">
                <div className="text-[10px] uppercase tracking-widest text-surface-500">Secrets</div>
                <div className="mono-data mt-1 text-sm text-surface-100">{onboardingGuide.configuration_summary.secret_names.length}</div>
              </div>
              <div className="rounded border border-white/8 bg-surface-900/40 p-2">
                <div className="text-[10px] uppercase tracking-widest text-surface-500">Cmds</div>
                <div className="mono-data mt-1 text-sm text-surface-100">{onboardingGuide.configuration_summary.suggested_commands_count}</div>
              </div>
            </div>
          </div>

          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <h3 className="flex items-center gap-2 text-xs font-semibold text-surface-200">
                <Activity size={14} className="text-brand-300" />
                Provider health
              </h3>
              <Badge variant={providerHealth.length > 0 && readyProviders === providerHealth.length ? 'success' : 'info'}>
                {readyProviders}/{providerHealth.length}
              </Badge>
            </div>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
              {providerHealth.map((check) => (
                <div key={check.provider} className="rounded border border-white/8 bg-white/[0.03] p-3">
                  <div className="flex items-center justify-between gap-2">
                    <span className="text-xs font-medium text-surface-200">{check.label}</span>
                    <Badge variant={providerHealthBadgeVariant(check.status)}>
                      {providerHealthLabel(check.status)}
                    </Badge>
                  </div>
                  <p className="mt-2 text-[11px] leading-5 text-surface-400">{check.evidence}</p>
                  <p className="mt-1 text-[10px] leading-4 text-surface-500">{check.next_step}</p>
                </div>
              ))}
            </div>
          </div>

          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <h3 className="text-xs font-semibold text-surface-200">Workflow plan</h3>
              <Badge variant="neutral">{pack.workflow_plan.length}</Badge>
            </div>
            <div className="max-h-48 overflow-auto rounded border border-white/8">
              <table className="w-full">
                <tbody className="divide-y divide-white/5">
                  {pack.workflow_plan.map((workflow) => (
                    <tr key={workflow.file}>
                      <td className="py-2 px-2 text-[11px] text-surface-200 mono-data">{workflow.file}</td>
                      <td className="py-2 px-2 text-[10px] text-surface-500">{workflow.reason}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>

          <div className="space-y-2">
            <h3 className="text-xs font-semibold text-surface-200">Policy rules</h3>
            <div className="rounded border border-white/8 divide-y divide-white/5">
              {pack.policy_rules.map((rule) => (
                <div key={rule.rule} className="flex items-center justify-between gap-3 px-2 py-2">
                  <span className="text-[11px] text-surface-400">{rule.rule}</span>
                  <span className="text-[11px] text-surface-100 text-right">{rule.setting}</span>
                </div>
              ))}
            </div>
          </div>

          <div className="space-y-2">
            <h3 className="text-xs font-semibold text-surface-200">Required configuration</h3>
            <div className="flex flex-wrap gap-2">
              {pack.variables.map((variable) => (
                <Badge key={variable.name} variant="info">{variable.name}</Badge>
              ))}
              {pack.secrets.map((secret) => (
                <Badge key={secret.name} variant="warning">{secret.name}</Badge>
              ))}
              {pack.variables.length === 0 && pack.secrets.length === 0 && (
                <span className="text-[11px] text-surface-500">No config selected.</span>
              )}
            </div>
          </div>
      </div>
    </section>
  )
}
