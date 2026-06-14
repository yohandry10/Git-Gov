import { useEffect, useMemo, useState } from 'react'
import { AlertTriangle, Download, FileJson, Layers3, PackageCheck, ShieldCheck, Upload } from 'lucide-react'
import { Badge } from '@/components/shared/Badge'
import { Button } from '@/components/shared/Button'
import { formatTs } from '@/lib/timezone'
import { useControlPlaneStore } from '@/store/useControlPlaneStore'
import type { ComplianceEvidenceMappingItem, DeploymentGateAuthorizationRecord } from '@/store/useControlPlaneStore/types'

function shortHash(value?: string | null): string {
  if (!value) return 'not available'
  return value.length > 16 ? value.slice(0, 16) : value
}

function safeDownloadName(value: string): string {
  return value.trim().replace(/[^A-Za-z0-9._-]/g, '_').slice(0, 80) || 'review-package'
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

function missingEvidence(items: ComplianceEvidenceMappingItem[]): string[] {
  return Array.from(new Set(items.flatMap((item) => item.missing_evidence))).sort()
}

function coverageCounts(items: ComplianceEvidenceMappingItem[]) {
  return items.reduce(
    (acc, item) => {
      if (item.status === 'covered' || item.status === 'evidence_present') acc.covered += 1
      else if (item.status === 'partial') acc.partial += 1
      else acc.missing += 1
      return acc
    },
    { covered: 0, partial: 0, missing: 0 },
  )
}

function authorizationLabel(authorization: DeploymentGateAuthorizationRecord): string {
  return `${authorization.release_id} / ${authorization.environment} / ${authorization.authorization_id}`
}

export function ComplianceEvidenceFlowPanel() {
  const authorizations = useControlPlaneStore((state) => state.deploymentGateAuthorizations)
  const frameworks = useControlPlaneStore((state) => state.complianceControlFrameworks)
  const selectedFrameworkId = useControlPlaneStore((state) => state.selectedComplianceFrameworkId)
  const importResponse = useControlPlaneStore((state) => state.complianceFrameworkImportResponse)
  const selectedDeploymentGateId = useControlPlaneStore((state) => state.complianceEvidenceSelectedDeploymentGateId)
  const evidenceExport = useControlPlaneStore((state) => state.complianceEvidenceExport)
  const evidenceMapping = useControlPlaneStore((state) => state.complianceEvidenceMapping)
  const reviewPackage = useControlPlaneStore((state) => state.complianceReviewPackage)
  const reviewPackageArtifact = useControlPlaneStore((state) => state.complianceReviewPackageArtifact)
  const isExportCreating = useControlPlaneStore((state) => state.isComplianceEvidenceExportCreating)
  const isFrameworksLoading = useControlPlaneStore((state) => state.isComplianceFrameworksLoading)
  const isFrameworkImporting = useControlPlaneStore((state) => state.isComplianceFrameworkPackImporting)
  const isMappingCreating = useControlPlaneStore((state) => state.isComplianceEvidenceMappingCreating)
  const isPackageCreating = useControlPlaneStore((state) => state.isComplianceReviewPackageCreating)
  const isDownloading = useControlPlaneStore((state) => state.isComplianceReviewPackageDownloading)
  const error = useControlPlaneStore((state) => state.complianceEvidenceError)
  const displayTimezone = useControlPlaneStore((state) => state.displayTimezone)
  const loadFrameworks = useControlPlaneStore((state) => state.loadComplianceFrameworks)
  const importFrameworkPack = useControlPlaneStore((state) => state.importComplianceFrameworkPack)
  const selectFramework = useControlPlaneStore((state) => state.selectComplianceFramework)
  const createExport = useControlPlaneStore((state) => state.createComplianceEvidenceExport)
  const createMapping = useControlPlaneStore((state) => state.createComplianceEvidenceMapping)
  const createPackage = useControlPlaneStore((state) => state.createComplianceReviewPackage)
  const downloadPackage = useControlPlaneStore((state) => state.downloadComplianceReviewPackage)
  const resetFlow = useControlPlaneStore((state) => state.resetComplianceEvidenceFlow)

  const defaultAuthorizationId = authorizations[0]?.authorization_id ?? ''
  const [draftAuthorizationId, setDraftAuthorizationId] = useState(
    selectedDeploymentGateId || '',
  )
  const [packFormat, setPackFormat] = useState<'json' | 'yaml'>('json')
  const [packContent, setPackContent] = useState('')
  const effectiveAuthorizationId = draftAuthorizationId || defaultAuthorizationId
  const selectedFramework = frameworks.find((framework) => framework.framework_id === selectedFrameworkId) ?? frameworks[0] ?? null

  useEffect(() => {
    void loadFrameworks()
  }, [loadFrameworks])

  const selectedAuthorization = useMemo(
    () => authorizations.find((item) => item.authorization_id === effectiveAuthorizationId) ?? null,
    [authorizations, effectiveAuthorizationId],
  )
  const mappingItems = evidenceMapping?.items ?? []
  const counts = coverageCounts(mappingItems)
  const missing = missingEvidence(mappingItems)
  const packageRecord = reviewPackage?.review_package ?? null
  const exportRecord = evidenceExport?.export ?? null
  const mappingRecord = evidenceMapping?.mapping ?? null

  const handleGenerateExport = async () => {
    const response = await createExport(effectiveAuthorizationId)
    if (response) {
      setDraftAuthorizationId(response.export.deployment_gate_id || effectiveAuthorizationId)
    }
  }

  const handleGenerateMapping = async () => {
    if (!exportRecord) return
    await createMapping(exportRecord.export_id, selectedFramework?.framework_id)
  }

  const handleImportFramework = async () => {
    const response = await importFrameworkPack(packContent, packFormat)
    if (response) {
      setPackContent('')
    }
  }

  const handleGeneratePackage = async () => {
    if (!mappingRecord) return
    await createPackage(mappingRecord.mapping_id)
  }

  const handleDownloadPackage = async () => {
    if (!packageRecord) return
    const artifact = await downloadPackage(packageRecord.review_package_id)
    if (artifact) {
      downloadJson(
        `gitgov-control-review-${safeDownloadName(packageRecord.review_package_id)}.json`,
        artifact,
      )
    }
  }

  const canGenerateExport = Boolean(effectiveAuthorizationId)
  const canGenerateMapping = Boolean(exportRecord)
  const canGeneratePackage = Boolean(mappingRecord)
  const canDownloadPackage = Boolean(packageRecord)

  return (
    <section id="compliance-evidence-flow" className="glass-panel p-5 scroll-mt-4">
      <div className="card-header mb-4">
        <div>
          <div className="flex items-center gap-2">
            <PackageCheck size={16} className="text-brand-400" />
            <h2>Governance Evidence Review</h2>
            <Badge variant={packageRecord ? 'success' : 'info'}>
              {packageRecord ? 'review package ready' : 'manual flow'}
            </Badge>
          </div>
          <p>Generate deployment evidence, map it to GitGov controls, and package JSON for customer or auditor review.</p>
        </div>
        <Button size="sm" variant="ghost" onClick={resetFlow} title="Clear the current evidence review flow">
          Reset
        </Button>
      </div>

      <div className="mb-4 rounded border border-warning-500/25 bg-warning-500/8 p-3 text-xs text-warning-100">
        <div className="flex items-center gap-2 font-medium">
          <AlertTriangle size={14} />
          No certification claim
        </div>
        <p className="mt-1 leading-5">
          This flow organizes deployment governance evidence for customer or auditor review. It is not SOC 2, ISO, NIST, PCI, SBS, LGPD, a certification, a compliance score, or an official regulatory claim.
        </p>
      </div>

      <div className="grid grid-cols-1 gap-3 xl:grid-cols-[minmax(0,1.2fr)_minmax(0,1fr)]">
        <div className="rounded-lg border border-white/8 bg-surface-900/60 p-3">
          <label htmlFor="compliance-gate-select" className="text-[10px] font-medium uppercase tracking-widest text-surface-500">
            Deployment Gate authorization
          </label>
          <select
            id="compliance-gate-select"
            value={effectiveAuthorizationId}
            onChange={(event) => setDraftAuthorizationId(event.target.value)}
            className="mt-2 w-full rounded border border-surface-600 bg-surface-800 px-2 py-2 text-xs text-surface-100 focus:border-brand-400 focus:outline-none"
          >
            {authorizations.length === 0 && <option value="">No authorizations loaded</option>}
            {authorizations.map((authorization) => (
              <option key={authorization.authorization_id} value={authorization.authorization_id}>
                {authorizationLabel(authorization)}
              </option>
            ))}
          </select>

          {selectedAuthorization ? (
            <div className="mt-3 grid grid-cols-1 gap-1 text-[11px] text-surface-400 md:grid-cols-2">
              <span className="truncate">Repo: <span className="text-surface-200">{selectedAuthorization.repository_full_name}</span></span>
              <span>Decision: <span className="text-surface-200">{selectedAuthorization.decision}</span></span>
              <span className="truncate">Policy: <span className="font-mono text-surface-200">{shortHash(selectedAuthorization.policy_checksum)}</span></span>
              <span>Created: <span className="text-surface-200">{formatTs(selectedAuthorization.created_at, displayTimezone)}</span></span>
              <span>Manual-first: <span className="text-surface-200">yes</span></span>
              <span>Agent required: <span className="text-surface-200">no</span></span>
            </div>
          ) : (
            <p className="mt-3 text-xs text-surface-600">
              Refresh Deployment Gate History first, then choose an authorization to package.
            </p>
          )}

          <div className="mt-4 grid grid-cols-1 gap-2 md:grid-cols-4">
            <Button
              size="sm"
              variant="secondary"
              loading={isExportCreating}
              disabled={!canGenerateExport}
              onClick={() => void handleGenerateExport()}
              title="Generate deployment evidence export"
            >
              <FileJson size={13} />
              Export
            </Button>
            <Button
              size="sm"
              variant="secondary"
              loading={isMappingCreating}
              disabled={!canGenerateMapping}
              onClick={() => void handleGenerateMapping()}
              title="Map the evidence export to GitGov release governance controls"
            >
              <Layers3 size={13} />
              Map
            </Button>
            <Button
              size="sm"
              variant="secondary"
              loading={isPackageCreating}
              disabled={!canGeneratePackage}
              onClick={() => void handleGeneratePackage()}
              title="Create the hashable JSON review package"
            >
              <PackageCheck size={13} />
              Package
            </Button>
            <Button
              size="sm"
              variant="outline"
              loading={isDownloading}
              disabled={!canDownloadPackage}
              onClick={() => void handleDownloadPackage()}
              title="Download the server-generated review package JSON"
            >
              <Download size={13} />
              JSON
            </Button>
          </div>

          {error && (
            <div className="mt-3 rounded border border-danger-500/20 bg-danger-500/8 p-2 text-[11px] text-danger-100">
              {error}
            </div>
          )}
        </div>

        <div className="rounded-lg border border-white/8 bg-surface-900/60 p-3">
          <div className="flex items-center gap-2">
            <ShieldCheck size={14} className="text-success-300" />
            <span className="text-xs font-medium text-surface-200">Review flags</span>
          </div>
          <div className="mt-3 grid grid-cols-2 gap-2 text-[11px]">
            <div className="rounded border border-white/6 bg-white/[0.03] p-2">
              <div className="text-surface-500">Compliance claim</div>
              <div className="mt-1 font-mono text-surface-100">{String(packageRecord?.compliance_claim ?? mappingRecord?.compliance_claim ?? false)}</div>
            </div>
            <div className="rounded border border-white/6 bg-white/[0.03] p-2">
              <div className="text-surface-500">Regulatory claim</div>
              <div className="mt-1 font-mono text-surface-100">{String(packageRecord?.regulatory_claim ?? mappingRecord?.regulatory_claim ?? false)}</div>
            </div>
            <div className="rounded border border-white/6 bg-white/[0.03] p-2">
              <div className="text-surface-500">Auditor review</div>
              <div className="mt-1 font-mono text-surface-100">{String(packageRecord?.requires_auditor_review ?? mappingRecord?.requires_auditor_review ?? true)}</div>
            </div>
            <div className="rounded border border-white/6 bg-white/[0.03] p-2">
              <div className="text-surface-500">Certification</div>
              <div className="mt-1 font-mono text-surface-100">{String(packageRecord?.certification ?? false)}</div>
            </div>
          </div>
          {reviewPackageArtifact && (
            <p className="mt-3 text-[11px] text-success-200">
              Server artifact downloaded and ready for local JSON save.
            </p>
          )}
        </div>
      </div>

      <div className="mt-4 grid grid-cols-1 gap-3 xl:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
        <div className="rounded-lg border border-white/8 bg-surface-900/60 p-3">
          <div className="flex items-center justify-between gap-2">
            <div>
              <div className="text-xs font-medium text-surface-200">Framework</div>
              <p className="mt-1 text-[11px] text-surface-500">
                Customer packs stay customer-owned and require customer or auditor review.
              </p>
            </div>
            <Button
              size="sm"
              variant="ghost"
              loading={isFrameworksLoading}
              onClick={() => void loadFrameworks()}
              title="Reload frameworks"
            >
              Reload
            </Button>
          </div>
          <label htmlFor="compliance-framework-select" className="mt-3 block text-[10px] font-medium uppercase tracking-widest text-surface-500">
            Mapping framework
          </label>
          <select
            id="compliance-framework-select"
            value={selectedFramework?.framework_id ?? selectedFrameworkId}
            onChange={(event) => selectFramework(event.target.value)}
            className="mt-2 w-full rounded border border-surface-600 bg-surface-800 px-2 py-2 text-xs text-surface-100 focus:border-brand-400 focus:outline-none"
          >
            {frameworks.length === 0 && <option value={selectedFrameworkId}>GitGov baseline loading</option>}
            {frameworks.map((framework) => (
              <option key={framework.framework_id} value={framework.framework_id}>
                {framework.name} {framework.version} / {framework.owner_type}
              </option>
            ))}
          </select>
          {selectedFramework && (
            <div className="mt-3 grid grid-cols-1 gap-1 text-[11px] text-surface-400 md:grid-cols-2">
              <span>Owner: <span className="text-surface-200">{selectedFramework.owner_name ?? selectedFramework.owner_type}</span></span>
              <span>Source: <span className="text-surface-200">{selectedFramework.source}</span></span>
              <span>Claims: <span className="text-surface-200">false</span></span>
              <span>Auditor review: <span className="text-surface-200">true</span></span>
              <span className="truncate md:col-span-2" title={selectedFramework.pack_hash ?? undefined}>
                Pack hash: <span className="font-mono text-surface-200">{shortHash(selectedFramework.pack_hash)}</span>
              </span>
            </div>
          )}
        </div>

        <div className="rounded-lg border border-white/8 bg-surface-900/60 p-3">
          <div className="flex items-center gap-2">
            <Upload size={14} className="text-brand-300" />
            <span className="text-xs font-medium text-surface-200">Import Customer Framework Pack</span>
          </div>
          <div className="mt-3 flex gap-2">
            <select
              value={packFormat}
              onChange={(event) => setPackFormat(event.target.value as 'json' | 'yaml')}
              className="w-24 rounded border border-surface-600 bg-surface-800 px-2 py-2 text-xs text-surface-100 focus:border-brand-400 focus:outline-none"
              title="Framework pack format"
            >
              <option value="json">JSON</option>
              <option value="yaml">YAML</option>
            </select>
            <Button
              size="sm"
              variant="secondary"
              loading={isFrameworkImporting}
              disabled={!packContent.trim()}
              onClick={() => void handleImportFramework()}
              title="Import customer-owned framework pack"
            >
              <Upload size={13} />
              Import
            </Button>
          </div>
          <textarea
            value={packContent}
            onChange={(event) => setPackContent(event.target.value)}
            placeholder="Paste customer-owned framework pack JSON or YAML"
            className="mt-3 h-28 w-full resize-y rounded border border-surface-600 bg-surface-950 px-2 py-2 font-mono text-[11px] text-surface-100 placeholder:text-surface-600 focus:border-brand-400 focus:outline-none"
          />
          {importResponse && (
            <p className="mt-2 text-[11px] text-success-200">
              Imported {importResponse.framework.name} with {importResponse.framework.controls?.length ?? importResponse.framework_pack.control_count} controls.
            </p>
          )}
        </div>
      </div>

      <div className="mt-4 grid grid-cols-1 gap-2 md:grid-cols-3">
        <div className="rounded border border-white/8 bg-white/[0.03] p-3">
          <div className="text-[10px] text-surface-500">Evidence export</div>
          <div className="mt-1 truncate font-mono text-xs text-surface-100">{exportRecord?.export_id ?? 'not generated'}</div>
          <div className="mt-1 truncate text-[10px] text-surface-500" title={exportRecord?.artifact_hash}>
            Hash: <span className="text-surface-300">{shortHash(exportRecord?.artifact_hash)}</span>
          </div>
        </div>
        <div className="rounded border border-white/8 bg-white/[0.03] p-3">
          <div className="text-[10px] text-surface-500">Control mapping</div>
          <div className="mt-1 truncate font-mono text-xs text-surface-100">{mappingRecord?.mapping_id ?? 'not generated'}</div>
          <div className="mt-1 text-[10px] text-surface-500">
            Controls: <span className="text-surface-300">{mappingItems.length || 0}</span>
          </div>
          <div className="mt-1 truncate text-[10px] text-surface-500" title={mappingRecord?.framework_id ?? selectedFramework?.framework_id}>
            Framework: <span className="text-surface-300">{mappingRecord?.framework_id ?? selectedFramework?.framework_id ?? 'not selected'}</span>
          </div>
        </div>
        <div className="rounded border border-white/8 bg-white/[0.03] p-3">
          <div className="text-[10px] text-surface-500">Review package</div>
          <div className="mt-1 truncate font-mono text-xs text-surface-100">{packageRecord?.review_package_id ?? 'not generated'}</div>
          <div className="mt-1 truncate text-[10px] text-surface-500" title={packageRecord?.artifact_hash}>
            Hash: <span className="text-surface-300">{shortHash(packageRecord?.artifact_hash)}</span>
          </div>
        </div>
      </div>

      {mappingItems.length > 0 && (
        <div className="mt-4 rounded-lg border border-white/8 bg-surface-900/60">
          <div className="flex flex-wrap items-center justify-between gap-2 border-b border-white/6 px-3 py-2">
            <span className="text-[11px] font-medium text-surface-300">Control coverage</span>
            <div className="flex flex-wrap gap-2 text-[10px] text-surface-500">
              <span>covered {counts.covered}</span>
              <span>partial {counts.partial}</span>
              <span>missing {counts.missing}</span>
            </div>
          </div>
          <div className="max-h-[320px] divide-y divide-white/6 overflow-auto">
            {mappingItems.map((item) => (
              <div key={item.control_id} className="p-3 text-xs">
                <div className="flex flex-wrap items-center gap-2">
                  <Badge variant={item.status === 'covered' || item.status === 'evidence_present' ? 'success' : item.status === 'partial' ? 'warning' : 'danger'}>
                    {item.status}
                  </Badge>
                  <span className="font-mono text-surface-400">{item.control_id}</span>
                  <span className="font-medium text-surface-100">{item.control_title}</span>
                </div>
                <div className="mt-2 grid grid-cols-1 gap-1 text-[11px] text-surface-400 md:grid-cols-2">
                  <span>Evidence refs: <span className="text-surface-200">{item.evidence_refs.length}</span></span>
                  <span>Missing: <span className="text-surface-200">{item.missing_evidence.length}</span></span>
                </div>
                {item.missing_evidence.length > 0 && (
                  <p className="mt-2 text-[11px] text-warning-100">
                    Missing evidence: {item.missing_evidence.join(', ')}
                  </p>
                )}
                <p className="mt-1 text-[11px] text-surface-500">{item.notes_safe}</p>
              </div>
            ))}
          </div>
        </div>
      )}

      {missing.length > 0 && (
        <div className="mt-3 rounded border border-warning-500/20 bg-warning-500/8 p-3 text-[11px] text-warning-100">
          Missing evidence surfaced for review: {missing.join(', ')}
        </div>
      )}
    </section>
  )
}
