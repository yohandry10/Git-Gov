import { Download, ShieldCheck } from 'lucide-react'
import { Badge } from '@/components/shared/Badge'
import { Button } from '@/components/shared/Button'
import { formatTs } from '@/lib/timezone'
import { useControlPlaneStore } from '@/store/useControlPlaneStore'

function shortHash(value?: string | null): string {
  if (!value) return 'not available'
  return value.length > 16 ? value.slice(0, 16) : value
}

function safeDownloadName(value: string): string {
  return value.trim().replace(/[^A-Za-z0-9._-]/g, '_').slice(0, 80) || 'framework-review-report-manifest'
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

export function ComplianceFrameworkReviewReportProvenancePanel() {
  const report = useControlPlaneStore((state) => state.complianceFrameworkReviewReport?.report ?? null)
  const manifest = useControlPlaneStore((state) => state.complianceFrameworkReviewReportProvenanceManifest)
  const isCreating = useControlPlaneStore((state) => state.isComplianceFrameworkReviewReportProvenanceManifestCreating)
  const displayTimezone = useControlPlaneStore((state) => state.displayTimezone)
  const createManifest = useControlPlaneStore((state) => state.createComplianceFrameworkReviewReportProvenanceManifest)

  if (!report) return null

  const canCreate = report.review_status === 'reviewed'

  const handleCreate = async () => {
    const response = await createManifest(report.report_id)
    if (response) {
      downloadJson(`gitgov-framework-review-manifest-${safeDownloadName(response.manifest.manifest_id)}.json`, response.artifact)
    }
  }

  return (
    <div className="mt-3 rounded border border-white/8 bg-white/[0.03] p-2">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="flex items-center gap-2 text-xs font-medium text-surface-200">
          <ShieldCheck size={13} className="text-brand-300" />
          Reviewed provenance manifest
          <Badge variant={canCreate ? 'success' : 'neutral'}>{canCreate ? 'reviewed' : 'blocked until reviewed'}</Badge>
          {manifest && <Badge variant="info">{shortHash(manifest.manifest.manifest_hash)}</Badge>}
        </div>
        <Button
          size="sm"
          variant="outline"
          loading={isCreating}
          disabled={!canCreate}
          onClick={() => void handleCreate()}
          title="Generate a provenance manifest for this reviewed framework report"
        >
          <Download size={13} />
          Manifest
        </Button>
      </div>
      <p className="mt-2 text-[11px] leading-5 text-surface-500">
        Generates an append-only hash-chain manifest for the reviewed report, source hashes, reviewer provenance, assignments, and comments. It does not change the report artifact or create compliance, regulatory, or certification claims.
      </p>
      {manifest && (
        <div className="mt-2 grid grid-cols-1 gap-2 text-[11px] md:grid-cols-3">
          <div className="rounded border border-white/6 bg-surface-950 p-2">
            <div className="text-surface-500">Manifest</div>
            <div className="mt-1 truncate font-mono text-surface-200" title={manifest.manifest.manifest_id}>{manifest.manifest.manifest_id}</div>
          </div>
          <div className="rounded border border-white/6 bg-surface-950 p-2">
            <div className="text-surface-500">Hash chain</div>
            <div className="mt-1 truncate font-mono text-surface-200" title={manifest.manifest.manifest_hash}>{shortHash(manifest.manifest.manifest_hash)}</div>
            <div className="mt-1 truncate text-surface-500" title={manifest.manifest.previous_manifest_hash ?? undefined}>
              Previous: <span className="font-mono text-surface-300">{shortHash(manifest.manifest.previous_manifest_hash)}</span>
            </div>
          </div>
          <div className="rounded border border-white/6 bg-surface-950 p-2">
            <div className="text-surface-500">Signed by</div>
            <div className="mt-1 truncate font-mono text-surface-200">{manifest.manifest.generated_by_user_id}</div>
            <div className="mt-1 text-surface-500">{formatTs(manifest.manifest.created_at, displayTimezone)}</div>
          </div>
        </div>
      )}
    </div>
  )
}
