import { useCallback, useEffect } from 'react'
import { AlertTriangle, Download, FileText, History, RefreshCw, ShieldCheck, ShieldX } from 'lucide-react'
import { Badge } from '@/components/shared/Badge'
import { Button } from '@/components/shared/Button'
import { formatTs } from '@/lib/timezone'
import { useControlPlaneStore } from '@/store/useControlPlaneStore'

function decisionVariant(decision: string): 'success' | 'warning' | 'danger' | 'info' | 'neutral' {
  if (decision === 'approved') return 'success'
  if (decision === 'break_glass') return 'warning'
  if (decision === 'advisory') return 'warning'
  if (decision === 'blocked') return 'danger'
  return 'neutral'
}

function riskVariant(risk: string | null | undefined): 'success' | 'warning' | 'danger' | 'info' | 'neutral' {
  if (risk === 'low') return 'success'
  if (risk === 'medium') return 'warning'
  if (risk === 'high') return 'danger'
  if (risk === 'unknown') return 'neutral'
  return 'info'
}

function shortSha(value: string): string {
  return value.length > 12 ? value.slice(0, 12) : value
}

function governanceString(value: Record<string, unknown> | null | undefined, key: string): string | null {
  const raw = value?.[key]
  return typeof raw === 'string' && raw.trim() ? raw : null
}

function governanceBoolean(value: Record<string, unknown> | null | undefined, key: string): boolean | null {
  const raw = value?.[key]
  return typeof raw === 'boolean' ? raw : null
}

