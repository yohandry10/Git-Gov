import { useCallback, useEffect } from 'react'
import { BarChart3, RefreshCw, ShieldAlert, ShieldCheck } from 'lucide-react'
import { Badge } from '@/components/shared/Badge'
import { Button } from '@/components/shared/Button'
import { formatTs } from '@/lib/timezone'
import { useControlPlaneStore } from '@/store/useControlPlaneStore'

function postureVariant(posture: string): 'success' | 'warning' | 'danger' | 'info' | 'neutral' {
  if (posture === 'healthy') return 'success'
  if (posture === 'review') return 'warning'
  if (posture === 'attention') return 'danger'
  return 'neutral'
}

function shortHash(value: string | null | undefined): string {
  if (!value) return 'none'
  return value.length > 18 ? `${value.slice(0, 18)}...` : value
}

export function MultiRepoExecutiveGovernancePanel() {
  const selectedOrgName = useControlPlaneStore((state) => state.selectedOrgName)
  const executiveView = useControlPlaneStore((state) => state.multiRepoExecutiveGovernance)
  const updatedAt = useControlPlaneStore((state) => state.multiRepoExecutiveGovernanceUpdatedAt)
  const isLoading = useControlPlaneStore((state) => state.isMultiRepoExecutiveGovernanceLoading)
  const error = useControlPlaneStore((state) => state.multiRepoExecutiveGovernanceError)
  const displayTimezone = useControlPlaneStore((state) => state.displayTimezone)
  const loadExecutiveView = useControlPlaneStore((state) => state.loadMultiRepoExecutiveGovernance)

  const refresh = useCallback(() => {
    void loadExecutiveView({
      org_name: selectedOrgName || null,
      limit: 25,
      offset: 0,
    })
  }, [loadExecutiveView, selectedOrgName])

  useEffect(() => {
    void refresh()
  }, [refresh])

  const totals = executiveView?.totals
  const repositories = executiveView?.repositories ?? []

  return (
    <section id="multi-repo-executive-governance" className="glass-panel p-5 scroll-mt-4">
      <div className="card-header mb-4">
        <div>
          <div className="flex items-center gap-2">
            <BarChart3 size={16} className="text-brand-400" />
            <h2>Executive Governance View</h2>
            <Badge variant={repositories.length > 0 ? 'success' : 'info'}>
              {repositories.length} repos
            </Badge>
          </div>
          <p>Read-only repository posture from Deployment Gates, Change Risk, CAB packets, and manifests.</p>
        </div>
        <Button
          size="sm"
          variant="outline"
          loading={isLoading}
          onClick={refresh}
          title="Refresh executive governance view"
        >
          <RefreshCw size={14} />
          Refresh
        </Button>
      </div>

      <div className="grid grid-cols-2 gap-2 md:grid-cols-6">
        <div className="rounded border border-white/8 bg-white/[0.03] p-3">
          <div className="text-[10px] text-surface-500">Repositories</div>
          <div className="mt-1 text-sm text-surface-100">{totals?.repositories ?? 0}</div>
        </div>
        <div className="rounded border border-white/8 bg-white/[0.03] p-3">
          <div className="text-[10px] text-surface-500">Gates</div>
          <div className="mt-1 text-sm text-surface-100">{totals?.gate_count ?? 0}</div>
        </div>
        <div className="rounded border border-white/8 bg-white/[0.03] p-3">
          <div className="text-[10px] text-surface-500">Blocked</div>
          <div className="mt-1 text-sm text-surface-100">{totals?.blocked_gate_count ?? 0}</div>
        </div>
        <div className="rounded border border-white/8 bg-white/[0.03] p-3">
          <div className="text-[10px] text-surface-500">High risk</div>
          <div className="mt-1 text-sm text-surface-100">{totals?.high_risk_count ?? 0}</div>
        </div>
        <div className="rounded border border-white/8 bg-white/[0.03] p-3">
          <div className="text-[10px] text-surface-500">CAB packets</div>
          <div className="mt-1 text-sm text-surface-100">{totals?.cab_packet_count ?? 0}</div>
        </div>
        <div className="rounded border border-white/8 bg-white/[0.03] p-3">
          <div className="text-[10px] text-surface-500">Loaded</div>
          <div className="mt-1 truncate text-xs text-surface-200">
            {updatedAt ? formatTs(updatedAt, displayTimezone) : 'Not loaded'}
          </div>
        </div>
      </div>

      <div className="mt-3 rounded border border-warning-500/20 bg-warning-500/8 px-3 py-2 text-[11px] text-warning-100">
        Advisory only. Does not approve, block, certify, deploy, mutate providers, or mutate repositories.
      </div>

      {error && (
        <div className="mt-3 rounded border border-danger-500/30 bg-danger-500/10 px-3 py-2 text-xs text-danger-100">
          {error}
        </div>
      )}

      <div className="mt-4 overflow-x-auto rounded-lg border border-white/8 bg-surface-900/60">
        <div className="grid min-w-[860px] grid-cols-[minmax(180px,2fr)_110px_90px_90px_90px_90px_minmax(140px,1fr)] gap-2 border-b border-white/6 px-3 py-2 text-[10px] font-medium uppercase text-surface-500">
          <span>Repository</span>
          <span>Posture</span>
          <span>Gates</span>
          <span>Risk</span>
          <span>CAB</span>
          <span>Manifests</span>
          <span>Latest</span>
        </div>
        <div className="max-h-[360px] overflow-auto divide-y divide-white/6">
          {repositories.map((repo) => (
            <div
              key={repo.repository_full_name}
              className="grid min-w-[860px] grid-cols-[minmax(180px,2fr)_110px_90px_90px_90px_90px_minmax(140px,1fr)] gap-2 px-3 py-3 text-xs"
            >
              <span className="truncate font-medium text-surface-100" title={repo.repository_full_name}>
                {repo.repository_full_name}
              </span>
              <span>
                <Badge variant={postureVariant(repo.posture)}>
                  {repo.posture === 'attention' ? <ShieldAlert size={12} /> : <ShieldCheck size={12} />}
                  {repo.posture}
                </Badge>
              </span>
              <span className="text-surface-300">
                {repo.gate_count}
                <span className="text-surface-600"> / {repo.blocked_gate_count} blocked</span>
              </span>
              <span className="text-surface-300">
                {repo.change_risk_count}
                <span className="text-surface-600"> / {repo.high_risk_count} high</span>
              </span>
              <span className="text-surface-300">{repo.cab_packet_count}</span>
              <span className="text-surface-300">
                {repo.cab_manifest_count}
                <span className="text-surface-600"> / {repo.revoked_manifest_count} revoked</span>
              </span>
              <span className="min-w-0 text-[11px] text-surface-400">
                <span className="block truncate">Gate: {repo.latest_gate_decision || 'none'}</span>
                <span className="block truncate">Risk: {repo.latest_risk_level || 'none'} / {repo.latest_review_status || 'none'}</span>
                <span className="block truncate font-mono" title={repo.latest_manifest_hash || undefined}>
                  {shortHash(repo.latest_manifest_hash)}
                </span>
              </span>
            </div>
          ))}
          {repositories.length === 0 && (
            <div className="px-3 py-6 text-center text-xs text-surface-500">
              No repository governance evidence loaded yet.
            </div>
          )}
        </div>
      </div>
    </section>
  )
}
