import { useCallback, useEffect, useMemo, useState } from 'react'
import { Activity, AlertTriangle, CheckCircle2, PlayCircle, RefreshCw } from 'lucide-react'
import { Badge } from '@/components/shared/Badge'
import { Button } from '@/components/shared/Button'
import { formatTs } from '@/lib/timezone'
import { useControlPlaneStore } from '@/store/useControlPlaneStore'
import type { ChangeRiskEvaluationRecord, DeploymentGateAuthorizationRecord } from '@/store/useControlPlaneStore/types'

function riskVariant(level: string): 'success' | 'warning' | 'danger' | 'info' | 'neutral' {
  if (level === 'low') return 'success'
  if (level === 'medium') return 'warning'
  if (level === 'high') return 'danger'
  if (level === 'unknown') return 'info'
  return 'neutral'
}

function shortValue(value?: string | null): string {
  if (!value) return 'Not set'
  return value.length > 14 ? value.slice(0, 14) : value
}

function firstGate(authorizations: DeploymentGateAuthorizationRecord[]) {
  return authorizations[0] ?? null
}

function Field({
  label,
  value,
  onChange,
  placeholder,
}: {
  label: string
  value: string
  onChange: (value: string) => void
  placeholder?: string
}) {
  return (
    <label className="block">
      <span className="text-[10px] uppercase tracking-widest text-surface-500">{label}</span>
      <input
        value={value}
        onChange={(event) => onChange(event.target.value)}
        placeholder={placeholder}
        className="mt-1 w-full rounded border border-white/10 bg-surface-950/70 px-2 py-2 text-xs text-surface-100 outline-none transition-colors placeholder:text-surface-600 focus:border-brand-500/60"
      />
    </label>
  )
}

