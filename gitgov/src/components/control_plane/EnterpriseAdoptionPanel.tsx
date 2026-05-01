import { useEffect, useMemo, useState } from 'react'
import { Activity, AlertTriangle, Download, KeyRound, PackageCheck, Plus, Save, ShieldCheck, Trash2, Workflow } from 'lucide-react'
import { Badge } from '@/components/shared/Badge'
import { Button } from '@/components/shared/Button'
import { useControlPlaneStore } from '@/store/useControlPlaneStore'
import {
  ADOPTION_MODULE_OPTIONS,
  ADOPTION_POLICY_PRESET_OPTIONS,
  ADOPTION_PROVIDER_OPTIONS,
  ADOPTION_RELEASE_GOVERNANCE_MODE_OPTIONS,
  DEFAULT_ENTERPRISE_ADOPTION_PROFILE,
  buildReleaseGovernancePolicy,
  buildEnterpriseAdoptionPack,
  buildEnterpriseAdoptionPackFilename,
  buildEnterpriseWorkflowTemplatePack,
  buildEnterpriseWorkflowTemplatePackFilename,
  buildEnterpriseProviderHealth,
  normalizeEnterpriseAdoptionProfile,
  validateEnterpriseAdoptionProfile,
  type AdoptionModule,
  type AdoptionPolicyPreset,
  type AdoptionProvider,
  type AdoptionReleaseGovernanceMode,
  type EnterpriseAdoptionProfile,
  type EnterpriseReleaseGovernancePolicy,
  type EnterpriseProviderHealthStatus,
} from './dashboard-helpers'

function cloneDefaultProfile(): EnterpriseAdoptionProfile {
  return normalizeEnterpriseAdoptionProfile(DEFAULT_ENTERPRISE_ADOPTION_PROFILE)
}

function toggleValue<T extends string>(values: T[], value: T): T[] {
  return values.includes(value)
    ? values.filter((candidate) => candidate !== value)
    : [...values, value]
}

function selectedClass(selected: boolean): string {
  return selected
    ? 'border-brand-500/60 bg-brand-500/12 text-brand-200'
    : 'border-white/10 bg-white/[0.03] text-surface-300 hover:border-white/20 hover:bg-white/[0.05]'
}

function providerHealthBadgeVariant(status: EnterpriseProviderHealthStatus): 'success' | 'warning' | 'info' {
  if (status === 'ready') return 'success'
  if (status === 'needs-config') return 'warning'
  return 'info'
}

