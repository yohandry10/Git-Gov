import { Archive, CheckCircle2, RotateCcw, XCircle } from 'lucide-react'
import { useMemo, useState } from 'react'
import { Badge } from '@/components/shared/Badge'
import { Button } from '@/components/shared/Button'
import { formatTs } from '@/lib/timezone'
import { useControlPlaneStore } from '@/store/useControlPlaneStore'
import type { ComplianceFrameworkPackRecord } from '@/store/useControlPlaneStore/types'

function statusBadgeVariant(status: string) {
  if (status === 'reviewed') return 'success'
  if (status === 'needs_changes') return 'warning'
  if (status === 'rejected' || status === 'archived') return 'danger'
  return 'info'
}

function shortHash(value?: string | null): string {
  if (!value) return 'not available'
  return value.length > 16 ? value.slice(0, 16) : value
}

function packLabel(pack: ComplianceFrameworkPackRecord): string {
  return `${pack.framework_name} ${pack.framework_version}`
}

export function ComplianceFrameworkPackReviewPanel() {
  const packs = useControlPlaneStore((state) => state.complianceFrameworkPacks)
  const displayTimezone = useControlPlaneStore((state) => state.displayTimezone)
  const isReviewing = useControlPlaneStore((state) => state.isComplianceFrameworkPackReviewing)
  const reviewPack = useControlPlaneStore((state) => state.reviewComplianceFrameworkPack)
  const [activePackId, setActivePackId] = useState('')
  const [reviewNotes, setReviewNotes] = useState('')
  const [rejectedReason, setRejectedReason] = useState('')

  const activePack = useMemo(
    () => packs.find((pack) => pack.framework_pack_id === activePackId) ?? packs[0] ?? null,
    [activePackId, packs],
  )

  const selectedPackId = activePack?.framework_pack_id ?? ''
  const submitReview = async (
    status: 'needs_review' | 'reviewed' | 'needs_changes' | 'rejected' | 'archived',
  ) => {
    if (!selectedPackId) return
    const response = await reviewPack(selectedPackId, status, {
      review_notes_safe: reviewNotes,
      rejected_reason_safe: rejectedReason,
    })
    if (response) {
      setActivePackId(response.framework_pack_id)
      if (status === 'reviewed' || status === 'archived') {
        setReviewNotes('')
        setRejectedReason('')
      }
    }
  }

  if (packs.length === 0) {
    return null
  }

  return (
    <div className="mt-4 rounded-lg border border-white/8 bg-surface-900/60 p-3">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div>
          <div className="text-xs font-medium text-surface-200">Framework Pack Review</div>
          <p className="mt-1 text-[11px] text-surface-500">
            Customer packs are blocked from mapping until an admin marks the imported pack reviewed.
          </p>
        </div>
        {activePack && (
          <Badge variant={statusBadgeVariant(activePack.review_status)}>
            {activePack.review_status}
          </Badge>
        )}
      </div>

      <label htmlFor="framework-pack-review-select" className="mt-3 block text-[10px] font-medium uppercase tracking-widest text-surface-500">
        Imported pack
      </label>
      <select
        id="framework-pack-review-select"
        value={selectedPackId}
        onChange={(event) => setActivePackId(event.target.value)}
        className="mt-2 w-full rounded border border-surface-600 bg-surface-800 px-2 py-2 text-xs text-surface-100 focus:border-brand-400 focus:outline-none"
      >
        {packs.map((pack) => (
          <option key={pack.framework_pack_id} value={pack.framework_pack_id}>
            {packLabel(pack)} / {pack.review_status}
          </option>
        ))}
      </select>

      {activePack && (
        <>
          <div className="mt-3 grid grid-cols-1 gap-1 text-[11px] text-surface-400 md:grid-cols-2">
            <span className="truncate">Pack: <span className="font-mono text-surface-200">{activePack.framework_pack_id}</span></span>
            <span className="truncate">Framework: <span className="font-mono text-surface-200">{activePack.framework_id}</span></span>
            <span>Controls: <span className="text-surface-200">{activePack.control_count}</span></span>
            <span>Owner: <span className="text-surface-200">{activePack.owner_name}</span></span>
            <span className="truncate" title={activePack.pack_hash}>Hash: <span className="font-mono text-surface-200">{shortHash(activePack.pack_hash)}</span></span>
            <span>Imported: <span className="text-surface-200">{formatTs(activePack.created_at, displayTimezone)}</span></span>
            <span>Claims: <span className="text-surface-200">false</span></span>
            <span>GitGov certifies: <span className="text-surface-200">false</span></span>
            <span>Regulatory mapping: <span className="text-surface-200">false</span></span>
            <span>Auditor review: <span className="text-surface-200">{String(activePack.requires_auditor_review)}</span></span>
            {activePack.reviewed_at && (
              <span>Reviewed: <span className="text-surface-200">{formatTs(activePack.reviewed_at, displayTimezone)}</span></span>
            )}
            {activePack.reviewed_by_user_id && (
              <span className="truncate">Reviewer: <span className="text-surface-200">{activePack.reviewed_by_user_id}</span></span>
            )}
          </div>

          <textarea
            value={reviewNotes}
            onChange={(event) => setReviewNotes(event.target.value)}
            placeholder="Review notes"
            maxLength={1000}
            className="mt-3 h-20 w-full resize-y rounded border border-surface-600 bg-surface-950 px-2 py-2 text-[11px] text-surface-100 placeholder:text-surface-600 focus:border-brand-400 focus:outline-none"
          />
          <textarea
            value={rejectedReason}
            onChange={(event) => setRejectedReason(event.target.value)}
            placeholder="Rejection reason"
            maxLength={1000}
            className="mt-2 h-16 w-full resize-y rounded border border-surface-600 bg-surface-950 px-2 py-2 text-[11px] text-surface-100 placeholder:text-surface-600 focus:border-brand-400 focus:outline-none"
          />

          <div className="mt-3 grid grid-cols-2 gap-2 lg:grid-cols-5">
            <Button size="sm" variant="secondary" loading={isReviewing} onClick={() => void submitReview('reviewed')} title="Mark pack reviewed">
              <CheckCircle2 size={13} />
              Reviewed
            </Button>
            <Button size="sm" variant="secondary" loading={isReviewing} onClick={() => void submitReview('needs_changes')} title="Mark pack as needing changes">
              <RotateCcw size={13} />
              Changes
            </Button>
            <Button size="sm" variant="outline" loading={isReviewing} onClick={() => void submitReview('needs_review')} title="Return pack to review queue">
              <RotateCcw size={13} />
              Review
            </Button>
            <Button size="sm" variant="danger" loading={isReviewing} onClick={() => void submitReview('rejected')} title="Reject pack">
              <XCircle size={13} />
              Reject
            </Button>
            <Button size="sm" variant="danger" loading={isReviewing} onClick={() => void submitReview('archived')} title="Archive pack">
              <Archive size={13} />
              Archive
            </Button>
          </div>
        </>
      )}
    </div>
  )
}
