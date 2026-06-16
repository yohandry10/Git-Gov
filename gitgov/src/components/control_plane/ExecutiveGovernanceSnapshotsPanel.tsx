import { useEffect, useMemo, useState } from 'react'
import { Archive, Download, Eye, FileJson, Plus } from 'lucide-react'
import { Badge } from '@/components/shared/Badge'
import { Button } from '@/components/shared/Button'
import { formatTs } from '@/lib/timezone'
import { useControlPlaneStore } from '@/store/useControlPlaneStore'
import type { MultiRepoExecutiveGovernanceQuery } from '@/store/useControlPlaneStore/types'

function shortHash(value: string | null | undefined): string {
  if (!value) return 'none'
  return value.length > 20 ? `${value.slice(0, 20)}...` : value
}

function filterLabel(filters: Record<string, unknown>): string {
  const active = Object.entries(filters)
    .filter(([, value]) => value !== null && value !== undefined && value !== '')
    .map(([key, value]) => `${key}:${String(value)}`)
  return active.length > 0 ? active.join(' ') : 'no filters'
}

type Props = {
  filters: MultiRepoExecutiveGovernanceQuery
  repositoryCount: number
}

export function ExecutiveGovernanceSnapshotsPanel({ filters, repositoryCount }: Props) {
  const selectedOrgName = useControlPlaneStore((state) => state.selectedOrgName)
  const displayTimezone = useControlPlaneStore((state) => state.displayTimezone)
  const snapshots = useControlPlaneStore((state) => state.executiveGovernanceSnapshots)
  const total = useControlPlaneStore((state) => state.executiveGovernanceSnapshotsTotal)
  const artifact = useControlPlaneStore((state) => state.executiveGovernanceSnapshotArtifact)
  const error = useControlPlaneStore((state) => state.executiveGovernanceSnapshotError)
  const isCreating = useControlPlaneStore((state) => state.isExecutiveGovernanceSnapshotCreating)
  const isLoading = useControlPlaneStore((state) => state.isExecutiveGovernanceSnapshotsLoading)
  const isDownloading = useControlPlaneStore((state) => state.isExecutiveGovernanceSnapshotDownloading)
  const isArchiving = useControlPlaneStore((state) => state.isExecutiveGovernanceSnapshotArchiving)
  const createSnapshot = useControlPlaneStore((state) => state.createExecutiveGovernanceSnapshot)
  const loadSnapshots = useControlPlaneStore((state) => state.loadExecutiveGovernanceSnapshots)
  const getSnapshot = useControlPlaneStore((state) => state.getExecutiveGovernanceSnapshot)
  const downloadSnapshot = useControlPlaneStore((state) => state.downloadExecutiveGovernanceSnapshot)
  const archiveSnapshot = useControlPlaneStore((state) => state.archiveExecutiveGovernanceSnapshot)
  const [name, setName] = useState('Executive governance snapshot')

  useEffect(() => {
    void loadSnapshots({ org_name: selectedOrgName || null, status: 'active', limit: 10, offset: 0 })
  }, [loadSnapshots, selectedOrgName])

  const scopedFilters = useMemo<MultiRepoExecutiveGovernanceQuery>(() => ({
    ...filters,
    org_name: null,
    limit: filters.limit ?? 100,
    offset: filters.offset ?? 0,
  }), [filters])

  const createCurrentSnapshot = () => {
    void createSnapshot({
      org_name: selectedOrgName || null,
      name: name.trim() || 'Executive governance snapshot',
      filters: scopedFilters,
      include_repository_rows: true,
      include_summary: true,
    })
  }

  return (
    <div className="mt-4 rounded border border-white/8 bg-white/[0.03] p-3">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="min-w-0">
          <div className="flex items-center gap-2">
            <FileJson size={15} className="text-brand-400" />
            <h3 className="text-sm font-medium text-surface-100">Executive snapshots</h3>
            <Badge variant="info">{total} active</Badge>
          </div>
          <p className="mt-1 text-[11px] text-surface-500">
            Hashable JSON artifact from the current filtered executive view.
          </p>
        </div>
        <div className="flex min-w-0 flex-wrap items-center gap-2">
          <input
            value={name}
            onChange={(event) => setName(event.target.value)}
            className="h-8 min-w-[220px] rounded border border-surface-700 bg-surface-900 px-2 text-xs text-surface-200 focus:border-surface-400 focus:outline-none"
            maxLength={160}
          />
          <Button size="sm" variant="secondary" loading={isCreating} onClick={createCurrentSnapshot}>
            <Plus size={14} />
            Create Snapshot
          </Button>
          <Button
            size="sm"
            variant="outline"
            loading={isLoading}
            onClick={() => void loadSnapshots({ org_name: selectedOrgName || null, status: 'active', limit: 10, offset: 0 })}
          >
            Refresh
          </Button>
        </div>
      </div>

      <div className="mt-2 text-[11px] text-surface-500">
        Current view: {repositoryCount} repositories, {filterLabel(scopedFilters as Record<string, unknown>)}.
      </div>

      {error && (
        <div className="mt-3 rounded border border-danger-500/30 bg-danger-500/10 px-3 py-2 text-xs text-danger-100">
          {error}
        </div>
      )}

      <div className="mt-3 divide-y divide-white/6 rounded border border-white/8">
        {snapshots.map((snapshot) => (
          <div key={snapshot.snapshot_id} className="grid gap-2 px-3 py-2 text-xs md:grid-cols-[minmax(150px,1fr)_90px_minmax(160px,1fr)_120px_auto]">
            <div className="min-w-0">
              <div className="truncate font-medium text-surface-100" title={snapshot.name}>{snapshot.name}</div>
              <div className="truncate text-[11px] text-surface-500" title={filterLabel(snapshot.filters)}>
                {filterLabel(snapshot.filters)}
              </div>
            </div>
            <div className="text-surface-300">{snapshot.repository_count} repos</div>
            <div className="truncate font-mono text-[11px] text-surface-400" title={snapshot.artifact_hash}>
              {shortHash(snapshot.artifact_hash)}
            </div>
            <div className="text-[11px] text-surface-500">{formatTs(snapshot.created_at, displayTimezone)}</div>
            <div className="flex justify-end gap-1">
              <Button size="sm" variant="ghost" title="View snapshot" onClick={() => void getSnapshot(snapshot.snapshot_id, { org_name: selectedOrgName || null })}>
                <Eye size={14} />
              </Button>
              <Button size="sm" variant="ghost" loading={isDownloading} title="Download snapshot" onClick={() => void downloadSnapshot(snapshot.snapshot_id, { org_name: selectedOrgName || null })}>
                <Download size={14} />
              </Button>
              <Button size="sm" variant="ghost" loading={isArchiving} title="Archive snapshot" onClick={() => void archiveSnapshot(snapshot.snapshot_id, selectedOrgName || null)}>
                <Archive size={14} />
              </Button>
            </div>
          </div>
        ))}
        {snapshots.length === 0 && (
          <div className="px-3 py-5 text-center text-xs text-surface-500">
            No executive snapshots yet.
          </div>
        )}
      </div>

      {artifact && (
        <div className="mt-3 rounded border border-white/8 bg-surface-950 p-3">
          <div className="mb-2 text-[11px] text-surface-500">Latest artifact preview</div>
          <pre className="max-h-48 overflow-auto whitespace-pre-wrap text-[11px] text-surface-300">
            {JSON.stringify(artifact, null, 2)}
          </pre>
        </div>
      )}
    </div>
  )
}
