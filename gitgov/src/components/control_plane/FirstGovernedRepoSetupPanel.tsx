import { useEffect, useMemo, useRef, useState } from 'react'
import { NavLink } from 'react-router-dom'
import { CheckCircle2, ClipboardCheck, GitBranch, GitPullRequest, ListChecks, PlayCircle, RefreshCw, Save, ShieldCheck, Workflow } from 'lucide-react'
import clsx from 'clsx'
import { Badge } from '@/components/shared/Badge'
import { Button } from '@/components/shared/Button'
import { useControlPlaneStore } from '@/store/useControlPlaneStore'
import {
  DEFAULT_FIRST_GOVERNED_REPO_SETUP,
  FIRST_GOVERNED_REPO_GOAL_OPTIONS,
  FIRST_GOVERNED_REPO_MODULE_OPTIONS,
  FIRST_GOVERNED_REPO_POLICY_PRESET_OPTIONS,
  FIRST_GOVERNED_REPO_PROVIDER_OPTIONS,
  buildFirstGovernedRepoSetupBaseline,
  normalizeFirstGovernedRepoSetupDraft,
  validateFirstGovernedRepoSetupDraft,
  type FirstGovernedRepoModule,
  type FirstGovernedRepoProvider,
  type FirstGovernedRepoSetupDraft,
} from './dashboard-helpers'

function optionClass(selected: boolean, disabled = false): string {
  return clsx(
    'min-h-[68px] rounded-lg border p-3 text-left transition-colors',
    selected
      ? 'border-brand-500/60 bg-brand-500/12 text-brand-100'
      : 'border-white/10 bg-white/[0.03] text-surface-300 hover:border-white/20 hover:bg-white/[0.05]',
    disabled && 'cursor-not-allowed opacity-70',
  )
}

function toggleValue<T extends string>(values: T[], value: T, required = false): T[] {
  if (required) return values
  return values.includes(value)
    ? values.filter((candidate) => candidate !== value)
    : [...values, value]
}

function readinessBadge(readiness: string): 'success' | 'warning' | 'info' {
  if (readiness === 'baseline_ready') return 'success'
  if (readiness === 'needs_preview') return 'warning'
  return 'info'
}

function readinessLabel(readiness: string): string {
  if (readiness === 'baseline_ready') return 'Ready'
  if (readiness === 'needs_preview') return 'Preview'
  return 'Repo'
}

function gapLabel(gap: string): string {
  const labels: Record<string, string> = {
    repository_full_name: 'Select governed repo',
    policy_workflow_preview: 'Review policy/workflow preview',
    quality_gate_evidence: 'Add quality gate evidence',
    formal_approval_policy: 'Add formal approval policy',
    provider_evidence: 'Validate provider evidence',
  }
  return labels[gap] ?? gap
}

function wizardString(state: Record<string, unknown> | null, key: string, fallback = 'not_started'): string {
  const value = state?.[key]
  return typeof value === 'string' ? value : fallback
}

function wizardArray(state: Record<string, unknown> | null, key: string): string[] {
  const value = state?.[key]
  return Array.isArray(value) ? value.filter((item): item is string => typeof item === 'string') : []
}

function wizardObjectArray(state: Record<string, unknown> | null, key: string): Array<Record<string, unknown>> {
  const value = state?.[key]
  return Array.isArray(value)
    ? value.filter((item): item is Record<string, unknown> => Boolean(item) && typeof item === 'object' && !Array.isArray(item))
    : []
}

function nextDraft(
  draft: FirstGovernedRepoSetupDraft,
  patch: Partial<Omit<FirstGovernedRepoSetupDraft, 'baseline'>>,
  previewAck = draft.baseline.policy_workflow_preview_acknowledged,
): FirstGovernedRepoSetupDraft {
  const merged = { ...draft, ...patch }
  const baseline = buildFirstGovernedRepoSetupBaseline({
    repository_full_name: merged.repository_full_name,
    default_branch: merged.default_branch,
    goal: merged.goal,
    selected_providers: merged.selected_providers,
    selected_modules: merged.selected_modules,
    policy_preset: merged.policy_preset,
    policyWorkflowPreviewAcknowledged: previewAck,
  })
  return {
    ...merged,
    status: baseline.gate_readiness === 'baseline_ready' ? 'ready' : 'draft',
    baseline,
  }
}

