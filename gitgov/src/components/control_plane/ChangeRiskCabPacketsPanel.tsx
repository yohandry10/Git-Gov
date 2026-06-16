import { useEffect, useMemo, useState } from 'react'
import { Archive, ClipboardCheck, Download, FileJson, PackageCheck, RefreshCw, Save } from 'lucide-react'
import { Badge } from '@/components/shared/Badge'
import { Button } from '@/components/shared/Button'
import { formatTs } from '@/lib/timezone'
import { useControlPlaneStore } from '@/store/useControlPlaneStore'
import type {
  ChangeRiskCabPacketRecord,
  ChangeRiskCabPacketReviewStatus,
  ChangeRiskEvaluationRecord,
} from '@/store/useControlPlaneStore/types'

const CAB_REVIEW_STATUSES: ChangeRiskCabPacketReviewStatus[] = [
  'pending_review',
  'reviewed',
  'accepted_risk',
  'needs_mitigation',
  'returned_to_owner',
  'rejected',
]

function safeDownloadName(value: string): string {
  return value.replace(/[^a-zA-Z0-9._-]+/g, '-').replace(/^-+|-+$/g, '').slice(0, 96) || 'packet'
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

function reviewLabel(status: string): string {
  return status.replaceAll('_', ' ')
}

function packetVariant(status: string): 'success' | 'warning' | 'danger' | 'info' | 'neutral' {
  if (status === 'active') return 'success'
  if (status === 'archived') return 'neutral'
  return 'info'
}

function reviewVariant(status: string): 'success' | 'warning' | 'danger' | 'info' | 'neutral' {
  if (status === 'reviewed' || status === 'accepted_risk') return 'success'
  if (status === 'needs_mitigation' || status === 'returned_to_owner') return 'warning'
  if (status === 'rejected') return 'danger'
  if (status === 'pending_review') return 'info'
  return 'neutral'
}

function shortHash(value?: string | null): string {
  if (!value) return 'not recorded'
  return value.length > 24 ? `${value.slice(0, 17)}...${value.slice(-6)}` : value
}

function readTotalEvaluations(artifact: Record<string, unknown> | null): number | null {
  const summary = artifact?.summary
  if (!summary || typeof summary !== 'object') return null
  const count = (summary as { total_evaluations?: unknown }).total_evaluations
  return typeof count === 'number' ? count : null
}

function readArtifactHash(artifact: Record<string, unknown> | null): string | null {
  const verification = artifact?.verification
  if (!verification || typeof verification !== 'object') return null
  const hash = (verification as { packet_hash?: unknown }).packet_hash
  return typeof hash === 'string' ? hash : null
}

function uniqueValues(values: Array<string | null | undefined>): string[] {
  return Array.from(new Set(values.map((value) => value?.trim()).filter(Boolean) as string[]))
}

function PacketRow({
  packet,
  displayTimezone,
  onDownload,
  onArchive,
  onReview,
  isDownloading,
  isArchiving,
}: {
  packet: ChangeRiskCabPacketRecord
  displayTimezone: string
  onDownload: (packet: ChangeRiskCabPacketRecord) => void
  onArchive: (packet: ChangeRiskCabPacketRecord) => void
  onReview: (packet: ChangeRiskCabPacketRecord) => void
  isDownloading: boolean
  isArchiving: boolean
}) {
  return (
    <div className="p-3 text-xs">
      <div className="flex flex-wrap items-start justify-between gap-2">
        <div className="min-w-0">
          <div className="flex flex-wrap items-center gap-2">
            <Badge variant={packetVariant(packet.status)}>{packet.status}</Badge>
            <Badge variant={reviewVariant(packet.review_status)}>{reviewLabel(packet.review_status)}</Badge>
            <span className="font-medium text-surface-100">{packet.name}</span>
          </div>
          <div className="mt-1 grid grid-cols-1 gap-1 text-[11px] text-surface-400 md:grid-cols-2">
            <span className="truncate">Packet: <span className="font-mono text-surface-200">{packet.packet_id}</span></span>
            <span>Created: <span className="text-surface-200">{formatTs(packet.created_at, displayTimezone)}</span></span>
            <span className="truncate">Hash: <span className="font-mono text-surface-200">{shortHash(packet.artifact_hash)}</span></span>
            <span>Downloads: <span className="text-surface-200">{packet.download_count}</span></span>
            {packet.review_updated_at && (
              <span>Review: <span className="text-surface-200">{formatTs(packet.review_updated_at, displayTimezone)}</span></span>
            )}
          </div>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <Button
            size="sm"
            variant="outline"
            onClick={() => onReview(packet)}
            title="Open manual CAB disposition"
          >
            <ClipboardCheck size={13} />
            Review
          </Button>
          <Button
            size="sm"
            variant="outline"
            loading={isDownloading}
            disabled={packet.status !== 'active'}
            onClick={() => onDownload(packet)}
            title="Download CAB packet JSON"
          >
            <Download size={13} />
            JSON
          </Button>
          <Button
            size="sm"
            variant="secondary"
            loading={isArchiving}
            disabled={packet.status !== 'active'}
            onClick={() => onArchive(packet)}
            title="Archive CAB packet"
          >
            <Archive size={13} />
            Archive
          </Button>
        </div>
      </div>
    </div>
  )
}

function CabPacketDispositionPanel({
  selectedPacket,
  displayTimezone,
  onSave,
  isSaving,
}: {
  selectedPacket: ChangeRiskCabPacketRecord
  displayTimezone: string
  onSave: (payload: {
    review_status: ChangeRiskCabPacketReviewStatus
    review_notes: string
    mitigation_notes: string
    decision_reason: string
    follow_up_required: boolean
    follow_up_owner: string
  }) => void
  isSaving: boolean
}) {
  const [reviewStatus, setReviewStatus] = useState<ChangeRiskCabPacketReviewStatus>(selectedPacket.review_status)
  const [reviewNotes, setReviewNotes] = useState(selectedPacket.review_notes_safe ?? '')
  const [mitigationNotes, setMitigationNotes] = useState(selectedPacket.mitigation_notes_safe ?? '')
  const [decisionReason, setDecisionReason] = useState(selectedPacket.decision_reason_safe ?? '')
  const [followUpRequired, setFollowUpRequired] = useState(Boolean(selectedPacket.follow_up_required))
  const [followUpOwner, setFollowUpOwner] = useState(selectedPacket.follow_up_owner_safe ?? '')

  return (
    <div className="border-b border-white/6 bg-surface-950/40 p-3 text-xs">
      <div className="rounded border border-warning-500/30 bg-warning-500/10 px-3 py-2 text-warning-100">
        Manual CAB disposition only. Does not approve, block, certify, or deploy.
      </div>
      <div className="mt-3 flex flex-wrap items-center gap-2">
        <Badge variant={reviewVariant(reviewStatus)}>{reviewLabel(reviewStatus)}</Badge>
        <span className="truncate font-mono text-surface-400">{selectedPacket.packet_id}</span>
        <span className="truncate font-mono text-surface-500">{shortHash(selectedPacket.artifact_hash)}</span>
        {selectedPacket.reviewed_at && (
          <span className="text-surface-500">Reviewed {formatTs(selectedPacket.reviewed_at, displayTimezone)}</span>
        )}
      </div>
      <div className="mt-3 grid grid-cols-1 gap-2 md:grid-cols-2">
        <label className="space-y-1">
          <span className="text-[11px] text-surface-500">Disposition</span>
          <select
            value={reviewStatus}
            onChange={(event) => setReviewStatus(event.target.value as ChangeRiskCabPacketReviewStatus)}
            className="h-9 w-full rounded border border-white/10 bg-surface-950/70 px-2 text-xs text-surface-100 outline-none transition-colors focus:border-brand-500/60"
          >
            {CAB_REVIEW_STATUSES.map((status) => (
              <option key={status} value={status}>{reviewLabel(status)}</option>
            ))}
          </select>
        </label>
        <label className="space-y-1">
          <span className="text-[11px] text-surface-500">Follow-up owner</span>
          <input
            value={followUpOwner}
            onChange={(event) => setFollowUpOwner(event.target.value)}
            maxLength={1000}
            className="h-9 w-full rounded border border-white/10 bg-surface-950/70 px-2 text-xs text-surface-100 outline-none transition-colors focus:border-brand-500/60"
          />
        </label>
      </div>
      <div className="mt-2 grid grid-cols-1 gap-2 md:grid-cols-3">
        <textarea
          value={reviewNotes}
          onChange={(event) => setReviewNotes(event.target.value)}
          placeholder="Review notes"
          maxLength={1000}
          className="min-h-[74px] rounded border border-white/10 bg-surface-950/70 px-2 py-2 text-xs text-surface-100 outline-none transition-colors placeholder:text-surface-600 focus:border-brand-500/60"
        />
        <textarea
          value={mitigationNotes}
          onChange={(event) => setMitigationNotes(event.target.value)}
          placeholder="Mitigation notes"
          maxLength={1000}
          className="min-h-[74px] rounded border border-white/10 bg-surface-950/70 px-2 py-2 text-xs text-surface-100 outline-none transition-colors placeholder:text-surface-600 focus:border-brand-500/60"
        />
        <textarea
          value={decisionReason}
          onChange={(event) => setDecisionReason(event.target.value)}
          placeholder="Decision reason"
          maxLength={1000}
          className="min-h-[74px] rounded border border-white/10 bg-surface-950/70 px-2 py-2 text-xs text-surface-100 outline-none transition-colors placeholder:text-surface-600 focus:border-brand-500/60"
        />
      </div>
      <div className="mt-3 flex flex-wrap items-center justify-between gap-2">
        <label className="inline-flex items-center gap-2 text-[11px] text-surface-300">
          <input
            type="checkbox"
            checked={followUpRequired}
            onChange={(event) => setFollowUpRequired(event.target.checked)}
            className="h-4 w-4 rounded border-white/20 bg-surface-950"
          />
          Follow-up required
        </label>
        <Button
          size="sm"
          loading={isSaving}
          onClick={() => onSave({
            review_status: reviewStatus,
            review_notes: reviewNotes,
            mitigation_notes: mitigationNotes,
            decision_reason: decisionReason,
            follow_up_required: followUpRequired,
            follow_up_owner: followUpOwner,
          })}
          title="Save manual CAB disposition"
        >
          <Save size={13} />
          Save disposition
        </Button>
      </div>
    </div>
  )
}

export function ChangeRiskCabPacketsPanel({
  selectedOrgName,
  repositoryFullName,
  branch,
  environment,
  reviewQueueFilter,
  evaluations,
  displayTimezone,
}: {
  selectedOrgName: string
  repositoryFullName: string
  branch: string
  environment: string
  reviewQueueFilter: string
  evaluations: ChangeRiskEvaluationRecord[]
  displayTimezone: string
}) {
  const cabPackets = useControlPlaneStore((state) => state.changeRiskCabPackets)
  const cabPacketsTotal = useControlPlaneStore((state) => state.changeRiskCabPacketsTotal)
  const cabPacketArtifact = useControlPlaneStore((state) => state.changeRiskCabPacketArtifact)
  const selectedCabPacket = useControlPlaneStore((state) => state.changeRiskCabPacket)
  const cabPacketReview = useControlPlaneStore((state) => state.changeRiskCabPacketReview)
  const isLoading = useControlPlaneStore((state) => state.isChangeRiskCabPacketsLoading)
  const isCreating = useControlPlaneStore((state) => state.isChangeRiskCabPacketCreating)
  const isDownloading = useControlPlaneStore((state) => state.isChangeRiskCabPacketDownloading)
  const isArchiving = useControlPlaneStore((state) => state.isChangeRiskCabPacketArchiving)
  const isReviewLoading = useControlPlaneStore((state) => state.isChangeRiskCabPacketReviewLoading)
  const isReviewUpdating = useControlPlaneStore((state) => state.isChangeRiskCabPacketReviewUpdating)
  const loadPackets = useControlPlaneStore((state) => state.loadChangeRiskCabPackets)
  const createPacket = useControlPlaneStore((state) => state.createChangeRiskCabPacket)
  const downloadPacket = useControlPlaneStore((state) => state.downloadChangeRiskCabPacket)
  const archivePacket = useControlPlaneStore((state) => state.archiveChangeRiskCabPacket)
  const getPacket = useControlPlaneStore((state) => state.getChangeRiskCabPacket)
  const getPacketReview = useControlPlaneStore((state) => state.getChangeRiskCabPacketReview)
  const updatePacketReview = useControlPlaneStore((state) => state.updateChangeRiskCabPacketReview)

  const [statusFilter, setStatusFilter] = useState<'all' | 'active' | 'archived'>('active')
  const [packetName, setPacketName] = useState('')
  const [reviewPacketId, setReviewPacketId] = useState<string | null>(null)

  const effectiveReviewStatus = reviewQueueFilter === 'all' ? null : reviewQueueFilter
  const defaultPacketName = useMemo(() => {
    const status = effectiveReviewStatus ? reviewLabel(effectiveReviewStatus) : 'all reviews'
    const stamp = new Date().toISOString().slice(0, 10)
    return `CAB packet - ${status} - ${stamp}`
  }, [effectiveReviewStatus])

  const packetQuery = useMemo(() => ({
    org_name: selectedOrgName || null,
    status: statusFilter === 'all' ? null : statusFilter,
    limit: 10,
    offset: 0,
  }), [selectedOrgName, statusFilter])

  useEffect(() => {
    void loadPackets(packetQuery)
  }, [loadPackets, packetQuery])

  const createFromFilters = async () => {
    await createPacket({
      org_name: selectedOrgName || null,
      name: packetName.trim() || defaultPacketName,
      repository_full_name: repositoryFullName.trim() || null,
      branch: repositoryFullName.trim() ? branch.trim() || null : null,
      environment: environment.trim() || null,
      review_status: effectiveReviewStatus,
      evaluation_ids: [],
      deployment_gate_ids: [],
    })
  }

  const createFromVisible = async () => {
    await createPacket({
      org_name: selectedOrgName || null,
      name: packetName.trim() || `${defaultPacketName} - visible selection`,
      repository_full_name: repositoryFullName.trim() || null,
      branch: repositoryFullName.trim() ? branch.trim() || null : null,
      environment: environment.trim() || null,
      review_status: effectiveReviewStatus,
      evaluation_ids: evaluations.map((evaluation) => evaluation.evaluation_id),
      deployment_gate_ids: uniqueValues(evaluations.map((evaluation) => evaluation.deployment_gate_id)),
    })
  }

  const handleDownload = async (packet: ChangeRiskCabPacketRecord) => {
    const artifact = await downloadPacket(packet.packet_id, { org_name: selectedOrgName || null })
    if (artifact) {
      downloadJson(`gitgov-change-risk-cab-${safeDownloadName(packet.packet_id)}.json`, artifact)
    }
  }

  const handleArchive = async (packet: ChangeRiskCabPacketRecord) => {
    await archivePacket(packet.packet_id, selectedOrgName || null)
  }

  const handleReview = async (packet: ChangeRiskCabPacketRecord) => {
    setReviewPacketId(packet.packet_id)
    await getPacket(packet.packet_id, { org_name: selectedOrgName || null })
    await getPacketReview(packet.packet_id, { org_name: selectedOrgName || null })
  }

  const selectedReviewPacket = cabPackets.find((packet) => packet.packet_id === reviewPacketId)
    ?? selectedCabPacket?.packet
    ?? null

  const handleSaveReview = async (payload: {
    review_status: ChangeRiskCabPacketReviewStatus
    review_notes: string
    mitigation_notes: string
    decision_reason: string
    follow_up_required: boolean
    follow_up_owner: string
  }) => {
    if (!selectedReviewPacket) return
    await updatePacketReview(selectedReviewPacket.packet_id, {
      org_name: selectedOrgName || null,
      ...payload,
    })
  }

  const selectedHash = readArtifactHash(cabPacketArtifact) ?? selectedCabPacket?.packet.artifact_hash ?? null
  const selectedEvaluationCount = readTotalEvaluations(cabPacketArtifact)

  return (
    <div className="rounded-lg border border-white/8 bg-surface-900/60">
      <div className="flex flex-wrap items-center justify-between gap-2 border-b border-white/6 px-3 py-2">
        <div>
          <div className="flex items-center gap-2">
            <PackageCheck size={14} className="text-brand-300" />
            <span className="text-[11px] font-medium text-surface-300">Change Risk CAB packets</span>
          </div>
          <div className="mt-1 text-[10px] text-surface-600">{cabPacketsTotal} total · manual JSON artifact</div>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <select
            value={statusFilter}
            onChange={(event) => setStatusFilter(event.target.value as 'all' | 'active' | 'archived')}
            className="h-8 min-w-[120px] rounded border border-white/10 bg-surface-950/70 px-2 text-xs text-surface-100 outline-none transition-colors focus:border-brand-500/60"
          >
            <option value="active">Active</option>
            <option value="archived">Archived</option>
            <option value="all">All</option>
          </select>
          <Button size="sm" variant="outline" loading={isLoading} onClick={() => void loadPackets(packetQuery)} title="Reload CAB packets">
            <RefreshCw size={13} />
            Refresh
          </Button>
        </div>
      </div>

      <div className="border-b border-white/6 p-3">
        <div className="grid grid-cols-1 gap-2 md:grid-cols-[1fr_auto_auto]">
          <input
            value={packetName}
            onChange={(event) => setPacketName(event.target.value)}
            placeholder={defaultPacketName}
            maxLength={160}
            className="h-9 rounded border border-white/10 bg-surface-950/70 px-2 text-xs text-surface-100 outline-none transition-colors placeholder:text-surface-600 focus:border-brand-500/60"
          />
          <Button size="sm" loading={isCreating} onClick={() => void createFromFilters()} title="Create packet from current filters">
            <FileJson size={14} />
            From filters
          </Button>
          <Button
            size="sm"
            variant="secondary"
            loading={isCreating}
            disabled={evaluations.length === 0}
            onClick={() => void createFromVisible()}
            title="Create packet from visible evaluations"
          >
            <PackageCheck size={14} />
            Visible
          </Button>
        </div>
        <div className="mt-2 text-[11px] text-surface-500">
          Uses the queue filters and/or visible evaluation IDs. It does not approve, block, deploy, mutate providers, or use agents.
        </div>
      </div>

      {cabPacketArtifact && (
        <div className="border-b border-white/6 bg-brand-500/8 p-3 text-xs text-brand-50">
          <div className="flex flex-wrap items-center gap-2">
            <Badge variant="info">artifact ready</Badge>
            {selectedEvaluationCount !== null && <span>{selectedEvaluationCount} evaluations</span>}
            <span className="truncate font-mono">{shortHash(selectedHash)}</span>
          </div>
        </div>
      )}

      {selectedReviewPacket && (
        <CabPacketDispositionPanel
          key={`${selectedReviewPacket.packet_id}-${selectedReviewPacket.review_updated_at ?? 'pending'}`}
          selectedPacket={selectedReviewPacket}
          displayTimezone={displayTimezone}
          onSave={handleSaveReview}
          isSaving={isReviewUpdating || isReviewLoading}
        />
      )}
      {cabPacketReview && selectedReviewPacket?.packet_id === cabPacketReview.packet_id && (
        <div className="border-b border-white/6 px-3 py-2 text-[11px] text-surface-500">
          Review source: manual only · artifact hash unchanged · no release blocking · no deployment execution
        </div>
      )}

      <div className="max-h-[320px] overflow-auto divide-y divide-white/6">
        {cabPackets.map((packet) => (
          <PacketRow
            key={packet.packet_id}
            packet={packet}
            displayTimezone={displayTimezone}
            onDownload={handleDownload}
            onArchive={handleArchive}
            onReview={handleReview}
            isDownloading={isDownloading}
            isArchiving={isArchiving}
          />
        ))}
        {cabPackets.length === 0 && (
          <div className="p-6 text-center text-xs text-surface-600">
            No Change Risk CAB packets match this filter.
          </div>
        )}
      </div>
    </div>
  )
}
