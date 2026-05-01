import { useEffect, useMemo, useState } from 'react'
import { CheckCircle2, ClipboardCheck, RefreshCw, ShieldAlert } from 'lucide-react'
import { Badge } from '@/components/shared/Badge'
import { Button } from '@/components/shared/Button'
import { formatTs } from '@/lib/timezone'
import {
  useControlPlaneStore,
  type CreateEnterpriseReleaseApprovalRequest,
  type EnterpriseReleaseApprovalDecision,
  type EnterpriseReleaseApprovalRiskSeverity,
} from '@/store/useControlPlaneStore'

const DECISIONS: EnterpriseReleaseApprovalDecision[] = ['approved', 'rejected', 'accepted-risk']
const RISK_SEVERITIES: EnterpriseReleaseApprovalRiskSeverity[] = ['none', 'low', 'medium', 'high', 'critical']
const HEX_64_RE = /^[a-fA-F0-9]{64}$/
const SHA_RE = /^[a-fA-F0-9]{7,64}$/
const TICKET_RE = /^[A-Z][A-Z0-9]+-[1-9][0-9]*$/

interface ApprovalForm {
  releaseId: string
  repositoryFullName: string
  branch: string
  targetSha: string
  environment: string
  decision: EnterpriseReleaseApprovalDecision
  approver: string
  approverRole: string
  ticketId: string
  evidencePacketHash: string
  evidencePacketUri: string
  riskSeverity: EnterpriseReleaseApprovalRiskSeverity
  riskAcceptanceReason: string
  expiresInDays: string
  operatorConfirmed: boolean
}

function decisionVariant(decision: string): 'success' | 'warning' | 'danger' | 'neutral' {
  if (decision === 'approved') return 'success'
  if (decision === 'accepted-risk') return 'warning'
  if (decision === 'rejected') return 'danger'
  return 'neutral'
}

function governanceStatusVariant(status: string): 'success' | 'warning' | 'danger' | 'info' | 'neutral' {
  if (status === 'approved') return 'success'
  if (status === 'blocked') return 'danger'
  if (status === 'advisory-warning' || status === 'would-block') return 'warning'
  if (status === 'recorded') return 'info'
  return 'neutral'
}

function sanitizeOptional(value: string): string | null {
  const trimmed = value.trim()
  return trimmed ? trimmed : null
}

function isValidRepo(value: string): boolean {
  const parts = value.trim().split('/')
  return parts.length === 2 && parts.every((part) => /^[A-Za-z0-9_.-]+$/.test(part))
}

function isValidEvidenceUri(value: string): boolean {
  const trimmed = value.trim()
  if (!trimmed) return true
  if (/\s/.test(trimmed)) return false
  return trimmed.startsWith('/') || trimmed.startsWith('http://') || trimmed.startsWith('https://')
}

function validateApprovalForm(form: ApprovalForm): string[] {
  const errors: string[] = []
  if (!form.releaseId.trim()) errors.push('Release is required.')
  if (!isValidRepo(form.repositoryFullName)) errors.push('Repository must look like owner/repo.')
  if (!form.environment.trim()) errors.push('Environment is required.')
  if (!form.approver.trim()) errors.push('Approver is required.')
  if (form.approverRole.trim() && !/^[A-Za-z0-9_.-]{1,64}$/.test(form.approverRole.trim())) {
    errors.push('Approver role must be 1 to 64 letters, numbers, dots, underscores, or dashes.')
  }
  if (!HEX_64_RE.test(form.evidencePacketHash.trim())) errors.push('Evidence hash must be a 64 character SHA-256 hex value.')
  if (form.targetSha.trim() && !SHA_RE.test(form.targetSha.trim())) errors.push('Target SHA must be 7 to 64 hex characters.')
  if (form.ticketId.trim() && !TICKET_RE.test(form.ticketId.trim().toUpperCase())) errors.push('Ticket must look like KAN-43.')
  if (!isValidEvidenceUri(form.evidencePacketUri)) errors.push('Evidence URI must be a relative API path or http(s) URL.')
  if (form.decision === 'approved' && ['high', 'critical'].includes(form.riskSeverity)) {
    errors.push('High or critical risk cannot be approved directly.')
  }
  if (form.decision === 'accepted-risk') {
    const expiresInDays = Number.parseInt(form.expiresInDays, 10)
    if (form.riskSeverity === 'none') errors.push('Accepted risk requires a non-none risk severity.')
    if (!form.riskAcceptanceReason.trim()) errors.push('Accepted risk requires a reason.')
    if (!Number.isFinite(expiresInDays) || expiresInDays < 1 || expiresInDays > 366) {
      errors.push('Accepted risk expiration must be 1 to 366 days.')
    }
  }
  if (!form.operatorConfirmed) errors.push('Confirm the evidence and decision before submitting.')
  return errors
}