export function FirstGovernedRepoSetupPanel() {
  const selectedOrgName = useControlPlaneStore((state) => state.selectedOrgName)
  const isConnected = useControlPlaneStore((state) => state.isConnected)
  const persistedSetup = useControlPlaneStore((state) => state.firstGovernedRepoSetup)
  const persistedSetupUpdatedAt = useControlPlaneStore((state) => state.firstGovernedRepoSetupUpdatedAt)
  const isLoading = useControlPlaneStore((state) => state.isFirstGovernedRepoSetupLoading)
  const isSaving = useControlPlaneStore((state) => state.isFirstGovernedRepoSetupSaving)
  const setupError = useControlPlaneStore((state) => state.firstGovernedRepoSetupError)
  const wizardState = useControlPlaneStore((state) => state.firstGovernedRepoWizardState)
  const isWizardLoading = useControlPlaneStore((state) => state.isFirstGovernedRepoWizardLoading)
  const isWizardActionRunning = useControlPlaneStore((state) => state.isFirstGovernedRepoWizardActionRunning)
  const wizardError = useControlPlaneStore((state) => state.firstGovernedRepoWizardError)
  const loadSetup = useControlPlaneStore((state) => state.loadFirstGovernedRepoSetup)
  const saveSetup = useControlPlaneStore((state) => state.saveFirstGovernedRepoSetup)
  const loadWizardState = useControlPlaneStore((state) => state.loadFirstGovernedRepoWizardState)
  const createWizardRun = useControlPlaneStore((state) => state.createFirstGovernedRepoWizardRun)
  const validateWizardRun = useControlPlaneStore((state) => state.validateFirstGovernedRepoWizardRun)
  const planWizardRun = useControlPlaneStore((state) => state.planFirstGovernedRepoWizardRun)
  const completeWizardRun = useControlPlaneStore((state) => state.completeFirstGovernedRepoWizardRun)
  const [draft, setDraft] = useState<FirstGovernedRepoSetupDraft>(() =>
    normalizeFirstGovernedRepoSetupDraft(DEFAULT_FIRST_GOVERNED_REPO_SETUP),
  )
  const [dirty, setDirty] = useState(false)
  const dirtyRef = useRef(false)
  const [localError, setLocalError] = useState<string | null>(null)

  useEffect(() => {
    if (!isConnected) return
    let cancelled = false
    void loadWizardState(selectedOrgName || undefined).then((response) => {
      if (cancelled || dirtyRef.current) return
      const record = response?.setup ?? null
      setDraft(normalizeFirstGovernedRepoSetupDraft(record ?? DEFAULT_FIRST_GOVERNED_REPO_SETUP))
    })
    return () => {
      cancelled = true
    }
  }, [isConnected, loadWizardState, selectedOrgName])

  const validation = useMemo(() => validateFirstGovernedRepoSetupDraft(draft), [draft])
  const wizardCurrentStep = wizardString(wizardState, 'current_step')
  const wizardStatus = wizardString(wizardState, 'status')
  const wizardGaps = wizardArray(wizardState, 'gaps')
  const providerHealth = wizardObjectArray(wizardState, 'provider_health')
  const updatedAtLabel = persistedSetupUpdatedAt
    ? new Date(persistedSetupUpdatedAt).toLocaleString()
    : 'Not saved'

  const updateDraft = (
    patch: Partial<Omit<FirstGovernedRepoSetupDraft, 'baseline'>>,
    previewAck?: boolean,
  ) => {
    dirtyRef.current = true
    setDirty(true)
    setLocalError(null)
    setDraft((current) => nextDraft(current, patch, previewAck))
  }

  const save = async () => {
    const finalDraft = nextDraft(draft, {})
    const finalValidation = validateFirstGovernedRepoSetupDraft(finalDraft)
    if (finalValidation.errors.length > 0) {
      setLocalError(finalValidation.errors.join(' '))
      return
    }
    const record = await saveSetup({
      status: finalValidation.ready ? 'ready' : 'draft',
      goal: finalDraft.goal,
      repository_full_name: finalDraft.repository_full_name,
      default_branch: finalDraft.default_branch,
      selected_providers: finalDraft.selected_providers,
      selected_modules: finalDraft.selected_modules,
      policy_preset: finalDraft.policy_preset,
      baseline: finalDraft.baseline,
    }, selectedOrgName || undefined)
    if (record) {
      setDraft(normalizeFirstGovernedRepoSetupDraft(record))
      dirtyRef.current = false
      setDirty(false)
      setLocalError(null)
    }
  }

  const wizardPayload = (status?: FirstGovernedRepoSetupDraft['status']) => ({
    status: status ?? draft.status,
    goal: draft.goal,
    repository_full_name: draft.repository_full_name,
    default_branch: draft.default_branch,
    selected_providers: draft.selected_providers,
    selected_modules: draft.selected_modules,
    policy_preset: draft.policy_preset,
    baseline: draft.baseline,
  })

  const applyWizardResponse = (record: typeof persistedSetup | null | undefined) => {
    if (!record) return
    setDraft(normalizeFirstGovernedRepoSetupDraft(record))
    dirtyRef.current = false
    setDirty(false)
    setLocalError(null)
  }

  const startWizard = async () => {
    const finalDraft = nextDraft(draft, {})
    const finalValidation = validateFirstGovernedRepoSetupDraft(finalDraft)
    if (finalValidation.errors.length > 0) {
      setLocalError(finalValidation.errors.join(' '))
      return
    }
    const response = await createWizardRun({
      status: finalValidation.ready ? 'ready' : 'draft',
      goal: finalDraft.goal,
      repository_full_name: finalDraft.repository_full_name,
      default_branch: finalDraft.default_branch,
      selected_providers: finalDraft.selected_providers,
      selected_modules: finalDraft.selected_modules,
      policy_preset: finalDraft.policy_preset,
      baseline: finalDraft.baseline,
    }, selectedOrgName || undefined)
    applyWizardResponse(response?.setup)
  }

  const runWizardStep = async (step: 'validate' | 'plan' | 'complete') => {
    const runId = persistedSetup?.run_id
    if (!runId) {
      setLocalError('Save or start the wizard run before this step.')
      return
    }
    const payload = wizardPayload(step === 'complete' ? 'completed' : undefined)
    const response = step === 'validate'
      ? await validateWizardRun(runId, payload, selectedOrgName || undefined)
      : step === 'plan'
        ? await planWizardRun(runId, payload, selectedOrgName || undefined)
        : await completeWizardRun(runId, payload, selectedOrgName || undefined)
    applyWizardResponse(response?.setup)
  }

  const refresh = async () => {
    dirtyRef.current = false
    setDirty(false)
    const response = await loadWizardState(selectedOrgName || undefined)
    const record = response?.setup ?? await loadSetup(selectedOrgName || undefined)
    setDraft(normalizeFirstGovernedRepoSetupDraft(record ?? DEFAULT_FIRST_GOVERNED_REPO_SETUP))
  }

  return (
    <section className="glass-panel p-5">
      <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
        <div>
          <div className="flex flex-wrap items-center gap-2">
            <div className="flex h-9 w-9 items-center justify-center rounded-lg border border-brand-500/20 bg-brand-500/10">
              <ClipboardCheck size={18} className="text-brand-300" />
            </div>
            <div>
              <h2 className="text-sm font-semibold text-surface-100">First Governed Repo Setup</h2>
              <p className="mt-1 text-xs leading-5 text-surface-400">
                Deployment Gates 0.1 baseline for the first repo.
              </p>
            </div>
            <Badge variant={readinessBadge(draft.baseline.gate_readiness)}>
              {readinessLabel(draft.baseline.gate_readiness)}
            </Badge>
            {dirty && <Badge variant="warning">Unsaved</Badge>}
            {persistedSetup?.run_id && <Badge variant="neutral">Run {persistedSetup.run_id.slice(0, 8)}</Badge>}
          </div>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <Button variant="secondary" size="sm" onClick={refresh} loading={isLoading || isWizardLoading}>
            <RefreshCw size={14} />
            Refresh
          </Button>
          <Button size="sm" onClick={save} loading={isSaving} disabled={!isConnected}>
            <Save size={14} />
            Save setup
          </Button>
          <NavLink
            to="/governance/releases"
            className="inline-flex items-center justify-center gap-1.5 rounded-lg border border-brand-500/40 px-3 py-1.5 text-sm font-medium text-brand-300 transition-colors hover:border-brand-400 hover:bg-brand-500/10"
          >
            <Workflow size={14} />
            Gate simulation
          </NavLink>
        </div>
      </div>

      {(setupError || wizardError || localError) && (
        <div className="mt-4 rounded-lg border border-danger-500/20 bg-danger-500/8 p-3 text-xs leading-5 text-danger-200">
          {localError ?? wizardError ?? setupError}
        </div>
      )}

      <div className="mt-4 grid grid-cols-1 gap-3 xl:grid-cols-[minmax(0,1.2fr)_minmax(340px,0.8fr)]">
        <div className="space-y-4">
          <div className="grid grid-cols-1 gap-3 md:grid-cols-[minmax(0,1fr)_180px]">
            <label className="block">
              <span className="mb-1.5 flex items-center gap-1.5 text-xs font-medium text-surface-300">
                <GitPullRequest size={13} />
                Repository
              </span>
              <input
                value={draft.repository_full_name}
                onChange={(event) => updateDraft({ repository_full_name: event.target.value })}
                placeholder="owner/repo"
                className="w-full rounded-lg border border-white/10 bg-surface-950/50 px-3 py-2 text-sm text-surface-100 outline-none transition-colors placeholder:text-surface-600 focus:border-brand-500/60"
              />
            </label>
            <label className="block">
              <span className="mb-1.5 flex items-center gap-1.5 text-xs font-medium text-surface-300">
                <GitBranch size={13} />
                Branch
              </span>
              <input
                value={draft.default_branch}
                onChange={(event) => updateDraft({ default_branch: event.target.value })}
                className="w-full rounded-lg border border-white/10 bg-surface-950/50 px-3 py-2 text-sm text-surface-100 outline-none transition-colors focus:border-brand-500/60"
              />
            </label>
          </div>

          <div>
            <p className="mb-2 text-xs font-medium uppercase tracking-widest text-surface-500">Goal</p>
            <div className="grid grid-cols-1 gap-2 md:grid-cols-2 xl:grid-cols-4">
              {FIRST_GOVERNED_REPO_GOAL_OPTIONS.map((option) => (
                <button
                  key={option.id}
                  type="button"
                  className={optionClass(draft.goal === option.id)}
                  onClick={() => updateDraft({ goal: option.id })}
                >
                  <span className="text-sm font-semibold">{option.label}</span>
                  <span className="mt-1 block text-[11px] leading-4 text-surface-400">{option.description}</span>
                </button>
              ))}
            </div>
          </div>

          <div>
            <p className="mb-2 text-xs font-medium uppercase tracking-widest text-surface-500">Policy preset</p>
            <div className="grid grid-cols-1 gap-2 md:grid-cols-3">
              {FIRST_GOVERNED_REPO_POLICY_PRESET_OPTIONS.map((option) => (
                <button
                  key={option.id}
                  type="button"
                  className={optionClass(draft.policy_preset === option.id)}
                  onClick={() => updateDraft({ policy_preset: option.id })}
                >
                  <span className="text-sm font-semibold">{option.label}</span>
                  <span className="mt-1 block text-[11px] leading-4 text-surface-400">{option.description}</span>
                </button>
              ))}
            </div>
          </div>

          <div className="grid grid-cols-1 gap-4 xl:grid-cols-2">
            <div>
              <p className="mb-2 text-xs font-medium uppercase tracking-widest text-surface-500">Providers</p>
              <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
                {FIRST_GOVERNED_REPO_PROVIDER_OPTIONS.map((option) => {
                  const selected = draft.selected_providers.includes(option.id)
                  return (
                    <button
                      key={option.id}
                      type="button"
                      className={optionClass(selected, option.required)}
                      onClick={() =>
                        updateDraft({
                          selected_providers: toggleValue(
                            draft.selected_providers,
                            option.id as FirstGovernedRepoProvider,
                            option.required,
                          ),
                        })
                      }
                    >
                      <span className="flex items-center justify-between gap-2 text-sm font-semibold">
                        {option.label}
                        {option.required && <Badge variant="info">Required</Badge>}
                      </span>
                      <span className="mt-1 block text-[11px] leading-4 text-surface-400">{option.description}</span>
                    </button>
                  )
                })}
              </div>
            </div>
            <div>
              <p className="mb-2 text-xs font-medium uppercase tracking-widest text-surface-500">Modules</p>
              <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
                {FIRST_GOVERNED_REPO_MODULE_OPTIONS.map((option) => {
                  const selected = draft.selected_modules.includes(option.id)
                  return (
                    <button
                      key={option.id}
                      type="button"
                      className={optionClass(selected, option.required)}
                      onClick={() =>
                        updateDraft({
                          selected_modules: toggleValue(
                            draft.selected_modules,
                            option.id as FirstGovernedRepoModule,
                            option.required,
                          ),
                        })
                      }
                    >
                      <span className="flex items-center justify-between gap-2 text-sm font-semibold">
                        {option.label}
                        {option.required && <Badge variant="info">Required</Badge>}
                      </span>
                      <span className="mt-1 block text-[11px] leading-4 text-surface-400">{option.description}</span>
                    </button>
                  )
                })}
              </div>
            </div>
          </div>
        </div>

        <aside className="space-y-3">
          <div className="rounded-lg border border-brand-500/20 bg-brand-500/8 p-3">
            <div className="flex items-center justify-between gap-2">
              <div className="flex items-center gap-2">
                <ListChecks size={15} className="text-brand-300" />
                <h3 className="text-sm font-semibold text-surface-100">Integration wizard</h3>
              </div>
              <Badge variant={wizardStatus === 'completed' ? 'success' : 'info'}>{wizardCurrentStep}</Badge>
            </div>
            <div className="mt-3 grid grid-cols-2 gap-2">
              <div className="rounded-md border border-white/8 bg-surface-950/40 p-2">
                <p className="text-[10px] uppercase tracking-widest text-surface-500">Status</p>
                <p className="mt-1 text-sm font-semibold text-surface-100">{wizardStatus}</p>
              </div>
              <div className="rounded-md border border-white/8 bg-surface-950/40 p-2">
                <p className="text-[10px] uppercase tracking-widest text-surface-500">Backend gaps</p>
                <p className="mono-data mt-1 text-lg font-semibold text-surface-100">{wizardGaps.length}</p>
              </div>
            </div>
            {providerHealth.length > 0 && (
              <div className="mt-3 space-y-2">
                {providerHealth.map((provider) => {
                  const providerId = typeof provider.provider === 'string' ? provider.provider : 'provider'
                  const status = typeof provider.status === 'string' ? provider.status : 'needs-evidence'
                  return (
                    <div key={providerId} className="flex items-center justify-between gap-2 rounded-md border border-white/8 bg-surface-950/40 px-2 py-1.5">
                      <span className="text-xs font-medium text-surface-300">{providerId}</span>
                      <Badge variant={status === 'ready' ? 'success' : status === 'needs-config' ? 'warning' : 'info'}>
                        {status}
                      </Badge>
                    </div>
                  )
                })}
              </div>
            )}
            <div className="mt-3 grid grid-cols-2 gap-2">
              <Button variant="secondary" size="sm" onClick={startWizard} loading={isWizardActionRunning} disabled={!isConnected}>
                <PlayCircle size={14} />
                Start
              </Button>
              <Button
                variant="secondary"
                size="sm"
                onClick={() => void runWizardStep('validate')}
                loading={isWizardActionRunning}
                disabled={!isConnected || !persistedSetup?.run_id}
              >
                <ShieldCheck size={14} />
                Validate
              </Button>
              <Button
                variant="secondary"
                size="sm"
                onClick={() => void runWizardStep('plan')}
                loading={isWizardActionRunning}
                disabled={!isConnected || !persistedSetup?.run_id}
              >
                <Workflow size={14} />
                Plan
              </Button>
              <Button
                size="sm"
                onClick={() => void runWizardStep('complete')}
                loading={isWizardActionRunning}
                disabled={!isConnected || !persistedSetup?.run_id || draft.baseline.gate_readiness !== 'baseline_ready'}
              >
                <CheckCircle2 size={14} />
                Complete
              </Button>
            </div>
          </div>

          <button
            type="button"
            onClick={() =>
              updateDraft(
                {},
                !draft.baseline.policy_workflow_preview_acknowledged,
              )
            }
            className={clsx(
              'flex w-full items-start gap-3 rounded-lg border p-3 text-left transition-colors',
              draft.baseline.policy_workflow_preview_acknowledged
                ? 'border-success-500/25 bg-success-500/8'
                : 'border-warning-500/25 bg-warning-500/8',
            )}
          >
            <CheckCircle2
              size={18}
              className={draft.baseline.policy_workflow_preview_acknowledged ? 'text-success-400' : 'text-warning-400'}
            />
            <span>
              <span className="block text-sm font-semibold text-surface-100">Policy and workflow preview</span>
              <span className="mt-1 block text-xs leading-5 text-surface-400">
                {draft.baseline.policy_workflow_preview_acknowledged ? 'Reviewed' : 'Pending review'}
              </span>
            </span>
          </button>

          <div className="rounded-lg border border-white/10 bg-white/[0.03] p-3">
            <div className="flex items-center gap-2">
              <ShieldCheck size={15} className="text-brand-300" />
              <h3 className="text-sm font-semibold text-surface-100">Baseline result</h3>
            </div>
            <div className="mt-3 grid grid-cols-2 gap-2">
              <div className="rounded-md border border-white/8 bg-surface-950/40 p-2">
                <p className="text-[10px] uppercase tracking-widest text-surface-500">Providers</p>
                <p className="mono-data mt-1 text-lg font-semibold text-surface-100">{draft.selected_providers.length}</p>
              </div>
              <div className="rounded-md border border-white/8 bg-surface-950/40 p-2">
                <p className="text-[10px] uppercase tracking-widest text-surface-500">Modules</p>
                <p className="mono-data mt-1 text-lg font-semibold text-surface-100">{draft.selected_modules.length}</p>
              </div>
            </div>
            <div className="mt-3 rounded-md border border-white/8 bg-surface-950/40 p-2">
              <p className="text-[10px] uppercase tracking-widest text-surface-500">First result</p>
              <p className="mt-1 text-xs font-medium text-surface-200">{draft.baseline.first_result.status}</p>
              <p className="mt-1 text-[11px] leading-4 text-surface-500">Saved: {updatedAtLabel}</p>
            </div>
          </div>

          <div className="rounded-lg border border-white/10 bg-white/[0.03] p-3">
            <div className="mb-2 flex items-center gap-2">
              <ShieldCheck size={15} className="text-warning-300" />
              <h3 className="text-sm font-semibold text-surface-100">Action Center gaps</h3>
            </div>
            {validation.gaps.length === 0 ? (
              <p className="text-xs leading-5 text-success-300">No setup gaps remain for advisory gate simulation.</p>
            ) : (
              <div className="space-y-2">
                {validation.gaps.map((gap) => (
                  <div key={gap} className="flex items-center justify-between gap-2 rounded-md border border-white/8 bg-surface-950/40 px-2 py-1.5">
                    <span className="text-xs text-surface-300">{gapLabel(gap)}</span>
                    <Badge variant="warning">Gap</Badge>
                  </div>
                ))}
              </div>
            )}
          </div>
        </aside>
      </div>
    </section>
  )
}
