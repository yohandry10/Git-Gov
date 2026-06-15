import { CheckCircle2, ShieldAlert } from 'lucide-react'
import { useState } from 'react'
import { Badge } from '@/components/shared/Badge'
import { Button } from '@/components/shared/Button'
import { formatTs } from '@/lib/timezone'
import { useControlPlaneStore, type CompliancePeriodReportRecord } from '@/store/useControlPlaneStore'

function reviewBadgeVariant(status?: string | null): 'success' | 'warning' | 'danger' | 'neutral' | 'info' {
  if (status === 'reviewed') return 'success'
  if (status === 'needs_changes') return 'warning'
  if (status === 'rejected') return 'danger'
  if (status === 'needs_review') return 'info'
  return 'neutral'
}

interface CompliancePeriodReportReviewPanelProps {
  periodReport: CompliancePeriodReportRecord
  displayTimezone: string
}

export function CompliancePeriodReportReviewPanel({
  periodReport,
  displayTimezone,
}: CompliancePeriodReportReviewPanelProps) {
  const [reviewStatus, setReviewStatus] = useState(periodReport.review_status || 'reviewed')
  const [reviewNotesSafe, setReviewNotesSafe] = useState(periodReport.review_notes_safe ?? '')
  const isReviewing = useControlPlaneStore((state) => state.isCompliancePeriodReportReviewing)
  const reviewPeriodReport = useControlPlaneStore((state) => state.reviewCompliancePeriodReport)
  const isArchived = periodReport.retention_status === 'archived'

  const handleReview = async () => {
    await reviewPeriodReport(periodReport.period_report_id, reviewStatus, reviewNotesSafe)
  }

  return (
    <div className="mt-2 rounded border border-white/6 bg-surface-950 p-2 text-[11px]">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="flex items-center gap-2 text-surface-300">
          <CheckCircle2 size={13} className="text-brand-300" />
          Period report review
          <Badge variant={reviewBadgeVariant(periodReport.review_status)}>
            {periodReport.review_status.replace(/_/g, ' ')}
          </Badge>
        </div>
        {isArchived && (
          <Badge variant="neutral">archived read-only</Badge>
        )}
      </div>

      <div className="mt-2 grid grid-cols-1 gap-2 md:grid-cols-3">
        <div className="rounded border border-white/6 bg-white/[0.02] p-2">
          <div className="text-surface-500">Reviewer</div>
          <div className="mt-1 truncate font-mono text-surface-200">
            {periodReport.reviewed_by_user_id ?? 'not reviewed'}
          </div>
        </div>
        <div className="rounded border border-white/6 bg-white/[0.02] p-2">
          <div className="text-surface-500">Reviewed at</div>
          <div className="mt-1 text-surface-200">
            {periodReport.reviewed_at ? formatTs(periodReport.reviewed_at, displayTimezone) : 'not yet'}
          </div>
        </div>
        <div className="rounded border border-white/6 bg-white/[0.02] p-2">
          <div className="text-surface-500">Positioning</div>
          <div className="mt-1 flex items-center gap-1 text-surface-200">
            <ShieldAlert size={12} />
            no certification claim
          </div>
        </div>
      </div>

      <div className="mt-2 grid grid-cols-1 gap-2 md:grid-cols-[160px_minmax(0,1fr)_auto]">
        <select
          className="h-9 rounded border border-white/10 bg-surface-950 px-2 text-xs text-surface-100 outline-none focus:border-brand-400"
          value={reviewStatus}
          disabled={isArchived}
          onChange={(event) => setReviewStatus(event.target.value)}
          aria-label="Period report review status"
        >
          <option value="reviewed">Reviewed</option>
          <option value="needs_changes">Needs changes</option>
          <option value="rejected">Rejected</option>
          <option value="needs_review">Needs review</option>
        </select>
        <textarea
          className="min-h-9 rounded border border-white/10 bg-surface-950 px-2 py-2 text-xs text-surface-100 outline-none placeholder:text-surface-600 focus:border-brand-400"
          value={reviewNotesSafe}
          disabled={isArchived}
          onChange={(event) => setReviewNotesSafe(event.target.value)}
          maxLength={1000}
          placeholder="Safe review note"
          aria-label="Period report review notes"
        />
        <Button
          size="sm"
          variant="outline"
          loading={isReviewing}
          disabled={isArchived}
          onClick={() => void handleReview()}
          title="Save manual period report review metadata"
        >
          <CheckCircle2 size={13} />
          Review
        </Button>
      </div>

      {periodReport.review_notes_safe && (
        <p className="mt-2 leading-5 text-surface-400">{periodReport.review_notes_safe}</p>
      )}
    </div>
  )
}