function toCreatePayload(form: ApprovalForm, orgName: string): CreateEnterpriseReleaseApprovalRequest {
  const expiresInDays = Number.parseInt(form.expiresInDays, 10)
  const expiresAt =
    form.decision === 'accepted-risk' && Number.isFinite(expiresInDays)
      ? Date.now() + expiresInDays * 24 * 60 * 60 * 1000
      : null

  return {
    org_name: sanitizeOptional(orgName),
    release_id: form.releaseId.trim(),
    repository_full_name: form.repositoryFullName.trim(),
    branch: sanitizeOptional(form.branch),
    target_sha: sanitizeOptional(form.targetSha),
    environment: form.environment.trim(),
    decision: form.decision,
    approver: form.approver.trim(),
    ticket_id: sanitizeOptional(form.ticketId.toUpperCase()),
    evidence_packet_hash: form.evidencePacketHash.trim(),
    evidence_packet_uri: sanitizeOptional(form.evidencePacketUri),
    evidence_summary: {
      source: 'dashboard-release-approval-wizard',
      approver_role: sanitizeOptional(form.approverRole.toLowerCase()),
      ticket_id: sanitizeOptional(form.ticketId.toUpperCase()),
      evidence_packet_uri: sanitizeOptional(form.evidencePacketUri),
    },
    risk_severity: form.riskSeverity,
    risk_acceptance_reason: form.decision === 'accepted-risk' ? sanitizeOptional(form.riskAcceptanceReason) : null,
    expires_at: expiresAt,
  }
}