function EvaluationSummary({ evaluation }: { evaluation: ChangeRiskEvaluationRecord }) {
  return (
    <div className="rounded-lg border border-white/8 bg-surface-900/70 p-3 text-xs">
      <div className="flex flex-wrap items-center gap-2">
        <Badge variant={riskVariant(evaluation.risk_level)}>risk: {evaluation.risk_level}</Badge>
        {evaluation.advisory_only && <Badge variant="info">advisory only</Badge>}
        {!evaluation.llm_used && <Badge variant="neutral">no AI</Badge>}
        {!evaluation.agent_governance_used && <Badge variant="neutral">no agent</Badge>}
        {!evaluation.compliance_claim && !evaluation.certification && <Badge variant="neutral">no claim</Badge>}
      </div>

      <div className="mt-3 grid grid-cols-1 gap-3 lg:grid-cols-2">
        <div>
          <div className="text-[10px] uppercase tracking-widest text-surface-500">Reasons</div>
          <ul className="mt-1 list-disc space-y-1 pl-4 text-surface-300">
            {evaluation.risk_reasons.slice(0, 5).map((reason) => <li key={reason}>{reason}</li>)}
            {evaluation.risk_reasons.length === 0 && <li>No deterministic risk reasons recorded.</li>}
          </ul>
        </div>
        <div>
          <div className="text-[10px] uppercase tracking-widest text-surface-500">Manual actions</div>
          <ul className="mt-1 list-disc space-y-1 pl-4 text-surface-300">
            {evaluation.recommended_manual_actions.slice(0, 5).map((action) => <li key={action}>{action}</li>)}
            {evaluation.recommended_manual_actions.length === 0 && <li>No extra manual action recorded.</li>}
          </ul>
        </div>
      </div>

      {(evaluation.missing_evidence.length > 0 || evaluation.blocking_gaps.length > 0) && (
        <div className="mt-3 grid grid-cols-1 gap-2 lg:grid-cols-2">
          <div className="rounded border border-warning-500/20 bg-warning-500/8 p-2">
            <div className="text-[10px] uppercase tracking-widest text-warning-200">Missing evidence</div>
            <div className="mt-1 text-[11px] text-warning-50">
              {evaluation.missing_evidence.join(', ') || 'None'}
            </div>
          </div>
          <div className="rounded border border-danger-500/20 bg-danger-500/8 p-2">
            <div className="text-[10px] uppercase tracking-widest text-danger-200">Blocking gaps</div>
            <div className="mt-1 text-[11px] text-danger-50">
              {evaluation.blocking_gaps.join(', ') || 'None'}
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export function ChangeRiskPanel() {
  const selectedOrgName = useControlPlaneStore((state) => state.selectedOrgName)
  const deploymentGateAuthorizations = useControlPlaneStore((state) => state.deploymentGateAuthorizations)
  const loadDeploymentGateAuthorizations = useControlPlaneStore((state) => state.loadDeploymentGateAuthorizations)
  const evaluations = useControlPlaneStore((state) => state.changeRiskEvaluations)
  const evaluationsTotal = useControlPlaneStore((state) => state.changeRiskEvaluationsTotal)
  const selectedEvaluation = useControlPlaneStore((state) => state.changeRiskSelectedEvaluation)
  const isLoading = useControlPlaneStore((state) => state.isChangeRiskEvaluationsLoading)
  const isCreating = useControlPlaneStore((state) => state.isChangeRiskEvaluationCreating)
  const error = useControlPlaneStore((state) => state.changeRiskError)
  const displayTimezone = useControlPlaneStore((state) => state.displayTimezone)
  const loadEvaluations = useControlPlaneStore((state) => state.loadChangeRiskEvaluations)
  const createEvaluation = useControlPlaneStore((state) => state.createChangeRiskEvaluation)

  const latestGate = firstGate(deploymentGateAuthorizations)
  const [selectedGateId, setSelectedGateId] = useState('')
  const [repositoryFullName, setRepositoryFullName] = useState('')
  const [branch, setBranch] = useState('')
  const [commitSha, setCommitSha] = useState('')
  const [releaseId, setReleaseId] = useState('')
  const [environment, setEnvironment] = useState('production')
  const [changeId, setChangeId] = useState('')

  const selectedGate = useMemo(
    () => deploymentGateAuthorizations.find((gate) => gate.authorization_id === selectedGateId) ?? null,
    [deploymentGateAuthorizations, selectedGateId],
  )

  const applyGate = useCallback((gate: DeploymentGateAuthorizationRecord | null) => {
    if (!gate) return
    setSelectedGateId(gate.authorization_id)
    setRepositoryFullName(gate.repository_full_name)
    setBranch(gate.branch)
    setCommitSha(gate.target_sha)
    setReleaseId(gate.release_id)
    setEnvironment(gate.environment)
    setChangeId(gate.ticket_id || gate.release_id)
  }, [])

  const refreshAll = useCallback(() => {
    void loadDeploymentGateAuthorizations({
      org_name: selectedOrgName || null,
      repository_full_name: repositoryFullName.trim() || null,
      branch: repositoryFullName.trim() ? branch.trim() || null : null,
      limit: 10,
      offset: 0,
    })
    void loadEvaluations({
      org_name: selectedOrgName || null,
      repository_full_name: repositoryFullName.trim() || null,
      branch: repositoryFullName.trim() ? branch.trim() || null : null,
      limit: 10,
      offset: 0,
    })
  }, [branch, loadDeploymentGateAuthorizations, loadEvaluations, repositoryFullName, selectedOrgName])

  useEffect(() => {
    void loadEvaluations({ org_name: selectedOrgName || null, limit: 10, offset: 0 })
  }, [loadEvaluations, selectedOrgName])

  const canEvaluate =
    repositoryFullName.trim() &&
    branch.trim() &&
    environment.trim() &&
    (selectedGateId.trim() || releaseId.trim() || commitSha.trim() || changeId.trim())

  const evaluateRisk = async () => {
    if (!canEvaluate) return
    const evidenceRefs = selectedGate
      ? [
          `deployment_gate:${selectedGate.authorization_id}`,
          selectedGate.evidence_packet_hash ? `evidence_packet_hash:${selectedGate.evidence_packet_hash}` : '',
        ].filter(Boolean)
      : []
    await createEvaluation({
      org_name: selectedOrgName || null,
      deployment_gate_id: selectedGateId.trim() || null,
      release_id: releaseId.trim() || null,
      repository_full_name: repositoryFullName.trim(),
      branch: branch.trim(),
      commit_sha: commitSha.trim() || null,
      environment: environment.trim(),
      change_id: changeId.trim() || null,
      evidence_packet_hash: selectedGate?.evidence_packet_hash || null,
      evidence_refs: evidenceRefs,
    })
  }

  return (
    <section id="change-risk-advisory" className="glass-panel p-5 scroll-mt-4">
      <div className="card-header mb-4">
        <div>
          <div className="flex items-center gap-2">
            <Activity size={16} className="text-brand-400" />
            <h2>Change Risk Advisory</h2>
            <Badge variant={selectedEvaluation ? riskVariant(selectedEvaluation.risk_level) : 'info'}>
              {selectedEvaluation ? selectedEvaluation.risk_level : `${evaluations.length}/${evaluationsTotal}`}
            </Badge>
          </div>
          <p>Deterministic review aid for releases and deployment gates. Advisory only; it does not approve, block, certify, or deploy.</p>
        </div>
        <Button size="sm" variant="outline" loading={isLoading} onClick={refreshAll} title="Refresh gates and risk history">
          <RefreshCw size={14} />
          Refresh
        </Button>
      </div>

      <div className="mb-4 rounded border border-brand-500/20 bg-brand-500/8 p-3 text-xs text-brand-100">
        <div className="flex items-center gap-2 font-medium">
          <CheckCircle2 size={14} />
          Manual-first operating mode
        </div>
        <p className="mt-1 text-[11px] leading-5 text-brand-100/80">
          This assessment uses stored GitGov evidence only. No LLM, no Agent Governance dependency, no provider mutation, no repository mutation, and no compliance or certification claim.
        </p>
      </div>

      <div className="grid grid-cols-1 gap-3 xl:grid-cols-[1.1fr_0.9fr]">
        <div className="rounded-lg border border-white/8 bg-surface-900/60 p-3">
          <div className="mb-3 flex flex-wrap items-center justify-between gap-2">
            <div>
              <div className="text-xs font-semibold text-surface-100">Assessment input</div>
              <div className="text-[11px] text-surface-500">Select a recent Deployment Gate or enter release context manually.</div>
            </div>
            {latestGate && (
              <Button size="sm" variant="secondary" onClick={() => applyGate(latestGate)} title="Use latest deployment gate">
                Latest gate
              </Button>
            )}
          </div>

          <div className="grid grid-cols-1 gap-2 md:grid-cols-2">
            <label className="block md:col-span-2">
              <span className="text-[10px] uppercase tracking-widest text-surface-500">Deployment gate</span>
              <select
                value={selectedGateId}
                onChange={(event) => applyGate(deploymentGateAuthorizations.find((gate) => gate.authorization_id === event.target.value) ?? null)}
                className="mt-1 w-full rounded border border-white/10 bg-surface-950/70 px-2 py-2 text-xs text-surface-100 outline-none transition-colors focus:border-brand-500/60"
              >
                <option value="">Manual context only</option>
                {deploymentGateAuthorizations.map((gate) => (
                  <option key={gate.authorization_id} value={gate.authorization_id}>
                    {gate.release_id} / {gate.environment} / {gate.decision} / {shortValue(gate.target_sha)}
                  </option>
                ))}
              </select>
            </label>
            <Field label="Repository" value={repositoryFullName} onChange={setRepositoryFullName} placeholder="owner/repo" />
            <Field label="Branch" value={branch} onChange={setBranch} placeholder="main" />
            <Field label="Release" value={releaseId} onChange={setReleaseId} placeholder="KAN-121" />
            <Field label="Environment" value={environment} onChange={setEnvironment} placeholder="production" />
            <Field label="Commit SHA" value={commitSha} onChange={setCommitSha} placeholder="commit sha" />
            <Field label="Change ID" value={changeId} onChange={setChangeId} placeholder="ticket or CAB id" />
          </div>

          <div className="mt-3 flex flex-wrap items-center gap-2">
            <Button size="sm" loading={isCreating} disabled={!canEvaluate} onClick={() => void evaluateRisk()} title="Create advisory risk assessment">
              <PlayCircle size={14} />
              Assess risk
            </Button>
            <span className="text-[11px] text-surface-500">Creates immutable advisory evidence for the selected tenant.</span>
          </div>
          {error && (
            <div className="mt-3 flex items-center gap-2 rounded border border-danger-500/25 bg-danger-500/10 p-2 text-xs text-danger-100">
              <AlertTriangle size={14} />
              {error}
            </div>
          )}
        </div>

        <div className="space-y-3">
          {selectedEvaluation ? (
            <EvaluationSummary evaluation={selectedEvaluation} />
          ) : (
            <div className="rounded-lg border border-white/8 bg-surface-900/60 p-4 text-xs text-surface-500">
              No assessment selected yet.
            </div>
          )}
          <div className="rounded-lg border border-white/8 bg-surface-900/60">
            <div className="flex items-center justify-between border-b border-white/6 px-3 py-2">
              <span className="text-[11px] font-medium text-surface-300">Recent risk assessments</span>
              <span className="text-[10px] text-surface-600">{evaluationsTotal} total</span>
            </div>
            <div className="max-h-[340px] overflow-auto divide-y divide-white/6">
              {evaluations.map((evaluation) => (
                <div key={evaluation.evaluation_id} className="p-3 text-xs">
                  <div className="flex flex-wrap items-center gap-2">
                    <Badge variant={riskVariant(evaluation.risk_level)}>{evaluation.risk_level}</Badge>
                    <span className="font-medium text-surface-100">{evaluation.release_id || 'release not set'}</span>
                    <span className="text-surface-500">{evaluation.environment || 'environment not set'}</span>
                  </div>
                  <div className="mt-2 grid grid-cols-1 gap-1 text-[11px] text-surface-400 md:grid-cols-2">
                    <span className="truncate">Repo: <span className="text-surface-200">{evaluation.repository_full_name || 'Not set'}</span></span>
                    <span className="truncate">Branch: <span className="text-surface-200">{evaluation.branch || 'Not set'}</span></span>
                    <span className="truncate">Gate: <span className="font-mono text-surface-200">{evaluation.deployment_gate_id || 'Not linked'}</span></span>
                    <span>Created: <span className="text-surface-200">{formatTs(evaluation.created_at, displayTimezone)}</span></span>
                  </div>
                  <div className="mt-2 text-[11px] text-surface-500">
                    {evaluation.risk_reasons.slice(0, 2).join(', ') || 'No reasons recorded.'}
                  </div>
                </div>
              ))}
              {evaluations.length === 0 && (
                <div className="p-6 text-center text-xs text-surface-600">
                  No change risk assessments have been recorded for this tenant.
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </section>
  )
}
