import { useEffect, useMemo } from 'react'
import { Download, FileCheck2, RefreshCw, RotateCcw } from 'lucide-react'
import { Badge } from '@/components/shared/Badge'
import { Button } from '@/components/shared/Button'
import { formatTs } from '@/lib/timezone'
import { useControlPlaneStore } from '@/store/useControlPlaneStore'
import type {
  ChangeRiskCabDecisionManifestRecord,
  ChangeRiskCabPacketRecord,
} from '@/store/useControlPlaneStore/types'

function safeDownloadName(value: string): string {
  return value.replace(/[^a-zA-Z0-9._-]+/g, '-').replace(/^-+|-+$/g, '').slice(0, 96) || 'manifest'
}

function downloadJson(filename: string, data: unknown) {
  const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = filename
  document.body.appendChild(link)
  link.click()
  document.body.removeChild(link)
  URL.revokeObjectURL(url)
}

function shortHash(value?: string | null): string {
  if (!value) return 'not recorded'
  return value.length > 24 ? `${value.slice(0, 17)}...${value.slice(-6)}` : value
}

function manifestVariant(status: string): 'success' | 'warning' | 'danger' | 'info' | 'neutral' {
  if (status === 'active') return 'success'
  if (status === 'revoked') return 'neutral'
  return 'info'
}

function ManifestRow({
  manifest,
  displayTimezone,
  onDownload,
  onRevoke,
  isDownloading,
  isRevoking,
}: {
  manifest: ChangeRiskCabDecisionManifestRecord
  displayTimezone: string
  onDownload: (manifest: ChangeRiskCabDecisionManifestRecord) => void
  onRevoke: (manifest: ChangeRiskCabDecisionManifestRecord) => void
  isDownloading: boolean
  isRevoking: boolean
}) {
  return (
    <div className="flex flex-wrap items-start justify-between gap-2 border-t border-white/6 px-3 py-2 text-xs">
      <div className="min-w-0">
        <div className="flex flex-wrap items-center gap-2">
          <Badge variant={manifestVariant(manifest.status)}>{manifest.status}</Badge>
          <Badge variant="info">{manifest.review_status_snapshot.replaceAll('_', ' ')}</Badge>
          <span className="truncate font-mono text-surface-200">{manifest.manifest_id}</span>
        </div>
        <div className="mt-1 grid grid-cols-1 gap-1 text-[11px] text-surface-500 md:grid-cols-2">
          <span className="truncate">Manifest: <span className="font-mono text-surface-300">{shortHash(manifest.manifest_hash)}</span></span>
          <span className="truncate">Packet: <span className="font-mono text-surface-300">{shortHash(manifest.cab_packet_hash)}</span></span>
          <span>Created: <span className="text-surface-300">{formatTs(manifest.created_at, displayTimezone)}</span></span>
          <span>Downloads: <span className="text-surface-300">{manifest.download_count}</span></span>
        </div>
      </div>
      <div className="flex flex-wrap items-center gap-2">
        <Button
          size="sm"
          variant="outline"
          loading={isDownloading}
          disabled={manifest.status !== 'active'}
          onClick={() => onDownload(manifest)}
          title="Download decision manifest JSON"
        >
          <Download size={13} />
          JSON
        </Button>
        <Button
          size="sm"
          variant="secondary"
          loading={isRevoking}
          disabled={manifest.status !== 'active'}
          onClick={() => onRevoke(manifest)}
          title="Revoke decision manifest"
        >
          <RotateCcw size={13} />
          Revoke
        </Button>
      </div>
    </div>
  )
}