export function ReleaseApprovalPanel() {
  const selectedOrgName = useControlPlaneStore((state) => state.selectedOrgName)
  const jiraCoverageFilters = useControlPlaneStore((state) => state.jiraCoverageFilters)
  const evidencePacket = useControlPlaneStore((state) => state.evidencePacket)
  const evidencePacketTicketId = useControlPlaneStore((state) => state.evidencePacketTicketId)
  const enterpriseAdoptionProfile = useControlPlaneStore((state) => state.enterpriseAdoptionProfile)
  const approvals = useControlPlaneStore((state) => state.releaseApprovals)
  const approvalsTotal = useControlPlaneStore((state) => state.releaseApprovalsTotal)
  const isLoading = useControlPlaneStore((state) => state.isReleaseApprovalsLoading)
  const isSubmitting = useControlPlaneStore((state) => state.isReleaseApprovalSubmitting)
  const isEvaluating = useControlPlaneStore((state) => state.isReleaseGovernanceEvaluating)
  const error = useControlPlaneStore((state) => state.releaseApprovalError)
  const governanceEvaluation = useControlPlaneStore((state) => state.releaseGovernanceEvaluation)
  const displayTimezone = useControlPlaneStore((state) => state.displayTimezone)
  const loadApprovals = useControlPlaneStore((state) => state.loadEnterpriseReleaseApprovals)
  const evaluateGovernance = useControlPlaneStore((state) => state.evaluateEnterpriseReleaseGovernance)
  const createApproval = useControlPlaneStore((state) => state.createEnterpriseReleaseApproval)

  const defaultRepository =
    enterpriseAdoptionProfile?.repository_full_name ||
    jiraCoverageFilters.repo_full_name ||
    'yohandry10/Git-Gov'
  const defaultBranch =
    enterpriseAdoptionProfile?.default_branch ||
    jiraCoverageFilters.branch ||
    'main'
  const defaultTicket = evidencePacket?.subject || evidencePacketTicketId || 'KAN-43'
  const defaultEvidenceUri = `/evidence/packets/tickets/${defaultTicket}`

  const [form, setForm] = useState<ApprovalForm>({
    releaseId: defaultTicket,
    repositoryFullName: defaultRepository,
    branch: defaultBranch,
    targetSha: '',
    environment: 'production',
    decision: 'approved',
    approver: '',
    approverRole: 'engineering',
    ticketId: defaultTicket,
    evidencePacketHash: evidencePacket?.content_hash ?? '',
    evidencePacketUri: defaultEvidenceUri,
    riskSeverity: 'none',
    riskAcceptanceReason: '',
    expiresInDays: '30',
    operatorConfirmed: false,
  })

  useEffect(() => {
    void loadApprovals({
      org_name: selectedOrgName || null,
      repository_full_name: defaultRepository || null,
      limit: 10,
      offset: 0,
    })
  }, [defaultRepository, loadApprovals, selectedOrgName])

  const validationErrors = useMemo(() => validateApprovalForm(form), [form])
  const canSubmit = validationErrors.length === 0 && !isSubmitting
  const canEvaluate = Boolean(isValidRepo(form.repositoryFullName) && form.releaseId.trim() && form.environment.trim())

  const updateForm = <K extends keyof ApprovalForm>(field: K, value: ApprovalForm[K]) => {
    setForm((current) => ({ ...current, [field]: value }))
  }

  const applyCurrentEvidencePacket = () => {
    if (!evidencePacket) return
    setForm((current) => ({
      ...current,
      releaseId: evidencePacket.subject,
      repositoryFullName: evidencePacket.repo_full_name || current.repositoryFullName || defaultRepository,
      branch: evidencePacket.branch || current.branch || defaultBranch,
      ticketId: evidencePacket.subject,
      evidencePacketHash: evidencePacket.content_hash,
      evidencePacketUri: `/evidence/packets/tickets/${evidencePacket.subject}`,
      operatorConfirmed: false,
    }))
  }

  const evaluateCurrentRelease = async () => {
    if (!canEvaluate) return
    await evaluateGovernance({
      org_name: selectedOrgName || null,
      repository_full_name: form.repositoryFullName,
      release_id: form.releaseId,
      environment: form.environment,
      evidence_packet_hash: HEX_64_RE.test(form.evidencePacketHash.trim()) ? form.evidencePacketHash.trim() : null,
    })
  }

  const handleSubmit = async () => {
    if (!canSubmit) return
    const created = await createApproval(toCreatePayload(form, selectedOrgName))
    if (!created) return
    await evaluateCurrentRelease()
    setForm((current) => ({
      ...current,
      operatorConfirmed: false,
      riskAcceptanceReason: '',
    }))
  }

  return (
    <section className="glass-panel p-5">
      <div className="card-header mb-4">
        <div>
          <div className="flex items-center gap-2">
            <ClipboardCheck size={16} className="text-brand-400" />
            <h2>Release Approvals</h2>
            <Badge variant={approvals.length > 0 ? 'success' : 'info'}>{approvals.length}/{approvalsTotal}</Badge>
          </div>
          <p>Formal release decisions with evidence hash, approver, risk context, and expiration.</p>
        </div>
        <Button
          size="sm"
          variant="outline"
          loading={isLoading}
          onClick={() => void loadApprovals({ org_name: selectedOrgName || null, repository_full_name: form.repositoryFullName, limit: 10, offset: 0 })}
          title="Refresh release approvals"
        >
          <RefreshCw size={14} />
          Refresh
        </Button>
      </div>

      {error && (
        <div className="mb-4 rounded border border-danger-500/20 bg-danger-500/8 p-3 text-xs text-danger-200">
          {error}
        </div>
      )}

      <div className="grid grid-cols-1 xl:grid-cols-[minmax(0,0.95fr)_minmax(0,1.05fr)] gap-4">
        <div className="space-y-3">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
            <label className="flex flex-col gap-1 text-[10px] text-surface-500">
              Release
              <input value={form.releaseId} onChange={(event) => updateForm('releaseId', event.target.value)} className="rounded border border-surface-600 bg-surface-800 px-2 py-1.5 text-xs text-surface-200 focus:border-surface-400 focus:outline-none" />
            </label>
            <label className="flex flex-col gap-1 text-[10px] text-surface-500">
              Ticket
              <input value={form.ticketId} onChange={(event) => updateForm('ticketId', event.target.value.toUpperCase())} className="rounded border border-surface-600 bg-surface-800 px-2 py-1.5 text-xs text-surface-200 focus:border-surface-400 focus:outline-none" />
            </label>
            <label className="flex flex-col gap-1 text-[10px] text-surface-500">
              Repository
              <input value={form.repositoryFullName} onChange={(event) => updateForm('repositoryFullName', event.target.value)} className="rounded border border-surface-600 bg-surface-800 px-2 py-1.5 text-xs text-surface-200 focus:border-surface-400 focus:outline-none" />
            </label>
            <label className="flex flex-col gap-1 text-[10px] text-surface-500">
              Branch
              <input value={form.branch} onChange={(event) => updateForm('branch', event.target.value)} className="rounded border border-surface-600 bg-surface-800 px-2 py-1.5 text-xs text-surface-200 focus:border-surface-400 focus:outline-none" />
            </label>
            <label className="flex flex-col gap-1 text-[10px] text-surface-500">
              Target SHA
              <input value={form.targetSha} onChange={(event) => updateForm('targetSha', event.target.value)} placeholder="optional" className="rounded border border-surface-600 bg-surface-800 px-2 py-1.5 text-xs text-surface-200 focus:border-surface-400 focus:outline-none" />
            </label>
            <label className="flex flex-col gap-1 text-[10px] text-surface-500">
              Environment
              <input value={form.environment} onChange={(event) => updateForm('environment', event.target.value)} className="rounded border border-surface-600 bg-surface-800 px-2 py-1.5 text-xs text-surface-200 focus:border-surface-400 focus:outline-none" />
            </label>
            <label className="flex flex-col gap-1 text-[10px] text-surface-500">
              Decision
              <select value={form.decision} onChange={(event) => updateForm('decision', event.target.value as EnterpriseReleaseApprovalDecision)} className="rounded border border-surface-600 bg-surface-800 px-2 py-1.5 text-xs text-surface-200 focus:border-surface-400 focus:outline-none">
                {DECISIONS.map((decision) => <option key={decision} value={decision}>{decision}</option>)}
              </select>
            </label>
            <label className="flex flex-col gap-1 text-[10px] text-surface-500">
              Risk
              <select value={form.riskSeverity} onChange={(event) => updateForm('riskSeverity', event.target.value as EnterpriseReleaseApprovalRiskSeverity)} className="rounded border border-surface-600 bg-surface-800 px-2 py-1.5 text-xs text-surface-200 focus:border-surface-400 focus:outline-none">
                {RISK_SEVERITIES.map((severity) => <option key={severity} value={severity}>{severity}</option>)}
              </select>
            </label>
            <label className="flex flex-col gap-1 text-[10px] text-surface-500 md:col-span-2">
              Approver
              <input value={form.approver} onChange={(event) => updateForm('approver', event.target.value)} placeholder="release.manager@example.com" className="rounded border border-surface-600 bg-surface-800 px-2 py-1.5 text-xs text-surface-200 focus:border-surface-400 focus:outline-none" />
            </label>
            <label className="flex flex-col gap-1 text-[10px] text-surface-500 md:col-span-2">
              Approver role
              <input value={form.approverRole} onChange={(event) => updateForm('approverRole', event.target.value.toLowerCase())} placeholder="engineering" className="rounded border border-surface-600 bg-surface-800 px-2 py-1.5 text-xs text-surface-200 focus:border-surface-400 focus:outline-none" />
            </label>
            <label className="flex flex-col gap-1 text-[10px] text-surface-500 md:col-span-2">
              Evidence hash
              <input value={form.evidencePacketHash} onChange={(event) => updateForm('evidencePacketHash', event.target.value)} className="font-mono rounded border border-surface-600 bg-surface-800 px-2 py-1.5 text-[11px] text-surface-200 focus:border-surface-400 focus:outline-none" />
            </label>
            <label className="flex flex-col gap-1 text-[10px] text-surface-500 md:col-span-2">
              Evidence URI
              <input value={form.evidencePacketUri} onChange={(event) => updateForm('evidencePacketUri', event.target.value)} className="rounded border border-surface-600 bg-surface-800 px-2 py-1.5 text-xs text-surface-200 focus:border-surface-400 focus:outline-none" />
            </label>
          </div>

          <Button size="sm" variant="outline" disabled={!evidencePacket} onClick={applyCurrentEvidencePacket} title="Use current evidence packet">
            <ClipboardCheck size={14} />
            Use current packet
          </Button>

          <Button size="sm" variant="outline" loading={isEvaluating} disabled={!canEvaluate} onClick={() => void evaluateCurrentRelease()} title="Evaluate release governance">
            <ShieldAlert size={14} />
            Evaluate governance
          </Button>

          {governanceEvaluation && (
            <div className="rounded border border-white/8 bg-white/[0.03] p-3 text-xs">
              <div className="flex flex-wrap items-center gap-2">
                <Badge variant={governanceStatusVariant(governanceEvaluation.status)}>{governanceEvaluation.status}</Badge>
                <span className="text-surface-300">
                  {governanceEvaluation.policy.mode} / {governanceEvaluation.policy.enforcement}
                </span>
                <span className="text-surface-500">
                  {governanceEvaluation.valid_approval_count}/{governanceEvaluation.required_approval_count} approvals
                </span>
              </div>
              <div className="mt-2 grid grid-cols-1 sm:grid-cols-3 gap-2 text-[11px] text-surface-400">
                <span>Applies: <span className="text-surface-200">{governanceEvaluation.policy.policy_applies ? 'yes' : 'no'}</span></span>
                <span>Blocking: <span className="text-surface-200">{governanceEvaluation.blocking ? 'yes' : 'no'}</span></span>
                <span>Would block: <span className="text-surface-200">{governanceEvaluation.would_block ? 'yes' : 'no'}</span></span>
              </div>
              {governanceEvaluation.policy.quorum_rules.length > 0 && (
                <div className="mt-2 flex flex-wrap gap-1">
                  {governanceEvaluation.policy.quorum_rules.map((rule) => (
                    <Badge key={rule.role} variant={rule.satisfied ? 'success' : 'warning'}>
                      {rule.role} {rule.observed}/{rule.required}
                    </Badge>
                  ))}
                </div>
              )}
              {governanceEvaluation.issues.length > 0 && (
                <ul className="mt-2 list-disc space-y-1 pl-4 text-[11px] text-warning-100">
                  {governanceEvaluation.issues.slice(0, 4).map((issue) => <li key={issue}>{issue}</li>)}
                </ul>
              )}
              {governanceEvaluation.next_steps.length > 0 && (
                <ul className="mt-2 list-disc space-y-1 pl-4 text-[11px] text-surface-400">
                  {governanceEvaluation.next_steps.slice(0, 3).map((step) => <li key={step}>{step}</li>)}
                </ul>
              )}
            </div>
          )}

          {form.decision === 'accepted-risk' && (
            <div className="grid grid-cols-1 md:grid-cols-[1fr_120px] gap-2 rounded border border-warning-500/20 bg-warning-500/8 p-3">
              <label className="flex flex-col gap-1 text-[10px] text-warning-100">
                Risk acceptance reason
                <textarea value={form.riskAcceptanceReason} onChange={(event) => updateForm('riskAcceptanceReason', event.target.value)} rows={3} maxLength={2000} className="resize-y rounded border border-warning-500/20 bg-surface-900 px-2 py-1.5 text-xs text-surface-200 focus:border-warning-400 focus:outline-none" />
              </label>
              <label className="flex flex-col gap-1 text-[10px] text-warning-100">
                Days
                <input type="number" min={1} max={366} value={form.expiresInDays} onChange={(event) => updateForm('expiresInDays', event.target.value)} className="rounded border border-warning-500/20 bg-surface-900 px-2 py-1.5 text-xs text-surface-200 focus:border-warning-400 focus:outline-none" />
              </label>
            </div>
          )}

          <label className="flex items-start gap-2 rounded border border-white/8 bg-white/[0.03] p-3 text-[11px] text-surface-300">
            <input type="checkbox" checked={form.operatorConfirmed} onChange={(event) => updateForm('operatorConfirmed', event.target.checked)} className="mt-0.5" />
            <span>I verified the evidence hash, decision, approver, and risk context for this release.</span>
          </label>

          {validationErrors.length > 0 && (
            <div className="rounded border border-warning-500/20 bg-warning-500/8 p-3 text-[11px] text-warning-100">
              <div className="mb-1 flex items-center gap-1 font-medium">
                <ShieldAlert size={13} />
                Needs review
              </div>
              <ul className="list-disc space-y-1 pl-4">
                {validationErrors.slice(0, 5).map((item) => <li key={item}>{item}</li>)}
              </ul>
            </div>
          )}

          <Button size="sm" variant="primary" loading={isSubmitting} disabled={!canSubmit} onClick={() => void handleSubmit()} title="Create release approval">
            <CheckCircle2 size={14} />
            Create approval
          </Button>
        </div>

        <div className="rounded-lg border border-white/8 bg-surface-900/60">
          <div className="flex items-center justify-between border-b border-white/6 px-3 py-2">
            <span className="text-[11px] font-medium text-surface-300">Recent decisions</span>
            <span className="text-[10px] text-surface-600">{approvalsTotal} total</span>
          </div>
          <div className="max-h-[520px] overflow-auto divide-y divide-white/6">
            {approvals.map((approval) => (
              <div key={approval.id} className="p-3 text-xs">
                <div className="flex flex-wrap items-center gap-2">
                  <Badge variant={decisionVariant(approval.decision)}>{approval.decision}</Badge>
                  <span className="font-medium text-surface-100">{approval.release_id}</span>
                  <span className="text-surface-500">{approval.environment}</span>
                </div>
                <div className="mt-2 grid grid-cols-1 md:grid-cols-2 gap-1 text-[11px] text-surface-400">
                  <span className="truncate">Repo: <span className="text-surface-200">{approval.repository_full_name}</span></span>
                  <span className="truncate">Approver: <span className="text-surface-200">{approval.approver}</span></span>
                  <span>Risk: <span className="text-surface-200">{approval.risk_severity}</span></span>
                  <span>Created: <span className="text-surface-200">{formatTs(approval.created_at, displayTimezone)}</span></span>
                </div>
                <div className="mt-2 truncate text-[10px] text-surface-500" title={approval.approval_hash}>
                  Approval hash: <span className="font-mono text-surface-300">{approval.approval_hash.slice(0, 16)}</span>
                </div>
                {approval.expires_at && (
                  <div className="mt-1 text-[10px] text-warning-300">
                    Risk expires: {formatTs(approval.expires_at, displayTimezone)}
                  </div>
                )}
              </div>
            ))}
            {approvals.length === 0 && (
              <div className="p-8 text-center text-xs text-surface-600">
                No release approvals in the current filter.
              </div>
            )}
          </div>
        </div>
      </div>
    </section>
  )
}
