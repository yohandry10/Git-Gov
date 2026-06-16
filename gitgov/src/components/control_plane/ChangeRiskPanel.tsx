import { useCallback, useEffect, useMemo, useState } from 'react'
import { Activity, AlertTriangle, CheckCircle2, ClipboardCheck, PlayCircle, RefreshCw } from 'lucide-react'
import { Badge } from '@/components/shared/Badge'
import { Button } from '@/components/shared/Button'
import { formatTs } from '@/lib/timezone'
import { useControlPlaneStore } from '@/store/useControlPlaneStore'
import type {
  ChangeRiskEvaluationRecord,
  ChangeRiskEvaluationReviewResponse,
  ChangeRiskReviewStatus,
  ChangeRiskEvaluationTraceResponse,
  ChangeRiskRuleCatalogResponse,
  DeploymentGateAuthorizationRecord,
} from '@/store/useControlPlaneStore/types'

function riskVariant(level: string): 'success' | 'warning' | 'danger' | 'info' | 'neutral' {
  if (level === 'low') return 'success'
  if (level === 'medium') return 'warning'
  if (level === 'high') return 'danger'
  if (level === 'unknown') return 'info'
  return 'neutral'
}

function reviewVariant(status: string): 'success' | 'warning' | 'danger' | 'info' | 'neutral' {
  if (status === 'reviewed') return 'success'
  if (status === 'accepted_risk') return 'warning'
  if (status === 'needs_mitigation') return 'info'
  if (status === 'rejected') return 'danger'
  if (status === 'needs_review') return 'neutral'
  return 'neutral'
}