export function ChangeRiskCabDecisionManifestPanel({
  selectedPacket,
  selectedOrgName,
  displayTimezone,
}: {
  selectedPacket: ChangeRiskCabPacketRecord
  selectedOrgName: string
  displayTimezone: string
}) {
  const manifests = useControlPlaneStore((state) => state.changeRiskCabDecisionManifests)
  const total = useControlPlaneStore((state) => state.changeRiskCabDecisionManifestsTotal)
  const isLoading = useControlPlaneStore((state) => state.isChangeRiskCabDecisionManifestsLoading)
  const isCreating = useControlPlaneStore((state) => state.isChangeRiskCabDecisionManifestCreating)
  const isDownloading = useControlPlaneStore((state) => state.isChangeRiskCabDecisionManifestDownloading)
  const isRevoking = useControlPlaneStore((state) => state.isChangeRiskCabDecisionManifestRevoking)
  const loadManifests = useControlPlaneStore((state) => state.loadChangeRiskCabDecisionManifests)
  const createManifest = useControlPlaneStore((state) => state.createChangeRiskCabDecisionManifest)
  const downloadManifest = useControlPlaneStore((state) => state.downloadChangeRiskCabDecisionManifest)
  const revokeManifest = useControlPlaneStore((state) => state.revokeChangeRiskCabDecisionManifest)

  const query = useMemo(() => ({
    org_name: selectedOrgName || null,
    limit: 10,
    offset: 0,
  }), [selectedOrgName])
  const canCreate = selectedPacket.status === 'active' && selectedPacket.review_status !== 'pending_review'

  useEffect(() => {
    void loadManifests(selectedPacket.packet_id, query)
  }, [loadManifests, selectedPacket.packet_id, query])

  const handleCreate = async () => {
    const response = await createManifest(selectedPacket.packet_id, { org_name: selectedOrgName || null })
    if (response?.artifact) {
      downloadJson(
        `gitgov-cab-decision-manifest-${safeDownloadName(response.manifest.manifest_id)}.json`,
        response.artifact,
      )
    }
  }

  const handleDownload = async (manifest: ChangeRiskCabDecisionManifestRecord) => {
    const artifact = await downloadManifest(manifest.manifest_id, { org_name: selectedOrgName || null })
    if (artifact) {
      downloadJson(`gitgov-cab-decision-manifest-${safeDownloadName(manifest.manifest_id)}.json`, artifact)
    }
  }

  const handleRevoke = async (manifest: ChangeRiskCabDecisionManifestRecord) => {
    await revokeManifest(manifest.manifest_id, selectedOrgName || null)
  }

  return (
    <div className="border-b border-white/6 bg-surface-950/30 text-xs">
      <div className="flex flex-wrap items-center justify-between gap-2 px-3 py-2">
        <div>
          <div className="flex items-center gap-2">
            <FileCheck2 size={14} className="text-brand-300" />
            <span className="text-[11px] font-medium text-surface-300">Decision Manifest</span>
          </div>
          <div className="mt-1 text-[10px] text-surface-600">
            Manual evidence only. Does not approve, block, certify, or deploy.
          </div>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <Button size="sm" variant="outline" loading={isLoading} onClick={() => void loadManifests(selectedPacket.packet_id, query)} title="Reload decision manifests">
            <RefreshCw size={13} />
            Refresh
          </Button>
          <Button size="sm" loading={isCreating} disabled={!canCreate} onClick={() => void handleCreate()} title="Create decision manifest from current CAB disposition">
            <FileCheck2 size={13} />
            Create
          </Button>
        </div>
      </div>
      <div className="px-3 pb-2 text-[11px] text-surface-500">
        {total} total · packet hash <span className="font-mono text-surface-300">{shortHash(selectedPacket.artifact_hash)}</span>
      </div>
      {manifests.map((manifest) => (
        <ManifestRow
          key={manifest.manifest_id}
          manifest={manifest}
          displayTimezone={displayTimezone}
          onDownload={handleDownload}
          onRevoke={handleRevoke}
          isDownloading={isDownloading}
          isRevoking={isRevoking}
        />
      ))}
      {manifests.length === 0 && (
        <div className="border-t border-white/6 px-3 py-3 text-center text-[11px] text-surface-600">
          No decision manifests for this packet yet.
        </div>
      )}
    </div>
  )
}
