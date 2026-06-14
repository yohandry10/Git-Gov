import { Download, FileCheck2, History, RefreshCw, ShieldAlert } from 'lucide-react'
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

export function ComplianceFrameworkReviewReportPanel() {
  const mapping = useControlPlaneStore((state) => state.complianceEvidenceMapping?.mapping ?? null)
  const packageRecord = useControlPlaneStore((state) => state.complianceReviewPackage?.review_package ?? null)
  const report = useControlPlaneStore((state) => state.complianceFrameworkReviewReport?.report ?? null)
  const reportHistory = useControlPlaneStore((state) => state.complianceFrameworkReviewReports)
  const reportArtifact = useControlPlaneStore((state) => state.complianceFrameworkReviewReportArtifact)
  const isCreating = useControlPlaneStore((state) => state.isComplianceFrameworkReviewReportCreating)
  const isHistoryLoading = useControlPlaneStore((state) => state.isComplianceFrameworkReviewReportsLoading)
  const isDownloading = useControlPlaneStore((state) => state.isComplianceFrameworkReviewReportDownloading)
  const displayTimezone = useControlPlaneStore((state) => state.displayTimezone)
  const createReport = useControlPlaneStore((state) => state.createComplianceFrameworkReviewReport)
  const loadReports = useControlPlaneStore((state) => state.loadComplianceFrameworkReviewReports)
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

  const handleDownloadHistory = async (reportId: string) => {
    const artifact = await downloadReport(reportId)
    if (artifact) {
      downloadJson(`gitgov-framework-review-${safeDownloadName(reportId)}.json`, artifact)
    }
  }

  return (
    <div className="mt-4 rounded-lg border border-white/8 bg-surface-900/60 p-3">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <div className="flex items-center gap-2">
            <FileCheck2 size={14} className="text-brand-300" />
            <span className="text-xs font-medium text-surface-200">Framework Review Report</span>
            <Badge variant={report ? 'success' : 'info'}>{report ? 'report ready' : 'JSON export'}</Badge>
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
    </div>
  )
}
