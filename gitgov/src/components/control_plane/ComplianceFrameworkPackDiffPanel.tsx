import { GitCompare } from 'lucide-react'
import { useMemo, useState } from 'react'
import { Badge } from '@/components/shared/Badge'
import { Button } from '@/components/shared/Button'
import { useControlPlaneStore } from '@/store/useControlPlaneStore'
import type { ComplianceFrameworkPackDiffControl, ComplianceFrameworkPackRecord } from '@/store/useControlPlaneStore/types'

function packLabel(pack: ComplianceFrameworkPackRecord): string {
  return `${pack.framework_name} ${pack.framework_version} / ${pack.review_status}`
}

function changeVariant(changeType: string) {
  if (changeType === 'added') return 'success'
  if (changeType === 'removed') return 'danger'
  if (changeType === 'changed') return 'warning'
  return 'info'
}

function controlTitle(control: ComplianceFrameworkPackDiffControl): string {
  return control.target?.title ?? control.base?.title ?? control.control_id
}

export function ComplianceFrameworkPackDiffPanel() {
  const packs = useControlPlaneStore((state) => state.complianceFrameworkPacks)
  const diff = useControlPlaneStore((state) => state.complianceFrameworkPackDiff)
  const isLoading = useControlPlaneStore((state) => state.isComplianceFrameworkPackDiffLoading)
  const loadDiff = useControlPlaneStore((state) => state.loadComplianceFrameworkPackDiff)
  const [basePackId, setBasePackId] = useState('')
  const [targetPackId, setTargetPackId] = useState('')

  const eligiblePacks = useMemo(
    () => packs.filter((pack) => pack.owner_type === 'customer' && pack.source === 'customer_provided'),
    [packs],
  )

  if (eligiblePacks.length < 2) {
    return null
  }

  const hasBaseSelection = eligiblePacks.some((pack) => pack.framework_pack_id === basePackId)
  const hasTargetSelection = eligiblePacks.some((pack) => pack.framework_pack_id === targetPackId)
  const selectedBasePackId = hasBaseSelection
    ? basePackId
    : eligiblePacks[1]?.framework_pack_id || eligiblePacks[0].framework_pack_id
  const selectedTargetPackId = hasTargetSelection ? targetPackId : eligiblePacks[0].framework_pack_id
  const changedControls = diff?.controls.filter((control) => control.change_type !== 'unchanged') ?? []

  return (
    <div className="mt-4 border-t border-white/8 pt-4">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div>
          <div className="text-xs font-medium text-surface-200">Framework Pack Diff</div>
          <p className="mt-1 text-[11px] text-surface-500">
            Compare customer pack versions before mapping them into audit evidence.
          </p>
        </div>
        {diff && (
          <Badge variant={diff.same_original_framework ? 'success' : 'warning'}>
            {diff.original_framework_id}
          </Badge>
        )}
      </div>

      <div className="mt-3 grid grid-cols-1 gap-2 md:grid-cols-[1fr_1fr_auto]">
        <select
          value={selectedBasePackId}
          onChange={(event) => setBasePackId(event.target.value)}
          className="rounded border border-surface-600 bg-surface-800 px-2 py-2 text-xs text-surface-100 focus:border-brand-400 focus:outline-none"
          aria-label="Base framework pack"
        >
          {eligiblePacks.map((pack) => (
            <option key={pack.framework_pack_id} value={pack.framework_pack_id}>
              Base: {packLabel(pack)}
            </option>
          ))}
        </select>
        <select
          value={selectedTargetPackId}
          onChange={(event) => setTargetPackId(event.target.value)}
          className="rounded border border-surface-600 bg-surface-800 px-2 py-2 text-xs text-surface-100 focus:border-brand-400 focus:outline-none"
          aria-label="Target framework pack"
        >
          {eligiblePacks.map((pack) => (
            <option key={pack.framework_pack_id} value={pack.framework_pack_id}>
              Target: {packLabel(pack)}
            </option>
          ))}
        </select>
        <Button
          size="sm"
          variant="secondary"
          loading={isLoading}
          disabled={!selectedBasePackId || !selectedTargetPackId || selectedBasePackId === selectedTargetPackId}
          onClick={() => void loadDiff(selectedBasePackId, selectedTargetPackId)}
          title="Compare framework pack versions"
        >
          <GitCompare size={13} />
          Diff
        </Button>
      </div>

      {diff && (
        <>
          <div className="mt-3 grid grid-cols-2 gap-2 text-[11px] text-surface-400 md:grid-cols-4">
            <span>Added: <span className="text-surface-100">{diff.summary.added}</span></span>
            <span>Removed: <span className="text-surface-100">{diff.summary.removed}</span></span>
            <span>Changed: <span className="text-surface-100">{diff.summary.changed}</span></span>
            <span>Unchanged: <span className="text-surface-100">{diff.summary.unchanged}</span></span>
          </div>
          <div className="mt-3 space-y-2">
            {changedControls.length === 0 ? (
              <div className="text-[11px] text-surface-500">No changed controls between these pack versions.</div>
            ) : changedControls.map((control) => (
              <div key={control.control_id} className="grid grid-cols-[auto_1fr] gap-2 text-[11px]">
                <Badge variant={changeVariant(control.change_type)}>{control.change_type}</Badge>
                <div className="min-w-0">
                  <div className="truncate text-surface-200">
                    <span className="font-mono">{control.control_id}</span> {controlTitle(control)}
                  </div>
                  {control.changed_fields.length > 0 && (
                    <div className="mt-0.5 truncate text-surface-500">
                      Fields: {control.changed_fields.join(', ')}
                    </div>
                  )}
                </div>
              </div>
            ))}
          </div>
          <div className="mt-3 grid grid-cols-2 gap-2 text-[11px] text-surface-500 md:grid-cols-4">
            <span>Compliance claim: <span className="text-surface-200">{String(diff.compliance_claim)}</span></span>
            <span>Regulatory claim: <span className="text-surface-200">{String(diff.regulatory_claim)}</span></span>
            <span>GitGov certifies: <span className="text-surface-200">{String(diff.gitgov_certifies)}</span></span>
            <span>Official mapping: <span className="text-surface-200">{String(diff.official_regulatory_mapping)}</span></span>
          </div>
        </>
      )}
    </div>
  )
}
