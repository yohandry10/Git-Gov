import { useState } from 'react'
import { CheckCircle2, Download, FileCheck2, History, MessageSquare, RefreshCw, ShieldAlert, Users } from 'lucide-react'
import { Badge } from '@/components/shared/Badge'
import { Button } from '@/components/shared/Button'
import { formatTs } from '@/lib/timezone'
import { useControlPlaneStore } from '@/store/useControlPlaneStore'

function shortHash(value?: string | null): string {
  if (!value) return 'not available'
  return value.length > 16 ? value.slice(0, 16) : value
}

function safeDownloadName(value: string): string {
  return value.trim().replace(/[^A-Za-z0-9._-]/g, '_').slice(0, 80) || 'framework-review-report'
}

function downloadJson(filename: string, data: unknown) {
  const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = filename
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
}

function reviewBadgeVariant(status?: string | null): 'success' | 'warning' | 'danger' | 'neutral' | 'info' {
  if (status === 'reviewed') return 'success'
  if (status === 'needs_changes') return 'warning'
  if (status === 'rejected') return 'danger'
  if (status === 'needs_review') return 'info'
  return 'neutral'
}

export function ComplianceFrameworkReviewReportPanel() {
  const [reviewStatus, setReviewStatus] = useState('reviewed')
  const [reviewNotesSafe, setReviewNotesSafe] = useState('')
  const [assignmentInput, setAssignmentInput] = useState('')
  const [assignmentNotesSafe, setAssignmentNotesSafe] = useState('')
  const [commentBodySafe, setCommentBodySafe] = useState('')
  const [commentSuggestion, setCommentSuggestion] = useState('')
  const mapping = useControlPlaneStore((state) => state.complianceEvidenceMapping?.mapping ?? null)
  const packageRecord = useControlPlaneStore((state) => state.complianceReviewPackage?.review_package ?? null)
  const report = useControlPlaneStore((state) => state.complianceFrameworkReviewReport?.report ?? null)
  const reportHistory = useControlPlaneStore((state) => state.complianceFrameworkReviewReports)
  const assignedReports = useControlPlaneStore((state) => state.assignedComplianceFrameworkReviewReports)
  const assignments = useControlPlaneStore((state) => state.complianceFrameworkReviewReportAssignments)
  const comments = useControlPlaneStore((state) => state.complianceFrameworkReviewReportComments)
  const reportArtifact = useControlPlaneStore((state) => state.complianceFrameworkReviewReportArtifact)
  const isCreating = useControlPlaneStore((state) => state.isComplianceFrameworkReviewReportCreating)
  const isHistoryLoading = useControlPlaneStore((state) => state.isComplianceFrameworkReviewReportsLoading)
  const isAssignedLoading = useControlPlaneStore((state) => state.isAssignedComplianceFrameworkReviewReportsLoading)
  const isAssignmentsLoading = useControlPlaneStore((state) => state.isComplianceFrameworkReviewReportAssignmentsLoading)
  const isAssignmentsSaving = useControlPlaneStore((state) => state.isComplianceFrameworkReviewReportAssignmentsSaving)
  const isCommentsLoading = useControlPlaneStore((state) => state.isComplianceFrameworkReviewReportCommentsLoading)
  const isCommenting = useControlPlaneStore((state) => state.isComplianceFrameworkReviewReportCommenting)
  const isReviewing = useControlPlaneStore((state) => state.isComplianceFrameworkReviewReportReviewing)
  const isDownloading = useControlPlaneStore((state) => state.isComplianceFrameworkReviewReportDownloading)
  const displayTimezone = useControlPlaneStore((state) => state.displayTimezone)
  const createReport = useControlPlaneStore((state) => state.createComplianceFrameworkReviewReport)
  const loadReports = useControlPlaneStore((state) => state.loadComplianceFrameworkReviewReports)
  const loadAssignedReports = useControlPlaneStore((state) => state.loadAssignedComplianceFrameworkReviewReports)
  const loadAssignments = useControlPlaneStore((state) => state.loadComplianceFrameworkReviewReportAssignments)
  const saveAssignments = useControlPlaneStore((state) => state.saveComplianceFrameworkReviewReportAssignments)
  const loadComments = useControlPlaneStore((state) => state.loadComplianceFrameworkReviewReportComments)
  const createComment = useControlPlaneStore((state) => state.createComplianceFrameworkReviewReportComment)
  const reviewReport = useControlPlaneStore((state) => state.reviewComplianceFrameworkReviewReport)
  const downloadReport = useControlPlaneStore((state) => state.downloadComplianceFrameworkReviewReport)

  const canGenerate = Boolean(mapping?.mapping_id && packageRecord?.review_package_id)
  const canDownload = Boolean(report?.report_id)

  const handleGenerate = async () => {
    if (!mapping || !packageRecord) return
    await createReport(mapping.mapping_id, packageRecord.review_package_id)
  }

  const handleDownload = async () => {
    if (!report) return
    const artifact = await downloadReport(report.report_id)
    if (artifact) {
      downloadJson(`gitgov-framework-review-${safeDownloadName(report.report_id)}.json`, artifact)
    }
  }

  const handleLoadHistory = async () => {
    await loadReports({
      framework_id: mapping?.framework_id ?? null,
      limit: 25,
    })
  }

  const handleLoadAssigned = async () => {
    await loadAssignedReports({
      framework_id: mapping?.framework_id ?? null,
      limit: 25,
    })
  }

  const handleDownloadHistory = async (reportId: string) => {
    const artifact = await downloadReport(reportId)
    if (artifact) {
      downloadJson(`gitgov-framework-review-${safeDownloadName(reportId)}.json`, artifact)
    }
  }

  const handleReview = async () => {
    if (!report) return
    await reviewReport(report.report_id, reviewStatus, reviewNotesSafe)
  }

  const handleLoadCollaboration = async () => {
    if (!report) return
    await Promise.all([
      loadAssignments(report.report_id),
      loadComments(report.report_id),
    ])
  }

  const handleSaveAssignments = async () => {
    if (!report) return
    const auditorClientIds = assignmentInput.split(',').map((value) => value.trim()).filter(Boolean)
    await saveAssignments(report.report_id, auditorClientIds, assignmentNotesSafe)
  }

  const handleCreateComment = async () => {
    if (!report) return
    const created = await createComment(report.report_id, commentBodySafe, commentSuggestion || null)
    if (created) setCommentBodySafe('')
  }

  return (
    <div className="mt-4 rounded-lg border border-white/8 bg-surface-900/60 p-3">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <div className="flex items-center gap-2">
            <FileCheck2 size={14} className="text-brand-300" />
            <span className="text-xs font-medium text-surface-200">Framework Review Report</span>
            <Badge variant={report ? 'success' : 'info'}>{report ? 'report ready' : 'JSON export'}</Badge>
            {report && <Badge variant={reviewBadgeVariant(report.review_status)}>{report.review_status}</Badge>}
          </div>
          <p className="mt-1 text-[11px] leading-5 text-surface-500">
            Framework-specific evidence report with source hashes, review provenance, missing evidence, and no certification claim.
          </p>
        </div>
        <div className="flex gap-2">
          <Button
            size="sm"
            variant="secondary"
            loading={isCreating}
            disabled={!canGenerate}
            onClick={() => void handleGenerate()}
            title="Generate framework-specific review report"
          >
            <FileCheck2 size={13} />
            Report
          </Button>
          <Button
            size="sm"
            variant="outline"
            loading={isHistoryLoading}
            onClick={() => void handleLoadHistory()}
            title="Load recent framework review reports"
          >
            <RefreshCw size={13} />
            History
          </Button>
          <Button
            size="sm"
            variant="outline"
            loading={isAssignedLoading}
            onClick={() => void handleLoadAssigned()}
            title="Load framework review reports assigned to the current Auditor"
          >
            <Users size={13} />
            Assigned
          </Button>
          <Button
            size="sm"
            variant="outline"
            loading={isDownloading}
            disabled={!canDownload}
            onClick={() => void handleDownload()}
            title="Download the server-generated framework review report JSON"
          >
            <Download size={13} />
            Download
          </Button>
        </div>
      </div>

      <div className="mt-3 rounded border border-warning-500/20 bg-warning-500/8 p-2 text-[11px] text-warning-100">
        <div className="flex items-center gap-2 font-medium">
          <ShieldAlert size={13} />
          Manual review required
        </div>
        <p className="mt-1 leading-5">
          This report organizes evidence for customer or auditor review. It is not a certification, compliance score, or official regulatory compliance claim.
        </p>
      </div>

      <div className="mt-3 grid grid-cols-1 gap-2 text-[11px] md:grid-cols-3">
        <div className="rounded border border-white/6 bg-white/[0.03] p-2">
          <div className="text-surface-500">Report</div>
          <div className="mt-1 truncate font-mono text-surface-100">{report?.report_id ?? 'not generated'}</div>
          <div className="mt-1 truncate text-surface-500" title={report?.artifact_hash}>
            Hash: <span className="text-surface-300">{shortHash(report?.artifact_hash)}</span>
          </div>
        </div>
        <div className="rounded border border-white/6 bg-white/[0.03] p-2">
          <div className="text-surface-500">Framework</div>
          <div className="mt-1 truncate text-surface-100">{report?.framework_id ?? mapping?.framework_id ?? 'not available'}</div>
          <div className="mt-1 text-surface-500">
            Owner: <span className="text-surface-300">{report?.framework_owner_type ?? 'not available'}</span>
          </div>
        </div>
        <div className="rounded border border-white/6 bg-white/[0.03] p-2">
          <div className="text-surface-500">Source hashes</div>
          <div className="mt-1 truncate text-surface-500" title={report?.mapping_hash ?? packageRecord?.mapping_hash}>
            Mapping: <span className="font-mono text-surface-300">{shortHash(report?.mapping_hash ?? packageRecord?.mapping_hash)}</span>
          </div>
          <div className="mt-1 truncate text-surface-500" title={report?.review_package_hash ?? packageRecord?.artifact_hash}>
            Package: <span className="font-mono text-surface-300">{shortHash(report?.review_package_hash ?? packageRecord?.artifact_hash)}</span>
          </div>
        </div>
      </div>

      {report && (
        <div className="mt-3 grid grid-cols-2 gap-2 text-[11px] md:grid-cols-4">
          <span>Claims: <span className="font-mono text-surface-200">{String(report.compliance_claim || report.regulatory_claim || report.certification)}</span></span>
          <span>Auditor review: <span className="font-mono text-surface-200">{String(report.requires_auditor_review)}</span></span>
          <span>Pack: <span className="font-mono text-surface-200">{shortHash(report.pack_hash)}</span></span>
          <span>Created: <span className="text-surface-200">{formatTs(report.created_at, displayTimezone)}</span></span>
          <span>Review status: <span className="font-mono text-surface-200">{report.review_status}</span></span>
          <span>Reviewer: <span className="font-mono text-surface-200">{report.reviewed_by_user_id ?? 'not reviewed'}</span></span>
          <span>Reviewed: <span className="text-surface-200">{report.reviewed_at ? formatTs(report.reviewed_at, displayTimezone) : 'not yet'}</span></span>
        </div>
      )}

      {report && (
        <div className="mt-3 rounded border border-white/8 bg-white/[0.03] p-2">
          <div className="mb-2 flex items-center gap-2 text-xs font-medium text-surface-200">
            <CheckCircle2 size={13} className="text-brand-300" />
            Manual report review
            <Badge variant={reviewBadgeVariant(report.review_status)}>{report.review_status}</Badge>
          </div>
          <div className="grid grid-cols-1 gap-2 md:grid-cols-[180px_minmax(0,1fr)_auto]">
            <select
              className="h-9 rounded border border-white/10 bg-surface-950 px-2 text-xs text-surface-100 outline-none focus:border-brand-400"
              value={reviewStatus}
              onChange={(event) => setReviewStatus(event.target.value)}
              aria-label="Framework report review status"
            >
              <option value="reviewed">Reviewed</option>
              <option value="needs_changes">Needs changes</option>
              <option value="rejected">Rejected</option>
              <option value="needs_review">Needs review</option>
            </select>
            <textarea
              className="min-h-9 rounded border border-white/10 bg-surface-950 px-2 py-2 text-xs text-surface-100 outline-none placeholder:text-surface-600 focus:border-brand-400"
              value={reviewNotesSafe}
              onChange={(event) => setReviewNotesSafe(event.target.value)}
              maxLength={1000}
              placeholder="Safe review note"
              aria-label="Framework report review notes"
            />
            <Button
              size="sm"
              variant="outline"
              loading={isReviewing}
              onClick={() => void handleReview()}
              title="Save manual framework report review metadata"
            >
              <CheckCircle2 size={13} />
              Review
            </Button>
          </div>
          {report.review_notes_safe && (
            <p className="mt-2 text-[11px] leading-5 text-surface-400">{report.review_notes_safe}</p>
          )}
        </div>
      )}

      {report && (
        <div className="mt-3 rounded border border-white/8 bg-white/[0.03] p-2">
          <div className="mb-2 flex flex-wrap items-center justify-between gap-2">
            <div className="flex items-center gap-2 text-xs font-medium text-surface-200">
              <Users size={13} className="text-brand-300" />
              Auditor assignments and comments
              {assignments && <Badge variant="info">{assignments.assignments.filter((item) => item.assignment_status === 'active').length} active</Badge>}
              {comments && <Badge variant="neutral">{comments.count} comments</Badge>}
            </div>
            <Button
              size="sm"
              variant="outline"
              loading={isAssignmentsLoading || isCommentsLoading}
              onClick={() => void handleLoadCollaboration()}
              title="Load assignments and comments for this framework review report"
            >
              <RefreshCw size={13} />
              Load
            </Button>
          </div>
          <div className="grid grid-cols-1 gap-2 md:grid-cols-[minmax(0,1fr)_minmax(0,1fr)_auto]">
            <input
              className="h-9 rounded border border-white/10 bg-surface-950 px-2 text-xs text-surface-100 outline-none placeholder:text-surface-600 focus:border-brand-400"
              value={assignmentInput}
              onChange={(event) => setAssignmentInput(event.target.value)}
              placeholder="auditor client ids, comma separated"
              aria-label="Assigned auditor client ids"
            />
            <input
              className="h-9 rounded border border-white/10 bg-surface-950 px-2 text-xs text-surface-100 outline-none placeholder:text-surface-600 focus:border-brand-400"
              value={assignmentNotesSafe}
              onChange={(event) => setAssignmentNotesSafe(event.target.value)}
              maxLength={1000}
              placeholder="Safe assignment note"
              aria-label="Assignment notes"
            />
            <Button
              size="sm"
              variant="outline"
              loading={isAssignmentsSaving}
              onClick={() => void handleSaveAssignments()}
              title="Replace active Auditor assignments for this report"
            >
              <Users size={13} />
              Assign
            </Button>
          </div>
          {assignments && assignments.assignments.length > 0 && (
            <div className="mt-2 flex flex-wrap gap-2 text-[11px]">
              {assignments.assignments.map((item) => (
                <span
                  key={item.id}
                  className="rounded border border-white/8 bg-surface-950 px-2 py-1 text-surface-300"
                  title={item.assignment_notes_safe ?? undefined}
                >
                  <span className="font-mono">{item.auditor_client_id}</span> · {item.assignment_status}
                </span>
              ))}
            </div>
          )}
          <div className="mt-3 grid grid-cols-1 gap-2 md:grid-cols-[140px_minmax(0,1fr)_auto]">
            <select
              className="h-9 rounded border border-white/10 bg-surface-950 px-2 text-xs text-surface-100 outline-none focus:border-brand-400"
              value={commentSuggestion}
              onChange={(event) => setCommentSuggestion(event.target.value)}
              aria-label="Comment review status suggestion"
            >
              <option value="">No suggestion</option>
              <option value="reviewed">Reviewed</option>
              <option value="needs_changes">Needs changes</option>
              <option value="rejected">Rejected</option>
              <option value="needs_review">Needs review</option>
            </select>
            <textarea
              className="min-h-9 rounded border border-white/10 bg-surface-950 px-2 py-2 text-xs text-surface-100 outline-none placeholder:text-surface-600 focus:border-brand-400"
              value={commentBodySafe}
              onChange={(event) => setCommentBodySafe(event.target.value)}
              maxLength={2000}
              placeholder="Safe reviewer comment"
              aria-label="Framework report comment"
            />
            <Button
              size="sm"
              variant="outline"
              loading={isCommenting}
              disabled={!commentBodySafe.trim()}
              onClick={() => void handleCreateComment()}
              title="Add a safe reviewer comment"
            >
              <MessageSquare size={13} />
              Comment
            </Button>
          </div>
          {comments && comments.comments.length > 0 && (
            <div className="mt-2 space-y-2">
              {comments.comments.map((item) => (
                <div key={item.id} className="rounded border border-white/6 bg-surface-950 p-2 text-[11px]">
                  <div className="flex flex-wrap gap-x-3 gap-y-1 text-surface-500">
                    <span>Reviewer: <span className="font-mono text-surface-300">{item.commenter_client_id}</span></span>
                    <span>Created: <span className="text-surface-300">{formatTs(item.created_at, displayTimezone)}</span></span>
                    {item.review_status_suggestion && <span>Suggestion: <span className="text-surface-300">{item.review_status_suggestion}</span></span>}
                  </div>
                  <p className="mt-1 leading-5 text-surface-300">{item.comment_body_safe}</p>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {reportArtifact && (
        <p className="mt-3 text-[11px] text-success-200">
          Server framework report artifact downloaded and ready for local JSON save.
        </p>
      )}

      {reportHistory && (
        <div className="mt-3 border-t border-white/8 pt-3">
          <div className="mb-2 flex items-center gap-2 text-xs font-medium text-surface-200">
            <History size={13} className="text-brand-300" />
            Framework report history
            <Badge variant="info">{reportHistory.count} loaded</Badge>
          </div>
          {reportHistory.items.length === 0 ? (
            <p className="text-[11px] text-surface-500">No framework review reports found for the current filter.</p>
          ) : (
            <div className="grid grid-cols-1 gap-2">
              {reportHistory.items.map((item) => (
                <div key={item.report_id} className="rounded border border-white/6 bg-white/[0.03] p-2">
                  <div className="flex flex-wrap items-start justify-between gap-2">
                    <div className="min-w-0">
                      <div className="truncate font-mono text-[11px] text-surface-100" title={item.report_id}>
                        {item.report_id}
                      </div>
                      <div className="mt-1 flex flex-wrap gap-x-3 gap-y-1 text-[11px] text-surface-500">
                        <span>Framework: <span className="text-surface-300">{item.framework_id}</span></span>
                        <span>Owner: <span className="text-surface-300">{item.framework_owner_type}</span></span>
                        <span>Review: <span className="text-surface-300">{item.review_status}</span></span>
                        <span>Created: <span className="text-surface-300">{formatTs(item.created_at, displayTimezone)}</span></span>
                        <span>Downloaded: <span className="text-surface-300">{item.downloaded_at ? formatTs(item.downloaded_at, displayTimezone) : 'not yet'}</span></span>
                      </div>
                      <div className="mt-1 truncate text-[11px] text-surface-500" title={item.artifact_hash}>
                        Report hash: <span className="font-mono text-surface-300">{shortHash(item.artifact_hash)}</span>
                      </div>
                    </div>
                    <Button
                      size="sm"
                      variant="outline"
                      loading={isDownloading}
                      onClick={() => void handleDownloadHistory(item.report_id)}
                      title="Download this framework review report JSON"
                    >
                      <Download size={13} />
                      Save
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {assignedReports && (
        <div className="mt-3 border-t border-white/8 pt-3">
          <div className="mb-2 flex items-center gap-2 text-xs font-medium text-surface-200">
            <Users size={13} className="text-brand-300" />
            Assigned to me
            <Badge variant="info">{assignedReports.count} loaded</Badge>
          </div>
          {assignedReports.items.length === 0 ? (
            <p className="text-[11px] text-surface-500">No assigned framework review reports found for the current filter.</p>
          ) : (
            <div className="grid grid-cols-1 gap-2">
              {assignedReports.items.map((item) => (
                <div key={item.report_id} className="rounded border border-white/6 bg-white/[0.03] p-2">
                  <div className="truncate font-mono text-[11px] text-surface-100" title={item.report_id}>
                    {item.report_id}
                  </div>
                  <div className="mt-1 flex flex-wrap gap-x-3 gap-y-1 text-[11px] text-surface-500">
                    <span>Framework: <span className="text-surface-300">{item.framework_id}</span></span>
                    <span>Review: <span className="text-surface-300">{item.review_status}</span></span>
                    <span>Created: <span className="text-surface-300">{formatTs(item.created_at, displayTimezone)}</span></span>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  )
}
