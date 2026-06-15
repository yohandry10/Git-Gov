import { Archive, CalendarDays, Clock3, Download, FileJson, FileText, History, RefreshCw } from 'lucide-react'
import { useState } from 'react'
import { Badge } from '@/components/shared/Badge'
import { Button } from '@/components/shared/Button'
import { formatTs } from '@/lib/timezone'
import { useControlPlaneStore } from '@/store/useControlPlaneStore'
import { CompliancePeriodReportProvenancePanel } from './CompliancePeriodReportProvenancePanel'

function shortHash(value?: string | null): string {
  if (!value) return 'not available'
  return value.length > 16 ? value.slice(0, 16) : value
}

function safeDownloadName(value: string): string {
  return value.trim().replace(/[^A-Za-z0-9._-]/g, '_').slice(0, 80) || 'period-compliance-report'
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

function downloadPdf(filename: string, base64: string) {
  const binary = atob(base64)
  const bytes = new Uint8Array(binary.length)
  for (let index = 0; index < binary.length; index += 1) {
    bytes[index] = binary.charCodeAt(index)
  }
  const blob = new Blob([bytes], { type: 'application/pdf' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = filename
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
}

function dateInputValue(timestamp: number): string {
  return new Date(timestamp).toISOString().slice(0, 10)
}

function parseDateStart(value: string): number {
  return new Date(`${value}T00:00:00.000Z`).getTime()
}

function parseDateEnd(value: string): number {
  return new Date(`${value}T00:00:00.000Z`).getTime()
}

function defaultPeriodStart(): number {
  const now = new Date()
  return Date.UTC(now.getUTCFullYear(), now.getUTCMonth(), 1)
}

function defaultPeriodEnd(): number {
  const now = new Date()
  return Date.UTC(now.getUTCFullYear(), now.getUTCMonth() + 1, 1)
}

function summaryNumber(artifact: Record<string, unknown> | null, key: string): number | null {
  const summary = artifact?.summary
  if (!summary || typeof summary !== 'object') return null
  const value = (summary as Record<string, unknown>)[key]
  return typeof value === 'number' ? value : null
}

function retentionBadgeVariant(status: string): 'success' | 'warning' | 'danger' | 'neutral' {
  if (status === 'active') return 'success'
  if (status === 'retention_expired') return 'warning'
  if (status === 'archived') return 'neutral'
  return 'danger'
}

function formatOptionalTs(timestamp: number | null | undefined, timezone: string): string {
  return timestamp ? formatTs(timestamp, timezone) : 'not recorded'
}

function oneYearFromNow(): number {
  return Date.now() + 365 * 24 * 60 * 60 * 1000
}

export function CompliancePeriodReportPanel() {
  const [dateRangeStart, setDateRangeStart] = useState(defaultPeriodStart)
  const [dateRangeEnd, setDateRangeEnd] = useState(defaultPeriodEnd)
  const selectedFrameworkId = useControlPlaneStore((state) => state.selectedComplianceFrameworkId)
  const periodReport = useControlPlaneStore((state) => state.compliancePeriodReport?.period_report ?? null)
  const periodReports = useControlPlaneStore((state) => state.compliancePeriodReports)
  const periodArtifact = useControlPlaneStore((state) => state.compliancePeriodReportArtifact)
  const periodAccessLog = useControlPlaneStore((state) => state.compliancePeriodReportAccessLog)
  const periodPdfExport = useControlPlaneStore((state) => state.compliancePeriodReportPdfExport?.pdf_export ?? null)
  const isCreating = useControlPlaneStore((state) => state.isCompliancePeriodReportCreating)
  const isLoading = useControlPlaneStore((state) => state.isCompliancePeriodReportsLoading)
  const isDownloading = useControlPlaneStore((state) => state.isCompliancePeriodReportDownloading)
  const isRetentionUpdating = useControlPlaneStore((state) => state.isCompliancePeriodReportRetentionUpdating)
  const isAccessLogLoading = useControlPlaneStore((state) => state.isCompliancePeriodReportAccessLogLoading)
  const isCreatingPdf = useControlPlaneStore((state) => state.isCompliancePeriodReportPdfExportCreating)
  const isDownloadingPdf = useControlPlaneStore((state) => state.isCompliancePeriodReportPdfExportDownloading)
  const displayTimezone = useControlPlaneStore((state) => state.displayTimezone)
  const userRole = useControlPlaneStore((state) => state.userRole)
  const createPeriodReport = useControlPlaneStore((state) => state.createCompliancePeriodReport)
  const loadPeriodReports = useControlPlaneStore((state) => state.loadCompliancePeriodReports)
  const downloadPeriodReport = useControlPlaneStore((state) => state.downloadCompliancePeriodReport)
  const updatePeriodReportRetention = useControlPlaneStore((state) => state.updateCompliancePeriodReportRetention)
  const loadPeriodReportAccessLog = useControlPlaneStore((state) => state.loadCompliancePeriodReportAccessLog)
  const createPeriodReportPdf = useControlPlaneStore((state) => state.createCompliancePeriodReportPdfExport)
  const downloadPeriodReportPdf = useControlPlaneStore((state) => state.downloadCompliancePeriodReportPdfExport)

  const isAdmin = userRole === 'Admin'
  const canGenerate = dateRangeStart > 0 && dateRangeEnd > dateRangeStart
  const reportCount = summaryNumber(periodArtifact, 'report_count') ?? periodReport?.report_count ?? 0
  const missingCount = summaryNumber(periodArtifact, 'missing_evidence_type_count')

  const handleGenerate = async () => {
    await createPeriodReport(dateRangeStart, dateRangeEnd, selectedFrameworkId || null)
  }

  const handleLoad = async () => {
    await loadPeriodReports({
      framework_id: selectedFrameworkId || null,
      limit: 25,
    })
  }

  const handleDownload = async (periodReportId: string) => {
    const artifact = await downloadPeriodReport(periodReportId)
    if (artifact) {
      downloadJson(`gitgov-period-compliance-${safeDownloadName(periodReportId)}.json`, artifact)
    }
  }

  const handleCreatePdf = async (periodReportId: string) => {
    await createPeriodReportPdf(periodReportId)
  }

  const handleExtendRetention = async (periodReportId: string) => {
    await updatePeriodReportRetention(periodReportId, {
      retention_until: oneYearFromNow(),
      archive: false,
    })
    await loadPeriodReportAccessLog(periodReportId, { limit: 10 })
  }

  const handleArchive = async (periodReportId: string) => {
    await updatePeriodReportRetention(periodReportId, { archive: true })
    await loadPeriodReportAccessLog(periodReportId, { limit: 10 })
  }

  const handleLoadAccessLog = async (periodReportId: string) => {
    await loadPeriodReportAccessLog(periodReportId, { limit: 10 })
  }

  const handleDownloadPdf = async (periodReportId: string, pdfExportId?: string | null) => {
    const response = await downloadPeriodReportPdf(periodReportId, pdfExportId)
    if (response) {
      downloadPdf(`gitgov-period-compliance-${safeDownloadName(response.pdf_export.pdf_export_id)}.pdf`, response.pdf_base64)
    }
  }

  return (
    <div className="mt-3 rounded border border-white/8 bg-white/[0.03] p-2">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="flex items-center gap-2 text-xs font-medium text-surface-200">
          <FileJson size={13} className="text-brand-300" />
          Period compliance report
          <Badge variant="info">manual JSON/PDF</Badge>
          {periodReport && <Badge variant="success">{periodReport.report_count} reports</Badge>}
          {periodReport && (
            <Badge variant={retentionBadgeVariant(periodReport.retention_status)}>
              {periodReport.retention_status.replace(/_/g, ' ')}
            </Badge>
          )}
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <label className="flex items-center gap-1 text-[11px] text-surface-500">
            <CalendarDays size={12} />
            <input
              type="date"
              className="h-8 rounded border border-white/10 bg-surface-950 px-2 text-xs text-surface-100 outline-none focus:border-brand-400"
              value={dateInputValue(dateRangeStart)}
              onChange={(event) => setDateRangeStart(parseDateStart(event.target.value))}
              aria-label="Period report start date"
            />
          </label>
          <input
            type="date"
            className="h-8 rounded border border-white/10 bg-surface-950 px-2 text-xs text-surface-100 outline-none focus:border-brand-400"
            value={dateInputValue(dateRangeEnd)}
            onChange={(event) => setDateRangeEnd(parseDateEnd(event.target.value))}
            aria-label="Period report end date"
          />
          <Button
            size="sm"
            variant="outline"
            loading={isCreating}
            disabled={!canGenerate}
            onClick={() => void handleGenerate()}
            title="Generate a manual period compliance report JSON"
          >
            <FileJson size={13} />
            Generate
          </Button>
          <Button
            size="sm"
            variant="outline"
            loading={isLoading}
            onClick={() => void handleLoad()}
            title="Load recent period compliance reports"
          >
            <RefreshCw size={13} />
            History
          </Button>
        </div>
      </div>

      <p className="mt-2 text-[11px] leading-5 text-surface-500">
        Summarizes reviewed Framework Review Reports inside the selected date range. The artifact includes source hashes, manifest hashes when present, missing evidence, and no certification or regulatory claim.
      </p>

      {periodReport && (
        <div className="mt-2 grid grid-cols-1 gap-2 text-[11px] md:grid-cols-4">
          <div className="rounded border border-white/6 bg-surface-950 p-2">
            <div className="text-surface-500">Period report</div>
            <div className="mt-1 truncate font-mono text-surface-200" title={periodReport.period_report_id}>{periodReport.period_report_id}</div>
            <div className="mt-1 text-surface-500">{formatTs(periodReport.created_at, displayTimezone)}</div>
          </div>
          <div className="rounded border border-white/6 bg-surface-950 p-2">
            <div className="text-surface-500">Hash</div>
            <div className="mt-1 truncate font-mono text-surface-200" title={periodReport.artifact_hash}>{shortHash(periodReport.artifact_hash)}</div>
            <div className="mt-1 text-surface-500">{periodReport.status}</div>
          </div>
          <div className="rounded border border-white/6 bg-surface-950 p-2">
            <div className="text-surface-500">Window</div>
            <div className="mt-1 text-surface-200">{formatTs(periodReport.date_range_start, displayTimezone)}</div>
            <div className="mt-1 text-surface-500">{formatTs(periodReport.date_range_end, displayTimezone)}</div>
          </div>
          <div className="rounded border border-white/6 bg-surface-950 p-2">
            <div className="text-surface-500">Evidence</div>
            <div className="mt-1 text-surface-200">{reportCount} reviewed reports</div>
            <div className="mt-1 text-surface-500">{missingCount ?? 'unknown'} missing evidence types</div>
          </div>
          <div className="rounded border border-white/6 bg-surface-950 p-2 md:col-span-2">
            <div className="flex items-center gap-1 text-surface-500">
              <Clock3 size={12} />
              Retention
            </div>
            <div className="mt-1 flex flex-wrap items-center gap-2">
              <Badge variant={retentionBadgeVariant(periodReport.retention_status)}>
                {periodReport.retention_status.replace(/_/g, ' ')}
              </Badge>
              <span className="text-surface-200">{formatOptionalTs(periodReport.retention_until, displayTimezone)}</span>
            </div>
            <div className="mt-1 text-surface-500">
              Archived: {formatOptionalTs(periodReport.archived_at, displayTimezone)}
            </div>
          </div>
          <div className="rounded border border-white/6 bg-surface-950 p-2 md:col-span-2">
            <div className="flex items-center gap-1 text-surface-500">
              <History size={12} />
              Export custody
            </div>
            <div className="mt-1 text-surface-200">{periodReport.download_count} downloads</div>
            <div className="mt-1 text-surface-500">
              Last download: {formatOptionalTs(periodReport.last_downloaded_at, displayTimezone)}
            </div>
          </div>
        </div>
      )}

      {periodReport && (
        <div className="mt-2 flex flex-wrap items-center justify-between gap-2 rounded border border-white/6 bg-surface-950 p-2 text-[11px]">
          <div className="min-w-0 truncate text-surface-500" title={periodReport.source_report_ids.join(', ')}>
            Sources: <span className="font-mono text-surface-300">{periodReport.source_report_ids.slice(0, 4).join(', ')}</span>
            {periodReport.source_report_ids.length > 4 ? ` +${periodReport.source_report_ids.length - 4}` : ''}
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <Button
              size="sm"
              variant="outline"
              loading={isAccessLogLoading}
              onClick={() => void handleLoadAccessLog(periodReport.period_report_id)}
              title="Load append-only access log for this period compliance report"
            >
              <History size={13} />
              Log
            </Button>
            {isAdmin && (
              <Button
                size="sm"
                variant="outline"
                loading={isRetentionUpdating}
                onClick={() => void handleExtendRetention(periodReport.period_report_id)}
                title="Extend retention one year from now"
              >
                <Clock3 size={13} />
                Extend
              </Button>
            )}
            {isAdmin && (
              <Button
                size="sm"
                variant="outline"
                loading={isRetentionUpdating}
                onClick={() => void handleArchive(periodReport.period_report_id)}
                title="Archive this period compliance report without deleting the artifact"
              >
                <Archive size={13} />
                Archive
              </Button>
            )}
            <Button
              size="sm"
              variant="outline"
              loading={isCreatingPdf}
              onClick={() => void handleCreatePdf(periodReport.period_report_id)}
              title="Generate a manual PDF export from this period compliance report"
            >
              <FileText size={13} />
              PDF
            </Button>
            <Button
              size="sm"
              variant="outline"
              loading={isDownloading}
              onClick={() => void handleDownload(periodReport.period_report_id)}
              title="Download this period compliance report JSON"
            >
              <Download size={13} />
              JSON
            </Button>
          </div>
        </div>
      )}

      {periodPdfExport && periodPdfExport.period_report_id === periodReport?.period_report_id && (
        <div className="mt-2 flex flex-wrap items-center justify-between gap-2 rounded border border-brand-400/20 bg-brand-400/5 p-2 text-[11px]">
          <div className="min-w-0">
            <div className="flex flex-wrap items-center gap-2 text-surface-300">
              <FileText size={13} className="text-brand-300" />
              Period PDF export
              <Badge variant="success">{periodPdfExport.page_count} page{periodPdfExport.page_count === 1 ? '' : 's'}</Badge>
            </div>
            <div className="mt-1 truncate font-mono text-surface-500" title={periodPdfExport.pdf_artifact_hash}>
              {shortHash(periodPdfExport.pdf_artifact_hash)}
            </div>
          </div>
          <Button
            size="sm"
            variant="outline"
            loading={isDownloadingPdf}
            onClick={() => void handleDownloadPdf(periodPdfExport.period_report_id, periodPdfExport.pdf_export_id)}
            title="Download the generated period compliance PDF"
          >
            <Download size={13} />
            PDF
          </Button>
        </div>
      )}

      {periodReport && (
        <CompliancePeriodReportProvenancePanel
          periodReport={periodReport}
          displayTimezone={displayTimezone}
        />
      )}

      {periodAccessLog && periodAccessLog.items.length > 0 && (
        <div className="mt-2 rounded border border-white/6 bg-surface-950 p-2 text-[11px]">
          <div className="flex items-center gap-2 text-surface-300">
            <History size={13} className="text-brand-300" />
            Access log
            <Badge variant="neutral">{periodAccessLog.count} events</Badge>
          </div>
          <div className="mt-2 space-y-1">
            {periodAccessLog.items.slice(0, 10).map((entry) => (
              <div key={entry.access_log_id} className="grid gap-1 rounded border border-white/6 bg-white/[0.02] p-2 md:grid-cols-[140px_1fr_160px]">
                <div className="font-mono text-surface-200">{entry.action}</div>
                <div className="min-w-0 truncate text-surface-500" title={`${entry.artifact_type} ${entry.artifact_id ?? ''} ${entry.artifact_hash ?? ''}`}>
                  {entry.artifact_type}
                  {entry.artifact_id ? ` · ${entry.artifact_id}` : ''}
                  {entry.artifact_hash ? ` · ${shortHash(entry.artifact_hash)}` : ''}
                </div>
                <div className="text-surface-500">{formatTs(entry.created_at, displayTimezone)}</div>
              </div>
            ))}
          </div>
        </div>
      )}

      {periodReports && periodReports.items.length > 0 && (
        <div className="mt-2 space-y-2">
          {periodReports.items.slice(0, 5).map((item) => (
            <div key={item.period_report_id} className="flex flex-wrap items-center justify-between gap-2 rounded border border-white/6 bg-surface-950 p-2 text-[11px]">
              <div className="min-w-0">
                <div className="truncate font-mono text-surface-200" title={item.period_report_id}>{item.period_report_id}</div>
                <div className="mt-1 flex flex-wrap gap-x-3 gap-y-1 text-surface-500">
                  <span>{item.report_count} reports</span>
                  <span>{shortHash(item.artifact_hash)}</span>
                  <span>{item.download_count} downloads</span>
                  <span>{item.retention_status.replace(/_/g, ' ')}</span>
                  <span>{formatTs(item.created_at, displayTimezone)}</span>
                  <span>{item.framework_id ?? 'all frameworks'}</span>
                </div>
              </div>
              <div className="flex flex-wrap items-center gap-2">
                <Button
                  size="sm"
                  variant="ghost"
                  loading={isAccessLogLoading}
                  onClick={() => void handleLoadAccessLog(item.period_report_id)}
                  title="Load access log for this historical period report"
                >
                  <History size={13} />
                  Log
                </Button>
                <Button
                  size="sm"
                  variant="ghost"
                  loading={isDownloadingPdf}
                  onClick={() => void handleDownloadPdf(item.period_report_id)}
                  title="Download the latest PDF export for this historical period report"
                >
                  <FileText size={13} />
                  PDF
                </Button>
                <Button
                  size="sm"
                  variant="ghost"
                  loading={isDownloading}
                  onClick={() => void handleDownload(item.period_report_id)}
                  title="Download historical period compliance report JSON"
                >
                  <Download size={13} />
                  JSON
                </Button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
