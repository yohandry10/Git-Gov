import { Download, FileCheck2 } from 'lucide-react'
import { Badge } from '@/components/shared/Badge'
import { Button } from '@/components/shared/Button'
import { formatTs } from '@/lib/timezone'
import { useControlPlaneStore } from '@/store/useControlPlaneStore'
import type { CompliancePeriodReportRecord } from '@/store/useControlPlaneStore'

function shortHash(value?: string | null): string {
  if (!value) return 'not available'
  return value.length > 16 ? value.slice(0, 16) : value
}

function safeDownloadName(value: string): string {
  return value.trim().replace(/[^A-Za-z0-9._-]/g, '_').slice(0, 80) || 'period-compliance-manifest'
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

interface CompliancePeriodReportProvenancePanelProps {
  periodReport: CompliancePeriodReportRecord
  displayTimezone: string
}

export function CompliancePeriodReportProvenancePanel({
  periodReport,
  displayTimezone,
}: CompliancePeriodReportProvenancePanelProps) {
  const manifestResponse = useControlPlaneStore((state) => state.compliancePeriodReportProvenanceManifest)
  const isCreating = useControlPlaneStore((state) => state.isCompliancePeriodReportProvenanceManifestCreating)
  const isDownloading = useControlPlaneStore((state) => state.isCompliancePeriodReportProvenanceManifestDownloading)
  const createManifest = useControlPlaneStore((state) => state.createCompliancePeriodReportProvenanceManifest)
  const downloadManifest = useControlPlaneStore((state) => state.downloadCompliancePeriodReportProvenanceManifest)

  const manifest = manifestResponse?.manifest.period_report_id === periodReport.period_report_id
    ? manifestResponse.manifest
    : null

  const handleCreate = async () => {
    await createManifest(periodReport.period_report_id)
  }

  const handleDownload = async () => {
    if (!manifest) return
    const artifact = await downloadManifest(periodReport.period_report_id, manifest.manifest_id)
    if (artifact) {
      downloadJson(`gitgov-period-compliance-manifest-${safeDownloadName(manifest.manifest_id)}.json`, artifact)
    }
  }

  return (
    <div className="mt-2 flex flex-wrap items-center justify-between gap-2 rounded border border-emerald-400/20 bg-emerald-400/5 p-2 text-[11px]">
      <div className="min-w-0">
        <div className="flex flex-wrap items-center gap-2 text-surface-300">
          <FileCheck2 size={13} className="text-emerald-300" />
          Provenance manifest
          {manifest ? <Badge variant="success">materialized</Badge> : <Badge variant="neutral">manual</Badge>}
        </div>
        {manifest ? (
          <div className="mt-1 flex flex-wrap gap-x-3 gap-y-1 text-surface-500">
            <span className="font-mono" title={manifest.manifest_hash}>{shortHash(manifest.manifest_hash)}</span>
            <span title={manifest.previous_manifest_hash ?? undefined}>
              Previous: {shortHash(manifest.previous_manifest_hash)}
            </span>
            <span>{manifest.signature_algorithm}</span>
            <span>{formatTs(manifest.created_at, displayTimezone)}</span>
          </div>
        ) : (
          <div className="mt-1 truncate text-surface-500" title={periodReport.artifact_hash}>
            Binds JSON hash, PDF exports, retention state, access log summary, and source hashes.
          </div>
        )}
      </div>
      <div className="flex flex-wrap gap-2">
        <Button
          size="sm"
          variant="outline"
          loading={isCreating}
          onClick={() => void handleCreate()}
          title="Materialize an append-only provenance manifest for this period report"
        >
          <FileCheck2 size={13} />
          Manifest
        </Button>
        {manifest && (
          <Button
            size="sm"
            variant="outline"
            loading={isDownloading}
            onClick={() => void handleDownload()}
            title="Download the period report provenance manifest JSON"
          >
            <Download size={13} />
            JSON
          </Button>
        )}
      </div>
    </div>
  )
}
