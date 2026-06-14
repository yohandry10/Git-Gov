import { Plus, Trash2 } from 'lucide-react'
import { Badge } from '@/components/shared/Badge'
import {
  ADOPTION_RELEASE_GOVERNANCE_MODE_OPTIONS,
  buildReleaseGovernanceEnvironmentRows,
  type AdoptionReleaseGovernanceMode,
  type EnterpriseReleaseGovernanceEnvironmentRow,
  type EnterpriseReleaseGovernancePolicy,
} from './dashboard-helpers'

interface ReleaseGovernanceEnvironmentPolicyPanelProps {
  policy: EnterpriseReleaseGovernancePolicy
  badgeVariant: 'success' | 'warning' | 'info' | 'neutral' | 'danger'
  onBaseModeChange: (mode: AdoptionReleaseGovernanceMode) => void
  onBaseEnvironmentChange: (environment: string) => void
  onAddOverride: () => void
  onOverrideEnvironmentChange: (index: number, environment: string) => void
  onOverrideModeChange: (index: number, mode: AdoptionReleaseGovernanceMode) => void
  onRemoveOverride: (index: number) => void
  selectedClass: (selected: boolean) => string
}

function environmentRowBadgeVariant(row: EnterpriseReleaseGovernanceEnvironmentRow): 'warning' | 'info' | 'neutral' {
  if (row.enforcement === 'blocking') return 'warning'
  if (row.enforcement === 'advisory') return 'info'
  return 'neutral'
}

function formatApproval(row: EnterpriseReleaseGovernanceEnvironmentRow): string {
  if (row.mode === 'quorum-required') return `Quorum ${row.quorum_summary}`
  if (row.approval_required) return 'Approval required'
  if (row.enforcement === 'advisory') return 'Advisory'
  return 'Record only'
}

export function ReleaseGovernanceEnvironmentPolicyPanel({
  policy,
  badgeVariant,
  onBaseModeChange,
  onBaseEnvironmentChange,
  onAddOverride,
  onOverrideEnvironmentChange,
  onOverrideModeChange,
  onRemoveOverride,
  selectedClass,
}: ReleaseGovernanceEnvironmentPolicyPanelProps) {
  const rows = buildReleaseGovernanceEnvironmentRows(policy)
  const overrides = policy.environment_overrides ?? []

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between gap-2">
        <span className="text-[10px] text-surface-500 uppercase tracking-widest">Release governance</span>
        <Badge variant={badgeVariant}>
          {policy.enforcement}
        </Badge>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
        {ADOPTION_RELEASE_GOVERNANCE_MODE_OPTIONS.map((option) => {
          const selected = policy.mode === option.id
          return (
            <button
              key={option.id}
              type="button"
              aria-pressed={selected}
              onClick={() => onBaseModeChange(option.id)}
              className={`rounded border px-3 py-2 text-xs font-medium transition-colors ${selectedClass(selected)}`}
            >
              {option.label}
            </button>
          )
        })}
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-[minmax(0,1fr)_minmax(140px,0.55fr)] gap-2">
        <label className="space-y-1">
          <span className="text-[10px] text-surface-500 uppercase tracking-widest">Base environment</span>
          <input
            value={policy.environment}
            onChange={(event) => onBaseEnvironmentChange(event.target.value)}
            className="w-full bg-surface-800 border border-surface-600 rounded px-2 py-1.5 text-xs text-surface-200 focus:outline-none focus:border-surface-400"
          />
        </label>
        <div className="rounded border border-white/8 bg-white/[0.03] p-2">
          <div className="text-[10px] uppercase tracking-widest text-surface-500">Quorum</div>
          <div className="mt-1 text-xs font-medium text-surface-100">
            {policy.quorum.enabled ? `${policy.quorum.rules.length} rules` : 'Off'}
          </div>
        </div>
      </div>

      <div className="space-y-2 rounded border border-white/8 bg-white/[0.02] p-3">
        <div className="flex items-center justify-between gap-2">
          <span className="text-[10px] text-surface-500 uppercase tracking-widest">Environment policy matrix</span>
          <button
            type="button"
            onClick={onAddOverride}
            className="inline-flex h-7 items-center gap-1 rounded border border-white/10 bg-white/[0.03] px-2 text-[11px] text-surface-200 hover:border-white/20 hover:bg-white/[0.05]"
            title="Add environment override"
          >
            <Plus size={13} />
            Add
          </button>
        </div>

        <div className="grid grid-cols-1 gap-2">
          {rows.map((row) => (
            <div
              key={`${row.source}-${row.environment}-${row.override_index ?? 'base'}`}
              className="rounded border border-white/8 bg-surface-900/40 p-2"
            >
              <div className="flex flex-wrap items-center justify-between gap-2">
                <div>
                  <div className="text-xs font-medium text-surface-100">{row.environment}</div>
                  <div className="text-[11px] text-surface-500">
                    {row.source === 'base' ? 'Base policy' : 'Environment override'} · {formatApproval(row)}
                  </div>
                </div>
                <Badge variant={environmentRowBadgeVariant(row)}>
                  {row.enforcement}
                </Badge>
              </div>
            </div>
          ))}
        </div>

        {overrides.length === 0 ? (
          <div className="text-[11px] text-surface-500">No environment overrides configured.</div>
        ) : (
          <div className="space-y-2">
            {overrides.map((override, index) => (
              <div key={`${override.environment}-${index}`} className="rounded border border-white/8 bg-white/[0.03] p-2">
                <div className="grid grid-cols-[minmax(0,1fr)_auto] gap-2">
                  <input
                    value={override.environment}
                    onChange={(event) => onOverrideEnvironmentChange(index, event.target.value)}
                    className="w-full bg-surface-800 border border-surface-600 rounded px-2 py-1.5 text-xs text-surface-200 focus:outline-none focus:border-surface-400"
                    aria-label={`Release governance override environment ${index + 1}`}
                  />
                  <button
                    type="button"
                    onClick={() => onRemoveOverride(index)}
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
                        onClick={() => onOverrideModeChange(index, option.id)}
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
  )
}