function providerHealthLabel(status: EnterpriseProviderHealthStatus): string {
  if (status === 'ready') return 'Ready'
  if (status === 'needs-config') return 'Config'
  return 'Evidence'
}

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
  const loadEnterpriseAdoptionProfile = useControlPlaneStore((state) => state.loadEnterpriseAdoptionProfile)
  const saveEnterpriseAdoptionProfile = useControlPlaneStore((state) => state.saveEnterpriseAdoptionProfile)
  const [profile, setProfile] = useState<EnterpriseAdoptionProfile>(() => cloneDefaultProfile())
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

  useEffect(() => {
    void loadEnterpriseAdoptionProfile(selectedOrgName || undefined)
  }, [loadEnterpriseAdoptionProfile, selectedOrgName])

  useEffect(() => {
    if (!persistedProfile) return
    setProfile(normalizeEnterpriseAdoptionProfile(persistedProfile))
  }, [persistedProfile])

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
      release_governance: buildReleaseGovernancePolicy(
        mode,
        current.release_governance?.environment ?? 'production',
      ),
      modules: mode === 'record-only' || current.modules.includes('formal-approval')
        ? current.modules
        : [...current.modules, 'formal-approval'],
    }))
  }

  const updateReleaseGovernanceEnvironment = (environment: string) => {
    setProfile((current) => {
      const currentGovernance = current.release_governance ?? buildReleaseGovernancePolicy('record-only')
      return {
        ...current,
        release_governance: {
          ...currentGovernance,
          environment: environment.trim() || 'production',
        },
      }
    })
  }

  const updateReleaseGovernanceOverrides = (
    updater: (overrides: EnterpriseReleaseGovernancePolicy[]) => EnterpriseReleaseGovernancePolicy[],
  ) => {
    setProfile((current) => {
      const currentGovernance = current.release_governance ?? buildReleaseGovernancePolicy('record-only')
      return {
        ...current,
        release_governance: {
          ...currentGovernance,
          environment_overrides: updater(currentGovernance.environment_overrides ?? []),
        },
      }
    })
  }

  const addReleaseGovernanceOverride = () => {
    setProfile((current) => {
      const currentGovernance = current.release_governance ?? buildReleaseGovernancePolicy('record-only')
      const usedEnvironments = new Set(
        [currentGovernance.environment, ...(currentGovernance.environment_overrides ?? []).map((override) => override.environment)]
          .map((environment) => environment.trim().toLowerCase())
          .filter(Boolean),
      )
      const environment = ['production', 'staging', 'development'].find((candidate) => !usedEnvironments.has(candidate)) ?? `environment-${(currentGovernance.environment_overrides ?? []).length + 1}`
      return {
        ...current,
        modules: current.modules.includes('formal-approval') ? current.modules : [...current.modules, 'formal-approval'],
        release_governance: {
          ...currentGovernance,
          environment_overrides: [
            ...(currentGovernance.environment_overrides ?? []),
            buildReleaseGovernancePolicy('approval-required', environment),
          ],
        },
      }
    })
  }

  const updateReleaseGovernanceOverrideEnvironment = (index: number, environment: string) => {
    updateReleaseGovernanceOverrides((overrides) => overrides.map((override, overrideIndex) => (
      overrideIndex === index
        ? { ...override, environment: environment.trim() || 'production' }
        : override
    )))
  }

  const updateReleaseGovernanceOverrideMode = (index: number, mode: AdoptionReleaseGovernanceMode) => {
    updateReleaseGovernanceOverrides((overrides) => overrides.map((override, overrideIndex) => (
      overrideIndex === index
        ? buildReleaseGovernancePolicy(mode, override.environment || 'production')
        : override
    )))
  }

  const removeReleaseGovernanceOverride = (index: number) => {
    updateReleaseGovernanceOverrides((overrides) => overrides.filter((_, overrideIndex) => overrideIndex !== index))
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

  const saveProfile = async () => {
    if (!validation.valid) return
    await saveEnterpriseAdoptionProfile(profile, selectedOrgName || undefined)
  }

  return (
    <section className="glass-panel p-5">
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
        </div>
      </div>

      {(isProfileLoading || profileError) && (
        <div className={`mb-4 rounded border p-3 text-xs ${profileError ? 'border-warning-500/20 bg-warning-500/8 text-warning-100' : 'border-white/8 bg-white/[0.03] text-surface-300'}`}>
          {profileError ?? 'Loading saved profile...'}
        </div>
      )}

      <div className="grid grid-cols-1 xl:grid-cols-[minmax(0,1fr)_minmax(320px,0.8fr)] gap-4">
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

          <div className="space-y-2">
            <div className="flex items-center justify-between gap-2">
              <span className="text-[10px] text-surface-500 uppercase tracking-widest">Release governance</span>
              <Badge variant={releaseGovernanceBadgeVariant}>
                {releaseGovernance.enforcement}
              </Badge>
            </div>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
              {ADOPTION_RELEASE_GOVERNANCE_MODE_OPTIONS.map((option) => {
                const selected = releaseGovernance.mode === option.id
                return (
                  <button
                    key={option.id}
                    type="button"
                    aria-pressed={selected}
                    onClick={() => updateReleaseGovernanceMode(option.id)}
                    className={`rounded border px-3 py-2 text-xs font-medium transition-colors ${selectedClass(selected)}`}
                  >
                    {option.label}
                  </button>
                )
              })}
            </div>
            <div className="grid grid-cols-1 sm:grid-cols-[minmax(0,1fr)_minmax(140px,0.55fr)] gap-2">
              <label className="space-y-1">
                <span className="text-[10px] text-surface-500 uppercase tracking-widest">Environment</span>
                <input
                  value={releaseGovernance.environment}
                  onChange={(event) => updateReleaseGovernanceEnvironment(event.target.value)}
                  className="w-full bg-surface-800 border border-surface-600 rounded px-2 py-1.5 text-xs text-surface-200 focus:outline-none focus:border-surface-400"
                />
              </label>
              <div className="rounded border border-white/8 bg-white/[0.03] p-2">
                <div className="text-[10px] uppercase tracking-widest text-surface-500">Quorum</div>
                <div className="mt-1 text-xs font-medium text-surface-100">
                  {releaseGovernance.quorum.enabled ? `${releaseGovernance.quorum.rules.length} rules` : 'Off'}
                </div>
              </div>
            </div>
            <div className="space-y-2 rounded border border-white/8 bg-white/[0.02] p-3">
              <div className="flex items-center justify-between gap-2">
                <span className="text-[10px] text-surface-500 uppercase tracking-widest">Environment overrides</span>
                <button
                  type="button"
                  onClick={addReleaseGovernanceOverride}
                  className="inline-flex h-7 items-center gap-1 rounded border border-white/10 bg-white/[0.03] px-2 text-[11px] text-surface-200 hover:border-white/20 hover:bg-white/[0.05]"
                  title="Add environment override"
                >
                  <Plus size={13} />
                  Add
                </button>
              </div>
              {(releaseGovernance.environment_overrides ?? []).length === 0 ? (
                <div className="text-[11px] text-surface-500">None</div>
              ) : (
                <div className="space-y-2">
                  {(releaseGovernance.environment_overrides ?? []).map((override, index) => (
                    <div key={`${override.environment}-${index}`} className="rounded border border-white/8 bg-surface-900/40 p-2">
                      <div className="grid grid-cols-[minmax(0,1fr)_auto] gap-2">
                        <input
                          value={override.environment}
                          onChange={(event) => updateReleaseGovernanceOverrideEnvironment(index, event.target.value)}
                          className="w-full bg-surface-800 border border-surface-600 rounded px-2 py-1.5 text-xs text-surface-200 focus:outline-none focus:border-surface-400"
                        />
                        <button
                          type="button"
                          onClick={() => removeReleaseGovernanceOverride(index)}
                          className="inline-flex h-8 w-8 items-center justify-center rounded border border-white/10 bg-white/[0.03] text-surface-300 hover:border-warning-500/30 hover:text-warning-200"
                          title="Remove environment override"
                        >
                          <Trash2 size={14} />
                        </button>
                      </div>
                      <div className="mt-2 grid grid-cols-2 gap-2">
                        {ADOPTION_RELEASE_GOVERNANCE_MODE_OPTIONS.map((option) => {
                          const selected = override.mode === option.id
                          return (
                            <button
                              key={option.id}
                              type="button"
                              aria-pressed={selected}
                              onClick={() => updateReleaseGovernanceOverrideMode(index, option.id)}
                              className={`rounded border px-2 py-1.5 text-[11px] font-medium transition-colors ${selectedClass(selected)}`}
                            >
                              {option.label}
                            </button>
                          )
                        })}
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>

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
      </div>
    </section>
  )
}
