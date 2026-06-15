import { ArchiveX, Download, PackageCheck, RefreshCw } from 'lucide-react'
import { Badge } from '@/components/shared/Badge'
import { Button } from '@/components/shared/Button'
import { formatTs } from '@/lib/timezone'
import { useControlPlaneStore } from '@/store/useControlPlaneStore'
import type {
  CompliancePeriodReportRecord,
  CompliancePeriodReportSharePackageRecord,
} from '@/store/useControlPlaneStore'

const NO_CLAIMS_NOTICE = 'This package organizes existing GitGov evidence for manual auditor/customer review. It is not a certification, legal attestation, compliance score, or official regulatory claim.'

function shortHash(value?: string | null): string {
  if (!value) return 'not available'
  return value.length > 16 ? value.slice(0, 16) : value
}

function safeDownloadName(value: string): string {
  return value.trim().replace(/[^A-Za-z0-9._-]/g, '_').slice(0, 80) || 'period-compliance-share-package'
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

function statusBadgeVariant(status: string): 'success' | 'warning' | 'danger' | 'neutral' {
  if (status === 'active') return 'success'
  if (status === 'revoked') return 'neutral'
  return 'warning'
}

function claimFlag(claims: Record<string, unknown>, key: string): string {
  const value = claims[key]
  return typeof value === 'boolean' ? String(value) : 'unknown'
}

function packageTitle(item: CompliancePeriodReportSharePackageRecord): string {
  return [
    item.share_package_id,
    item.artifact_hash,
    item.period_report_artifact_hash,
    item.pdf_artifact_hash,
    item.manifest_hash,
  ].join('\n')
}

interface CompliancePeriodReportSharePackagePanelProps {
  periodReport: CompliancePeriodReportRecord
  displayTimezone: string
}

export function CompliancePeriodReportSharePackagePanel({
  periodReport,
  displayTimezone,
}: CompliancePeriodReportSharePackagePanelProps) {
  const packagesResponse = useControlPlaneStore((state) => state.compliancePeriodReportSharePackages)
  const currentPackage = useControlPlaneStore((state) => state.compliancePeriodReportSharePackage?.share_package ?? null)
  const isCreating = useControlPlaneStore((state) => state.isCompliancePeriodReportSharePackageCreating)
  const isLoading = useControlPlaneStore((state) => state.isCompliancePeriodReportSharePackagesLoading)
  const isDownloading = useControlPlaneStore((state) => state.isCompliancePeriodReportSharePackageDownloading)
  const isRevoking = useControlPlaneStore((state) => state.isCompliancePeriodReportSharePackageRevoking)
  const createSharePackage = useControlPlaneStore((state) => state.createCompliancePeriodReportSharePackage)
  const loadSharePackages = useControlPlaneStore((state) => state.loadCompliancePeriodReportSharePackages)
  const downloadSharePackage = useControlPlaneStore((state) => state.downloadCompliancePeriodReportSharePackage)
  const revokeSharePackage = useControlPlaneStore((state) => state.revokeCompliancePeriodReportSharePackage)

  const packages = packagesResponse?.items.filter((item) => item.period_report_id === periodReport.period_report_id) ?? []
  const visiblePackages = packages.length > 0
    ? packages
    : currentPackage?.period_report_id === periodReport.period_report_id
      ? [currentPackage]
      : []
  const canCreate = periodReport.review_status === 'reviewed' && periodReport.retention_status !== 'archived'

  const handleCreate = async () => {
    await createSharePackage(periodReport.period_report_id)
  }

  const handleLoad = async () => {
    await loadSharePackages(periodReport.period_report_id, { limit: 25 })
  }

  const handleDownload = async (sharePackageId: string) => {
    const artifact = await downloadSharePackage(sharePackageId)
    if (artifact) {
      downloadJson(`gitgov-period-compliance-share-package-${safeDownloadName(sharePackageId)}.json`, artifact)
    }
  }

  const handleRevoke = async (sharePackageId: string) => {
    await revokeSharePackage(sharePackageId)
  }

  return (
    <div className="mt-2 rounded border border-sky-400/20 bg-sky-400/5 p-2 text-[11px]">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="flex items-center gap-2 text-surface-300">
          <PackageCheck size={13} className="text-sky-300" />
          Share packages
          <Badge variant={canCreate ? 'info' : 'neutral'}>{canCreate ? 'manual-ready' : 'needs prerequisites'}</Badge>
          {visiblePackages.length > 0 && <Badge variant="success">{visiblePackages.length} package{visiblePackages.length === 1 ? '' : 's'}</Badge>}
        </div>
        <div className="flex flex-wrap gap-2">
          <Button
            size="sm"
            variant="outline"
            loading={isLoading}
            onClick={() => void handleLoad()}
            title="Load share packages for this reviewed period report"
          >
            <RefreshCw size={13} />
            Packages
          </Button>
          <Button
            size="sm"
            variant="outline"
            loading={isCreating}
            disabled={!canCreate}
            onClick={() => void handleCreate()}
            title="Create a manual offline verification package from reviewed JSON, PDF, and provenance manifest artifacts"
          >
            <PackageCheck size={13} />
            Create
          </Button>
        </div>
      </div>

      <p className="mt-2 leading-5 text-surface-500">{NO_CLAIMS_NOTICE}</p>

      {!canCreate && (
        <div className="mt-2 rounded border border-white/6 bg-surface-950 p-2 text-surface-500">
          Creation requires a reviewed, non-archived period report with existing JSON, PDF export, and provenance manifest artifacts.
        </div>
      )}

      {visiblePackages.length > 0 && (
        <div className="mt-2 space-y-2">
          {visiblePackages.map((item) => (
            <div key={item.share_package_id} className="rounded border border-white/6 bg-surface-950 p-2" title={packageTitle(item)}>
              <div className="flex flex-wrap items-center justify-between gap-2">
                <div className="min-w-0">
                  <div className="flex flex-wrap items-center gap-2">
                    <span className="truncate font-mono text-surface-200">{item.share_package_id}</span>
                    <Badge variant={statusBadgeVariant(item.status)}>{item.status}</Badge>
                  </div>
                  <div className="mt-1 flex flex-wrap gap-x-3 gap-y-1 text-surface-500">
                    <span>Package: <span className="font-mono">{shortHash(item.artifact_hash)}</span></span>
                    <span>JSON: <span className="font-mono">{shortHash(item.period_report_artifact_hash)}</span></span>
                    <span>PDF: <span className="font-mono">{shortHash(item.pdf_artifact_hash)}</span></span>
                    <span>Manifest: <span className="font-mono">{shortHash(item.manifest_hash)}</span></span>
                  </div>
                  <div className="mt-1 flex flex-wrap gap-x-3 gap-y-1 text-surface-500">
                    <span>{item.download_count} downloads</span>
                    <span>Created {formatTs(item.created_at, displayTimezone)}</span>
                    <span>Last {item.last_downloaded_at ? formatTs(item.last_downloaded_at, displayTimezone) : 'not downloaded'}</span>
                    <span>Claims c:{claimFlag(item.no_claims_snapshot, 'certification')} r:{claimFlag(item.no_claims_snapshot, 'regulatory_claim')} score:{claimFlag(item.no_claims_snapshot, 'compliance_score')}</span>
                  </div>
                </div>
                <div className="flex flex-wrap gap-2">
                  <Button
                    size="sm"
                    variant="ghost"
                    loading={isDownloading}
                    disabled={item.status === 'revoked'}
                    onClick={() => void handleDownload(item.share_package_id)}
                    title="Download the offline share package JSON bundle"
                  >
                    <Download size={13} />
                    JSON
                  </Button>
                  <Button
                    size="sm"
                    variant="ghost"
                    loading={isRevoking}
                    disabled={item.status === 'revoked'}
                    onClick={() => void handleRevoke(item.share_package_id)}
                    title="Revoke future downloads for this share package"
                  >
                    <ArchiveX size={13} />
                    Revoke
                  </Button>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
