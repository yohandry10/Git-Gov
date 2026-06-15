import { Download, FileText } from 'lucide-react'
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

function downloadPdf(filename: string, pdfBase64: string) {
  const binary = atob(pdfBase64)
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

export function ComplianceFrameworkReviewReportPdfExportPanel() {
  const report = useControlPlaneStore((state) => state.complianceFrameworkReviewReport?.report ?? null)
  const manifest = useControlPlaneStore((state) => state.complianceFrameworkReviewReportProvenanceManifest?.manifest ?? null)
  const pdfExport = useControlPlaneStore((state) => state.complianceFrameworkReviewReportPdfExport?.pdf_export ?? null)
  const isCreating = useControlPlaneStore((state) => state.isComplianceFrameworkReviewReportPdfExportCreating)
  const isDownloading = useControlPlaneStore((state) => state.isComplianceFrameworkReviewReportPdfExportDownloading)
  const displayTimezone = useControlPlaneStore((state) => state.displayTimezone)
  const createPdfExport = useControlPlaneStore((state) => state.createComplianceFrameworkReviewReportPdfExport)
  const downloadPdfExport = useControlPlaneStore((state) => state.downloadComplianceFrameworkReviewReportPdfExport)

  if (!report) return null

  const canCreate = report.review_status === 'reviewed'

  const handleCreate = async () => {
    await createPdfExport(report.report_id, manifest?.manifest_id ?? null)
  }

  const handleDownload = async () => {
    const response = await downloadPdfExport(report.report_id, pdfExport?.pdf_export_id ?? null)
    if (response) {
      downloadPdf(`gitgov-framework-review-${safeDownloadName(response.pdf_export.pdf_export_id)}.pdf`, response.pdf_base64)
    }
  }

  return (
    <div className="mt-3 rounded border border-white/8 bg-white/[0.03] p-2">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="flex items-center gap-2 text-xs font-medium text-surface-200">
          <FileText size={13} className="text-brand-300" />
          Framework review PDF
          <Badge variant={canCreate ? 'success' : 'neutral'}>{canCreate ? 'reviewed' : 'blocked until reviewed'}</Badge>
          {pdfExport && <Badge variant="info">{shortHash(pdfExport.pdf_artifact_hash)}</Badge>}
        </div>
        <div className="flex gap-2">
          <Button
            size="sm"
            variant="outline"
            loading={isCreating}
            disabled={!canCreate}
            onClick={() => void handleCreate()}
            title="Generate a PDF export for this reviewed framework report"
          >
            <FileText size={13} />
            PDF
          </Button>
          <Button
            size="sm"
            variant="outline"
            loading={isDownloading}
            disabled={!pdfExport}
            onClick={() => void handleDownload()}
            title="Download the generated PDF export"
          >
            <Download size={13} />
            Download
          </Button>
        </div>
      </div>
      <p className="mt-2 text-[11px] leading-5 text-surface-500">
        Generates a readable PDF bound to the reviewed report and provenance manifest. It preserves the JSON artifacts as the canonical evidence and does not create certification, compliance, or regulatory claims.
      </p>
      {pdfExport && (
        <div className="mt-2 grid grid-cols-1 gap-2 text-[11px] md:grid-cols-3">
          <div className="rounded border border-white/6 bg-surface-950 p-2">
            <div className="text-surface-500">PDF export</div>
            <div className="mt-1 truncate font-mono text-surface-200" title={pdfExport.pdf_export_id}>{pdfExport.pdf_export_id}</div>
            <div className="mt-1 text-surface-500">{pdfExport.page_count} page{pdfExport.page_count === 1 ? '' : 's'}</div>
          </div>
          <div className="rounded border border-white/6 bg-surface-950 p-2">
            <div className="text-surface-500">Hash</div>
            <div className="mt-1 truncate font-mono text-surface-200" title={pdfExport.pdf_artifact_hash}>{shortHash(pdfExport.pdf_artifact_hash)}</div>
            <div className="mt-1 truncate text-surface-500" title={pdfExport.manifest_hash}>
              Manifest: <span className="font-mono text-surface-300">{shortHash(pdfExport.manifest_hash)}</span>
            </div>
          </div>
          <div className="rounded border border-white/6 bg-surface-950 p-2">
            <div className="text-surface-500">Generated</div>
            <div className="mt-1 truncate font-mono text-surface-200">{pdfExport.created_by_user_id}</div>
            <div className="mt-1 text-surface-500">{formatTs(pdfExport.created_at, displayTimezone)}</div>
          </div>
        </div>
      )}
    </div>
  )
}