export function DeploymentGateHistoryPanel() {
  const selectedOrgName = useControlPlaneStore((state) => state.selectedOrgName)
  const enterpriseAdoptionProfile = useControlPlaneStore((state) => state.enterpriseAdoptionProfile)
  const jiraCoverageFilters = useControlPlaneStore((state) => state.jiraCoverageFilters)
  const authorizations = useControlPlaneStore((state) => state.deploymentGateAuthorizations)
  const authorizationsTotal = useControlPlaneStore((state) => state.deploymentGateAuthorizationsTotal)
  const riskContexts = useControlPlaneStore((state) => state.deploymentGateRiskContexts)
  const updatedAt = useControlPlaneStore((state) => state.deploymentGateAuthorizationsUpdatedAt)
  const isLoading = useControlPlaneStore((state) => state.isDeploymentGateAuthorizationsLoading)
  const isRiskContextLoading = useControlPlaneStore((state) => state.isDeploymentGateRiskContextLoading)
  const riskContextError = useControlPlaneStore((state) => state.deploymentGateRiskContextError)
  const displayTimezone = useControlPlaneStore((state) => state.displayTimezone)
  const loadAuthorizations = useControlPlaneStore((state) => state.loadDeploymentGateAuthorizations)
  const getRiskContext = useControlPlaneStore((state) => state.getDeploymentGateRiskContext)
  const downloadManifest = useControlPlaneStore((state) => state.downloadChangeRiskCabDecisionManifest)

  const defaultRepository =
    enterpriseAdoptionProfile?.repository_full_name ||
    jiraCoverageFilters.repo_full_name ||
    ''
  const defaultBranch =
    enterpriseAdoptionProfile?.default_branch ||
    jiraCoverageFilters.branch ||
    'main'

  const refreshHistory = useCallback(() => loadAuthorizations({
    org_name: selectedOrgName || null,
    repository_full_name: defaultRepository || null,
    branch: defaultRepository ? defaultBranch || null : null,
    limit: 10,
    offset: 0,
  }), [defaultBranch, defaultRepository, loadAuthorizations, selectedOrgName])

  useEffect(() => {
    void refreshHistory()
  }, [refreshHistory])

  const loadRiskContext = useCallback((deploymentGateId: string) => {
    void getRiskContext(deploymentGateId, { org_name: selectedOrgName || null })
  }, [getRiskContext, selectedOrgName])

  const downloadLatestManifest = useCallback((manifestId: string) => {
    void downloadManifest(manifestId, { org_name: selectedOrgName || null })
  }, [downloadManifest, selectedOrgName])

  return (
    <section id="deployment-gate-history" className="glass-panel p-5 scroll-mt-4">
      <div className="card-header mb-4">
        <div>
          <div className="flex items-center gap-2">
            <History size={16} className="text-brand-400" />
            <h2>Deployment Gate History</h2>
            <Badge variant={authorizations.length > 0 ? 'success' : 'info'}>
              {authorizations.length}/{authorizationsTotal}
            </Badge>
          </div>
          <p>Deploy authorization attempts recorded by the CI/CD-facing Deployment Gates API.</p>
        </div>
        <Button
          size="sm"
          variant="outline"
          loading={isLoading}
          onClick={() => void refreshHistory()}
          title="Refresh deployment authorization history"
        >
          <RefreshCw size={14} />
          Refresh
        </Button>
      </div>

      <div className="grid grid-cols-1 gap-2 md:grid-cols-4">
        <div className="rounded border border-white/8 bg-white/[0.03] p-3">
          <div className="text-[10px] text-surface-500">Repository</div>
          <div className="mt-1 truncate text-xs text-surface-200">{defaultRepository || 'All scoped repos'}</div>
        </div>
        <div className="rounded border border-white/8 bg-white/[0.03] p-3">
          <div className="text-[10px] text-surface-500">Branch</div>
          <div className="mt-1 truncate text-xs text-surface-200">{defaultRepository ? defaultBranch || 'All branches' : 'All branches'}</div>
        </div>
        <div className="rounded border border-white/8 bg-white/[0.03] p-3">
          <div className="text-[10px] text-surface-500">Loaded</div>
          <div className="mt-1 truncate text-xs text-surface-200">
            {updatedAt ? formatTs(updatedAt, displayTimezone) : 'Not loaded'}
          </div>
        </div>
        <div className="rounded border border-white/8 bg-white/[0.03] p-3">
          <div className="text-[10px] text-surface-500">Blocking outcomes</div>
          <div className="mt-1 text-xs text-surface-200">
            {authorizations.filter((item) => item.blocking || item.decision === 'blocked').length}
          </div>
        </div>
      </div>

      <div className="mt-4 rounded-lg border border-white/8 bg-surface-900/60">
        <div className="flex items-center justify-between border-b border-white/6 px-3 py-2">
          <span className="text-[11px] font-medium text-surface-300">Recent authorizations</span>
          <span className="text-[10px] text-surface-600">{authorizationsTotal} total</span>
        </div>
        <div className="max-h-[460px] overflow-auto divide-y divide-white/6">
          {authorizations.map((authorization) => (
            <div key={authorization.authorization_id} className="p-3 text-xs">
              {(() => {
                const riskContext = riskContexts[authorization.authorization_id]
                const latestManifest = riskContext?.cab_decision_manifests[0]
                const latestPacket = riskContext?.cab_packets[0]
                const latestEvaluation = riskContext?.change_risk_evaluations[0]
                return (
                  <>
              <div className="flex flex-wrap items-center gap-2">
                {authorization.approved ? (
                  <ShieldCheck size={14} className="text-success-300" />
                ) : (
                  <ShieldX size={14} className="text-danger-300" />
                )}
                <Badge variant={decisionVariant(authorization.decision)}>{authorization.decision}</Badge>
                <span className="font-medium text-surface-100">{authorization.release_id}</span>
                <span className="text-surface-500">{authorization.environment}</span>
                {authorization.break_glass_used && <Badge variant="warning">break-glass used</Badge>}
                {authorization.break_glass_used && authorization.break_glass_approval_id && <Badge variant="success">pre-approved</Badge>}
                {!authorization.break_glass_used && authorization.break_glass_eligible && <Badge variant="warning">break-glass eligible</Badge>}
                {governanceString(authorization.governance_decision, 'decision') && (
                  <Badge variant="info">shared: {governanceString(authorization.governance_decision, 'decision')}</Badge>
                )}
                {governanceBoolean(authorization.governance_decision, 'agent_governance_used') === false && (
                  <Badge variant="neutral">agent not used</Badge>
                )}
              </div>

              <div className="mt-2 grid grid-cols-1 gap-1 text-[11px] text-surface-400 md:grid-cols-2">
                <span className="truncate">Repo: <span className="text-surface-200">{authorization.repository_full_name}</span></span>
                <span className="truncate">Branch: <span className="text-surface-200">{authorization.branch}</span></span>
                <span className="truncate">Target: <span className="font-mono text-surface-200">{shortSha(authorization.target_sha)}</span></span>
                <span className="truncate">Deployer: <span className="text-surface-200">{authorization.deployer}</span></span>
                <span>Blocking: <span className="text-surface-200">{authorization.blocking ? 'yes' : 'no'}</span></span>
                <span>Would block: <span className="text-surface-200">{authorization.would_block ? 'yes' : 'no'}</span></span>
                <span>Approvals: <span className="text-surface-200">{authorization.evaluation.valid_approval_count}/{authorization.evaluation.required_approval_count}</span></span>
                <span>Mode: <span className="text-surface-200">manual-first</span></span>
                <span>Created: <span className="text-surface-200">{formatTs(authorization.created_at, displayTimezone)}</span></span>
              </div>

              <div className="mt-2 text-[11px] text-surface-300">{authorization.reason}</div>
              <div className="mt-3 rounded border border-white/8 bg-surface-950/40 p-3">
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <div className="flex items-center gap-2">
                    <FileText size={14} className="text-brand-300" />
                    <span className="text-[11px] font-medium text-surface-200">Risk & CAB Context</span>
                    {riskContext?.latest_risk_level && (
                      <Badge variant={riskVariant(riskContext.latest_risk_level)}>
                        {riskContext.latest_risk_level}
                      </Badge>
                    )}
                    {riskContext?.latest_review_status && (
                      <Badge variant="info">{riskContext.latest_review_status}</Badge>
                    )}
                  </div>
                  <Button
                    size="sm"
                    variant="ghost"
                    loading={isRiskContextLoading}
                    onClick={() => loadRiskContext(authorization.authorization_id)}
                    title="Load read-only Risk and CAB context"
                  >
                    <RefreshCw size={13} />
                    Context
                  </Button>
                </div>
                <div className="mt-2 rounded border border-warning-500/20 bg-warning-500/8 px-2 py-1 text-[10px] text-warning-100">
                  Advisory only. Does not approve, block, certify, or deploy.
                </div>
                {riskContext ? (
                  <div className="mt-3 grid grid-cols-1 gap-2 md:grid-cols-3">
                    <div className="rounded border border-white/8 bg-white/[0.03] p-2">
                      <div className="text-[10px] text-surface-500">Risk evaluations</div>
                      <div className="mt-1 text-[11px] text-surface-200">{riskContext.change_risk_evaluations.length}</div>
                      <div className="mt-1 truncate font-mono text-[10px] text-surface-500" title={latestEvaluation?.evaluation_id}>
                        {latestEvaluation?.evaluation_id || 'No risk context yet'}
                      </div>
                      <div className="mt-1 text-[10px] text-surface-500">Rules: {riskContext.triggered_rules_count}</div>
                    </div>
                    <div className="rounded border border-white/8 bg-white/[0.03] p-2">
                      <div className="text-[10px] text-surface-500">CAB packet</div>
                      <div className="mt-1 truncate font-mono text-[10px] text-surface-300" title={latestPacket?.packet_id}>
                        {latestPacket?.packet_id || 'No packet linked'}
                      </div>
                      <div className="mt-1 truncate font-mono text-[10px] text-surface-500" title={latestPacket?.artifact_hash}>
                        {latestPacket?.artifact_hash ? shortSha(latestPacket.artifact_hash) : 'No packet hash'}
                      </div>
                    </div>
                    <div className="rounded border border-white/8 bg-white/[0.03] p-2">
                      <div className="text-[10px] text-surface-500">Decision manifest</div>
                      <div className="mt-1 truncate font-mono text-[10px] text-surface-300" title={latestManifest?.manifest_id}>
                        {latestManifest?.manifest_id || 'No manifest linked'}
                      </div>
                      <div className="mt-1 truncate font-mono text-[10px] text-surface-500" title={latestManifest?.manifest_hash}>
                        {latestManifest?.manifest_hash ? shortSha(latestManifest.manifest_hash) : 'No manifest hash'}
                      </div>
                      {latestManifest && (
                        <Button
                          size="sm"
                          variant="outline"
                          className="mt-2"
                          onClick={() => downloadLatestManifest(latestManifest.manifest_id)}
                          title="Download CAB decision manifest"
                        >
                          <Download size={13} />
                          Download
                        </Button>
                      )}
                    </div>
                  </div>
                ) : (
                  <div className="mt-3 text-[11px] text-surface-500">
                    {riskContextError || 'No risk context loaded yet.'}
                  </div>
                )}
              </div>
              {authorization.break_glass_used && (
                <div className="mt-2 rounded border border-warning-500/20 bg-warning-500/8 p-2 text-[11px] text-warning-100">
                  <div className="flex items-center gap-1 font-medium">
                    <AlertTriangle size={13} />
                    Break-glass authorization
                  </div>
                  <div className="mt-1">
                    Reason: <span className="text-warning-50">{authorization.break_glass_reason || authorization.reason}</span>
                  </div>
                  <div className="mt-1 grid grid-cols-1 gap-1 md:grid-cols-2">
                    <span>Authorized by: <span className="text-warning-50">{authorization.break_glass_authorized_by || authorization.requested_by}</span></span>
                    <span>Expires: <span className="text-warning-50">{authorization.break_glass_expires_at ? formatTs(authorization.break_glass_expires_at, displayTimezone) : 'Not set'}</span></span>
                    <span className="truncate" title={authorization.break_glass_approval_id || undefined}>Approval: <span className="font-mono text-warning-50">{authorization.break_glass_approval_id || 'Not linked'}</span></span>
                    <span className="truncate" title={authorization.break_glass_approval_hash || undefined}>Approval hash: <span className="font-mono text-warning-50">{authorization.break_glass_approval_hash ? authorization.break_glass_approval_hash.slice(0, 16) : 'Not linked'}</span></span>
                  </div>
                </div>
              )}
              {authorization.warnings.length > 0 && (
                <ul className="mt-2 list-disc space-y-1 pl-4 text-[11px] text-warning-100">
                  {authorization.warnings.slice(0, 3).map((warning) => <li key={warning}>{warning}</li>)}
                </ul>
              )}
              {authorization.blocked_by.length > 0 && (
                <ul className="mt-2 list-disc space-y-1 pl-4 text-[11px] text-danger-100">
                  {authorization.blocked_by.slice(0, 3).map((blocker) => <li key={blocker}>{blocker}</li>)}
                </ul>
              )}
              <div className="mt-2 grid grid-cols-1 gap-1 text-[10px] text-surface-500 md:grid-cols-2">
                <span className="truncate" title={authorization.authorization_id}>Authorization: <span className="font-mono text-surface-300">{authorization.authorization_id}</span></span>
                <span className="truncate" title={authorization.evidence_packet_hash}>Evidence: <span className="font-mono text-surface-300">{authorization.evidence_packet_hash.slice(0, 16)}</span></span>
                <span className="truncate" title={authorization.policy_checksum}>Policy checksum: <span className="font-mono text-surface-300">{authorization.policy_checksum.slice(0, 16)}</span></span>
                <span className="truncate">Requested by: <span className="text-surface-300">{authorization.requested_by}</span></span>
              </div>
                  </>
                )
              })()}
            </div>
          ))}
          {authorizations.length === 0 && (
            <div className="p-8 text-center text-xs text-surface-600">
              No deployment gate authorizations in the current filter.
            </div>
          )}
        </div>
      </div>
    </section>
  )
}