function reviewLabel(status: string): string {
  return status.replaceAll('_', ' ')
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

type TraceRuleEntry = {
  rule_id: string
  title: string
  severity: string
  triggered: boolean
  manual_action_hint: string
}

function readTraceRules(trace: ChangeRiskEvaluationTraceResponse | null): TraceRuleEntry[] {
  const rawRules = trace?.evaluation_trace?.rules
  if (!Array.isArray(rawRules)) return []
  return rawRules
    .filter((rule): rule is Record<string, unknown> => Boolean(rule) && typeof rule === 'object')
    .map((rule) => ({
      rule_id: String(rule.rule_id ?? ''),
      title: String(rule.title ?? rule.rule_id ?? ''),
      severity: String(rule.severity ?? 'medium'),
      triggered: Boolean(rule.triggered),
      manual_action_hint: String(rule.manual_action_hint ?? ''),
    }))
    .filter((rule) => rule.rule_id)
}

function WhyThisRisk({
  evaluation,
  trace,
  catalog,
  isLoading,
  onReload,
}: {
  evaluation: ChangeRiskEvaluationRecord
  trace: ChangeRiskEvaluationTraceResponse | null
  catalog: ChangeRiskRuleCatalogResponse | null
  isLoading: boolean
  onReload: () => void
}) {
  const traceRules = readTraceRules(trace)
  const triggered = traceRules.filter((rule) => rule.triggered)
  const nonTriggeredCount = trace?.non_triggered_rules?.length ?? evaluation.non_triggered_rules?.length ?? 0

  return (
    <div className="mt-3 rounded border border-info-500/20 bg-info-500/8 p-3">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div>
          <div className="text-[10px] uppercase tracking-widest text-info-200">Why this risk?</div>
          <div className="mt-1 text-[11px] text-info-50">
            Ruleset {trace?.ruleset_version || evaluation.ruleset_version || 'not recorded'} · {triggered.length || evaluation.triggered_rules.length} triggered · {nonTriggeredCount} not triggered
          </div>
        </div>
        <Button size="sm" variant="outline" loading={isLoading} onClick={onReload} title="Reload evaluation trace">
          <RefreshCw size={13} />
          Trace
        </Button>
      </div>

      <div className="mt-2 rounded border border-white/8 bg-surface-950/40 p-2 text-[11px] text-surface-300">
        Advisory only. Does not approve, block, certify, or deploy.
      </div>

      <div className="mt-3 grid grid-cols-1 gap-2">
        {(triggered.length > 0 ? triggered : evaluation.triggered_rules.map((ruleId) => ({
          rule_id: ruleId,
          title: catalog?.rules.find((rule) => rule.rule_id === ruleId)?.title ?? ruleId,
          severity: catalog?.rules.find((rule) => rule.rule_id === ruleId)?.severity ?? 'medium',
          triggered: true,
          manual_action_hint: catalog?.rules.find((rule) => rule.rule_id === ruleId)?.manual_action_hint ?? '',
        }))).slice(0, 6).map((rule) => (
          <div key={rule.rule_id} className="rounded border border-white/8 bg-surface-950/50 p-2">
            <div className="flex flex-wrap items-center gap-2">
              <Badge variant={riskVariant(rule.severity)}>{rule.severity}</Badge>
              <span className="font-medium text-surface-100">{rule.title}</span>
              <span className="font-mono text-[10px] text-surface-500">{rule.rule_id}</span>
            </div>
            {rule.manual_action_hint && (
              <div className="mt-1 text-[11px] text-surface-400">{rule.manual_action_hint}</div>
            )}
          </div>
        ))}
        {triggered.length === 0 && evaluation.triggered_rules.length === 0 && (
          <div className="rounded border border-white/8 bg-surface-950/50 p-2 text-[11px] text-surface-400">
            No risk rule triggered for this evaluation.
          </div>
        )}
      </div>

      <div className="mt-3 grid grid-cols-1 gap-2 text-[11px] md:grid-cols-2">
        <div className="truncate text-surface-400">
          Catalog: <span className="font-mono text-surface-200">{catalog?.catalog_hash || 'not loaded'}</span>
        </div>
        <div className="truncate text-surface-400">
          Trace: <span className="font-mono text-surface-200">{trace?.trace_hash || evaluation.trace_hash || 'not recorded'}</span>
        </div>
      </div>
    </div>
  )
}

function ManualReview({
  evaluation,
  review,
  isLoading,
  isUpdating,
  displayTimezone,
  onReload,
  onSubmit,
}: {
  evaluation: ChangeRiskEvaluationRecord
  review: ChangeRiskEvaluationReviewResponse | null
  isLoading: boolean
  isUpdating: boolean
  displayTimezone: string
  onReload: () => void
  onSubmit: (payload: {
    review_status: ChangeRiskReviewStatus
    review_notes: string
    mitigation_notes: string
    decision_reason: string
  }) => void
}) {
  const effectiveReview = review?.evaluation_id === evaluation.evaluation_id ? review : null
  const [reviewStatus, setReviewStatus] = useState<ChangeRiskReviewStatus>(evaluation.review_status || 'needs_review')
  const [reviewNotes, setReviewNotes] = useState(effectiveReview?.review_notes_safe ?? evaluation.review_notes_safe ?? '')
  const [mitigationNotes, setMitigationNotes] = useState(effectiveReview?.mitigation_notes_safe ?? evaluation.mitigation_notes_safe ?? '')
  const [decisionReason, setDecisionReason] = useState(effectiveReview?.decision_reason_safe ?? evaluation.decision_reason_safe ?? '')

  const reviewer = effectiveReview?.reviewed_by_user_id ?? evaluation.reviewed_by_user_id
  const reviewedAt = effectiveReview?.reviewed_at ?? evaluation.reviewed_at
  const updatedAt = effectiveReview?.review_updated_at ?? evaluation.review_updated_at

  return (
    <div className="mt-3 rounded border border-white/8 bg-surface-950/35 p-3">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="flex items-center gap-2">
          <ClipboardCheck size={14} className="text-brand-300" />
          <div>
            <div className="text-[10px] uppercase tracking-widest text-surface-500">Manual Review</div>
            <div className="mt-1 flex flex-wrap items-center gap-2 text-[11px] text-surface-400">
              <Badge variant={reviewVariant(effectiveReview?.review_status || evaluation.review_status || 'needs_review')}>
                {reviewLabel(effectiveReview?.review_status || evaluation.review_status || 'needs_review')}
              </Badge>
              <span>{reviewer || 'No reviewer'}</span>
              <span>{reviewedAt ? formatTs(reviewedAt, displayTimezone) : 'Not reviewed'}</span>
            </div>
          </div>
        </div>
        <Button size="sm" variant="outline" loading={isLoading} onClick={onReload} title="Reload manual review">
          <RefreshCw size={13} />
          Review
        </Button>
      </div>

      <div className="mt-3 grid grid-cols-1 gap-2 md:grid-cols-2">
        {(['needs_review', 'reviewed', 'accepted_risk', 'needs_mitigation', 'rejected'] as ChangeRiskReviewStatus[]).map((status) => (
          <button
            key={status}
            type="button"
            onClick={() => setReviewStatus(status)}
            className={`rounded border px-2 py-2 text-left text-[11px] transition-colors ${
              reviewStatus === status
                ? 'border-brand-500/60 bg-brand-500/15 text-brand-50'
                : 'border-white/8 bg-surface-900/60 text-surface-300 hover:bg-white/5'
            }`}
          >
            {reviewLabel(status)}
          </button>
        ))}
      </div>

      <div className="mt-3 grid grid-cols-1 gap-2">
        <label className="block">
          <span className="text-[10px] uppercase tracking-widest text-surface-500">Review notes</span>
          <textarea
            value={reviewNotes}
            onChange={(event) => setReviewNotes(event.target.value)}
            rows={2}
            maxLength={1000}
            className="mt-1 w-full rounded border border-white/10 bg-surface-950/70 px-2 py-2 text-xs text-surface-100 outline-none transition-colors placeholder:text-surface-600 focus:border-brand-500/60"
          />
        </label>
        <label className="block">
          <span className="text-[10px] uppercase tracking-widest text-surface-500">Mitigation notes</span>
          <textarea
            value={mitigationNotes}
            onChange={(event) => setMitigationNotes(event.target.value)}
            rows={2}
            maxLength={1000}
            className="mt-1 w-full rounded border border-white/10 bg-surface-950/70 px-2 py-2 text-xs text-surface-100 outline-none transition-colors placeholder:text-surface-600 focus:border-brand-500/60"
          />
        </label>
        <label className="block">
          <span className="text-[10px] uppercase tracking-widest text-surface-500">Decision reason</span>
          <textarea
            value={decisionReason}
            onChange={(event) => setDecisionReason(event.target.value)}
            rows={2}
            maxLength={1000}
            className="mt-1 w-full rounded border border-white/10 bg-surface-950/70 px-2 py-2 text-xs text-surface-100 outline-none transition-colors placeholder:text-surface-600 focus:border-brand-500/60"
          />
        </label>
      </div>

      <div className="mt-3 flex flex-wrap items-center gap-2">
        <Button
          size="sm"
          loading={isUpdating}
          onClick={() => onSubmit({ review_status: reviewStatus, review_notes: reviewNotes, mitigation_notes: mitigationNotes, decision_reason: decisionReason })}
          title="Save manual review"
        >
          <ClipboardCheck size={14} />
          Save review
        </Button>
        <span className="text-[11px] text-surface-500">
          {updatedAt ? `Updated ${formatTs(updatedAt, displayTimezone)}` : 'Advisory only; no deployment action is taken.'}
        </span>
      </div>
    </div>
  )
}

function EvaluationSummary({
  evaluation,
  trace,
  catalog,
  isTraceLoading,
  review,
  isReviewLoading,
  isReviewUpdating,
  displayTimezone,
  onReloadTrace,
  onReloadReview,
  onSubmitReview,
}: {
  evaluation: ChangeRiskEvaluationRecord
  trace: ChangeRiskEvaluationTraceResponse | null
  catalog: ChangeRiskRuleCatalogResponse | null
  isTraceLoading: boolean
  review: ChangeRiskEvaluationReviewResponse | null
  isReviewLoading: boolean
  isReviewUpdating: boolean
  displayTimezone: string
  onReloadTrace: () => void
  onReloadReview: () => void
  onSubmitReview: Parameters<typeof ManualReview>[0]['onSubmit']
}) {
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
      <WhyThisRisk
        evaluation={evaluation}
        trace={trace}
        catalog={catalog}
        isLoading={isTraceLoading}
        onReload={onReloadTrace}
      />
      <ManualReview
        key={`${evaluation.evaluation_id}:${review?.review_updated_at ?? evaluation.review_updated_at ?? 0}:${review?.review_status ?? evaluation.review_status ?? 'needs_review'}`}
        evaluation={evaluation}
        review={review}
        isLoading={isReviewLoading}
        isUpdating={isReviewUpdating}
        displayTimezone={displayTimezone}
        onReload={onReloadReview}
        onSubmit={onSubmitReview}
      />
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
  const ruleCatalog = useControlPlaneStore((state) => state.changeRiskRuleCatalog)
  const evaluationTrace = useControlPlaneStore((state) => state.changeRiskEvaluationTrace)
  const evaluationReview = useControlPlaneStore((state) => state.changeRiskEvaluationReview)
  const isLoading = useControlPlaneStore((state) => state.isChangeRiskEvaluationsLoading)
  const isRulesLoading = useControlPlaneStore((state) => state.isChangeRiskRulesLoading)
  const isTraceLoading = useControlPlaneStore((state) => state.isChangeRiskTraceLoading)
  const isCreating = useControlPlaneStore((state) => state.isChangeRiskEvaluationCreating)
  const isReviewLoading = useControlPlaneStore((state) => state.isChangeRiskReviewLoading)
  const isReviewUpdating = useControlPlaneStore((state) => state.isChangeRiskReviewUpdating)
  const error = useControlPlaneStore((state) => state.changeRiskError)
  const displayTimezone = useControlPlaneStore((state) => state.displayTimezone)
  const loadEvaluations = useControlPlaneStore((state) => state.loadChangeRiskEvaluations)
  const loadRules = useControlPlaneStore((state) => state.loadChangeRiskRules)
  const getEvaluation = useControlPlaneStore((state) => state.getChangeRiskEvaluation)
  const loadTrace = useControlPlaneStore((state) => state.loadChangeRiskEvaluationTrace)
  const loadReview = useControlPlaneStore((state) => state.loadChangeRiskEvaluationReview)
  const updateReview = useControlPlaneStore((state) => state.updateChangeRiskEvaluationReview)
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
    void loadRules()
  }, [branch, loadDeploymentGateAuthorizations, loadEvaluations, loadRules, repositoryFullName, selectedOrgName])

  useEffect(() => {
    void loadEvaluations({ org_name: selectedOrgName || null, limit: 10, offset: 0 })
    void loadRules()
  }, [loadEvaluations, loadRules, selectedOrgName])

  useEffect(() => {
    if (!selectedEvaluation?.evaluation_id) return
    void loadTrace(selectedEvaluation.evaluation_id, { org_name: selectedOrgName || null })
    void loadReview(selectedEvaluation.evaluation_id, { org_name: selectedOrgName || null })
  }, [loadReview, loadTrace, selectedEvaluation?.evaluation_id, selectedOrgName])

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
            <EvaluationSummary
              evaluation={selectedEvaluation}
              trace={evaluationTrace}
              catalog={ruleCatalog}
              isTraceLoading={isTraceLoading}
              review={evaluationReview}
              isReviewLoading={isReviewLoading}
              isReviewUpdating={isReviewUpdating}
              displayTimezone={displayTimezone}
              onReloadTrace={() => void loadTrace(selectedEvaluation.evaluation_id, { org_name: selectedOrgName || null })}
              onReloadReview={() => void loadReview(selectedEvaluation.evaluation_id, { org_name: selectedOrgName || null })}
              onSubmitReview={(payload) => void updateReview(selectedEvaluation.evaluation_id, {
                org_name: selectedOrgName || null,
                ...payload,
              })}
            />
          ) : (
            <div className="rounded-lg border border-white/8 bg-surface-900/60 p-4 text-xs text-surface-500">
              No assessment selected yet.
            </div>
          )}
          <div className="rounded-lg border border-white/8 bg-surface-900/60">
            <div className="flex items-center justify-between border-b border-white/6 px-3 py-2">
              <span className="text-[11px] font-medium text-surface-300">Recent risk assessments</span>
              <span className="text-[10px] text-surface-600">{isRulesLoading ? 'rules loading' : `${evaluationsTotal} total`}</span>
            </div>
            <div className="max-h-[340px] overflow-auto divide-y divide-white/6">
              {evaluations.map((evaluation) => (
                <button
                  type="button"
                  key={evaluation.evaluation_id}
                  className="block w-full p-3 text-left text-xs transition-colors hover:bg-white/5"
                  onClick={() => void getEvaluation(evaluation.evaluation_id, { org_name: selectedOrgName || null })}
                >
                  <div className="flex flex-wrap items-center gap-2">
                    <Badge variant={riskVariant(evaluation.risk_level)}>{evaluation.risk_level}</Badge>
                    <span className="font-medium text-surface-100">{evaluation.release_id || 'release not set'}</span>
                    <span className="text-surface-500">{evaluation.environment || 'environment not set'}</span>
                    <Badge variant={reviewVariant(evaluation.review_status || 'needs_review')}>{reviewLabel(evaluation.review_status || 'needs_review')}</Badge>
                  </div>
                  <div className="mt-2 grid grid-cols-1 gap-1 text-[11px] text-surface-400 md:grid-cols-2">
                    <span className="truncate">Repo: <span className="text-surface-200">{evaluation.repository_full_name || 'Not set'}</span></span>
                    <span className="truncate">Branch: <span className="text-surface-200">{evaluation.branch || 'Not set'}</span></span>
                    <span className="truncate">Gate: <span className="font-mono text-surface-200">{evaluation.deployment_gate_id || 'Not linked'}</span></span>
                    <span>Created: <span className="text-surface-200">{formatTs(evaluation.created_at, displayTimezone)}</span></span>
                  </div>
                  <div className="mt-2 text-[11px] text-surface-500">
                    {evaluation.triggered_rules?.slice(0, 2).join(', ') || evaluation.risk_reasons.slice(0, 2).join(', ') || 'No reasons recorded.'}
                  </div>
                </button>
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
