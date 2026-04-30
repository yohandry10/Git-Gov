import { useMemo, useState } from 'react'
import { AlertTriangle, Download, KeyRound, PackageCheck, ShieldCheck, Workflow } from 'lucide-react'
import { Badge } from '@/components/shared/Badge'
import { Button } from '@/components/shared/Button'
import {
  ADOPTION_MODULE_OPTIONS,
  ADOPTION_POLICY_PRESET_OPTIONS,
  ADOPTION_PROVIDER_OPTIONS,
  DEFAULT_ENTERPRISE_ADOPTION_PROFILE,
  buildEnterpriseAdoptionPack,
  buildEnterpriseAdoptionPackFilename,
  validateEnterpriseAdoptionProfile,
  type AdoptionModule,
  type AdoptionPolicyPreset,
  type AdoptionProvider,
  type EnterpriseAdoptionProfile,
} from './dashboard-helpers'

function cloneDefaultProfile(): EnterpriseAdoptionProfile {
  return {
    ...DEFAULT_ENTERPRISE_ADOPTION_PROFILE,
    providers: [...DEFAULT_ENTERPRISE_ADOPTION_PROFILE.providers],
    modules: [...DEFAULT_ENTERPRISE_ADOPTION_PROFILE.modules],
  }
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

export function EnterpriseAdoptionPanel() {
  const [profile, setProfile] = useState<EnterpriseAdoptionProfile>(() => cloneDefaultProfile())
  const pack = useMemo(() => buildEnterpriseAdoptionPack(profile), [profile])
  const validation = useMemo(() => validateEnterpriseAdoptionProfile(profile), [profile])
  const readinessTarget = pack.policy_rules.find((rule) => rule.rule === 'Release readiness target')?.setting ?? '0'
  const trendRule = pack.policy_rules.find((rule) => rule.rule === 'Vulnerability trend enforcement')?.setting ?? 'informational'

  const updateText = (
    field: 'customer_name' | 'repository_full_name' | 'default_branch' | 'jira_project_key',
    value: string,
  ) => {
    setProfile((current) => ({ ...current, [field]: value }))
  }

  const updatePolicyPreset = (policyPreset: AdoptionPolicyPreset) => {
    setProfile((current) => ({ ...current, policy_preset: policyPreset }))
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
          <p>Customer profile, governance modules, workflow plan, and secret-safe adoption pack.</p>
        </div>
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
      </div>

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
