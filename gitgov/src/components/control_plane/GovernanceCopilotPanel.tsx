import { useMemo, useState } from 'react'
import { Bot, Send, Sparkles } from 'lucide-react'
import { Badge } from '@/components/shared/Badge'
import { Button } from '@/components/shared/Button'
import { useControlPlaneStore, type GovernanceCopilotCitation } from '@/store/useControlPlaneStore'

function statusVariant(status: string): 'success' | 'warning' | 'danger' | 'neutral' | 'info' {
  if (status === 'ok') return 'success'
  if (status === 'missing' || status === 'skipped') return 'warning'
  if (status === 'error') return 'danger'
  return 'neutral'
}

function fieldValue(value?: string | null): string {
  return value?.trim() ?? ''
}

function normalizeTicketId(value: string): string {
  return value.trim().toUpperCase()
}

function CitationPill({ citation }: { citation: GovernanceCopilotCitation }) {
  return (
    <span className="inline-flex items-center gap-1 rounded border border-white/8 bg-white/[0.03] px-2 py-1 text-[10px] text-surface-300">
      <span className="font-mono text-brand-300">{citation.id}</span>
      <Badge variant={statusVariant(citation.status)} className="text-[9px]">
        {citation.status}
      </Badge>
    </span>
  )
}

export function GovernanceCopilotPanel() {
  const isConnected = useControlPlaneStore((s) => s.isConnected)
  const userRole = useControlPlaneStore((s) => s.userRole)
  const selectedOrgName = useControlPlaneStore((s) => s.selectedOrgName)
  const jiraCoverageFilters = useControlPlaneStore((s) => s.jiraCoverageFilters)
  const enterpriseAdoptionProfile = useControlPlaneStore((s) => s.enterpriseAdoptionProfile)
  const evidencePacketTicketId = useControlPlaneStore((s) => s.evidencePacketTicketId)
  const response = useControlPlaneStore((s) => s.governanceCopilotResponse)
  const isLoading = useControlPlaneStore((s) => s.isGovernanceCopilotLoading)
  const error = useControlPlaneStore((s) => s.governanceCopilotError)
  const askGovernanceCopilot = useControlPlaneStore((s) => s.askGovernanceCopilot)

  const [question, setQuestion] = useState('¿Está listo este release para producción?')
  const [ticketId, setTicketId] = useState('')
  const [releaseId, setReleaseId] = useState('')
  const [orgName, setOrgName] = useState('')
  const [repositoryFullName, setRepositoryFullName] = useState('')
  const [branch, setBranch] = useState('')
  const [environment, setEnvironment] = useState('production')
  const [hours, setHours] = useState(String(jiraCoverageFilters.hours || 720))

  const effectiveOrgName = fieldValue(orgName) || fieldValue(selectedOrgName)
  const effectiveRepositoryFullName =
    fieldValue(repositoryFullName) ||
    fieldValue(enterpriseAdoptionProfile?.repository_full_name) ||
    fieldValue(jiraCoverageFilters.repo_full_name)
  const effectiveBranch =
    fieldValue(branch) ||
    fieldValue(enterpriseAdoptionProfile?.default_branch) ||
    fieldValue(jiraCoverageFilters.branch)
  const normalizedTicketId = normalizeTicketId(ticketId || evidencePacketTicketId)
  const parsedHours = useMemo(() => {
    const parsed = Number.parseInt(hours, 10)
    if (!Number.isFinite(parsed)) return 720
    return Math.min(Math.max(parsed, 1), 8784)
  }, [hours])
  const canAsk = question.trim().length > 0 && !isLoading

  const handleAsk = async () => {
    if (!canAsk) return
    await askGovernanceCopilot({
      question,
      org_name: effectiveOrgName || null,
      repository_full_name: effectiveRepositoryFullName || null,
      branch: effectiveBranch || null,
      ticket_id: normalizedTicketId || null,
      release_id: fieldValue(releaseId) || normalizedTicketId || null,
      environment: fieldValue(environment) || null,
      hours: parsedHours,
    })
  }

  if (!isConnected || userRole !== 'Admin') return null

  return (
    <section className="glass-panel p-5">
      <div className="card-header mb-4">
        <div>
          <div className="flex items-center gap-2">
            <Bot size={16} className="text-brand-400" />
            <h2>Governance Copilot</h2>
            <Badge variant={response?.mode === 'ai' ? 'success' : 'info'}>
              {response?.mode === 'ai' ? 'AI' : 'Brief'}
            </Badge>
          </div>
          <p>Evidence-grounded readiness answers with GitGov citations.</p>
        </div>
        <Button
          size="sm"
          variant="primary"
          loading={isLoading}
          disabled={!canAsk}
          onClick={() => void handleAsk()}
          title="Ask governance copilot"
        >
          <Send size={14} />
          Ask
        </Button>
      </div>

      <div className="grid grid-cols-1 xl:grid-cols-[minmax(0,0.85fr)_minmax(0,1.15fr)] gap-4">
        <div className="space-y-3">
          <div className="flex flex-col gap-1">
            <label htmlFor="governance-copilot-question" className="text-[10px] text-surface-500">
              Question
            </label>
            <textarea
              id="governance-copilot-question"
              value={question}
              onChange={(event) => setQuestion(event.target.value)}
              rows={4}
              maxLength={2000}
              className="min-h-[96px] resize-y rounded border border-surface-600 bg-surface-800 px-3 py-2 text-xs text-surface-200 focus:border-surface-400 focus:outline-none"
            />
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
            <label className="flex flex-col gap-1 text-[10px] text-surface-500">
              Org
              <input
                value={orgName}
                onChange={(event) => setOrgName(event.target.value)}
                placeholder={effectiveOrgName || 'org-name'}
                className="rounded border border-surface-600 bg-surface-800 px-2 py-1.5 text-xs text-surface-200 focus:border-surface-400 focus:outline-none"
              />
            </label>
            <label className="flex flex-col gap-1 text-[10px] text-surface-500">
              Ticket
              <input
                value={ticketId}
                onChange={(event) => setTicketId(event.target.value)}
                placeholder={evidencePacketTicketId || 'KAN-37'}
                className="rounded border border-surface-600 bg-surface-800 px-2 py-1.5 text-xs text-surface-200 focus:border-surface-400 focus:outline-none"
              />
            </label>
            <label className="flex flex-col gap-1 text-[10px] text-surface-500">
              Release
              <input
                value={releaseId}
                onChange={(event) => setReleaseId(event.target.value)}
                placeholder={normalizedTicketId || 'release-id'}
                className="rounded border border-surface-600 bg-surface-800 px-2 py-1.5 text-xs text-surface-200 focus:border-surface-400 focus:outline-none"
              />
            </label>
            <label className="flex flex-col gap-1 text-[10px] text-surface-500">
              Repository
              <input
                value={repositoryFullName}
                onChange={(event) => setRepositoryFullName(event.target.value)}
                placeholder={effectiveRepositoryFullName || 'owner/repo'}
                className="rounded border border-surface-600 bg-surface-800 px-2 py-1.5 text-xs text-surface-200 focus:border-surface-400 focus:outline-none"
              />
            </label>
            <label className="flex flex-col gap-1 text-[10px] text-surface-500">
              Branch
              <input
                value={branch}
                onChange={(event) => setBranch(event.target.value)}
                placeholder={effectiveBranch || 'main'}
                className="rounded border border-surface-600 bg-surface-800 px-2 py-1.5 text-xs text-surface-200 focus:border-surface-400 focus:outline-none"
              />
            </label>
            <label className="flex flex-col gap-1 text-[10px] text-surface-500">
              Environment
              <select
                value={environment}
                onChange={(event) => setEnvironment(event.target.value)}
                className="rounded border border-surface-600 bg-surface-800 px-2 py-1.5 text-xs text-surface-200 focus:border-surface-400 focus:outline-none"
              >
                <option value="production">production</option>
                <option value="staging">staging</option>
                <option value="development">development</option>
              </select>
            </label>
            <label className="flex flex-col gap-1 text-[10px] text-surface-500">
              Hours
              <input
                type="number"
                min={1}
                max={8784}
                value={hours}
                onChange={(event) => setHours(event.target.value)}
                className="rounded border border-surface-600 bg-surface-800 px-2 py-1.5 text-xs text-surface-200 focus:border-surface-400 focus:outline-none"
              />
            </label>
          </div>

          {error && (
            <div className="rounded border border-danger-500/20 bg-danger-500/8 p-3 text-xs text-danger-200">
              {error}
            </div>
          )}
        </div>

        <div className="min-h-[300px] rounded-lg border border-white/8 bg-surface-900/60 p-4">
          {response ? (
            <div className="space-y-4">
              <div className="flex flex-wrap items-center gap-2">
                <Badge variant={response.mode === 'ai' ? 'success' : 'info'}>
                  {response.mode ?? 'fallback'}
                </Badge>
                {response.model && <Badge variant="neutral">{response.model}</Badge>}
                <span className="text-[10px] text-surface-500">
                  {response.sources.length} sources · {response.citations.length} citations
                </span>
              </div>

              <p className="whitespace-pre-wrap text-[12px] leading-relaxed text-surface-200">
                {response.answer}
              </p>

              {response.citations.length > 0 && (
                <div className="flex flex-wrap gap-1.5">
                  {response.citations.map((citation) => (
                    <CitationPill key={citation.id} citation={citation} />
                  ))}
                </div>
              )}

              {response.warnings.length > 0 && (
                <div className="rounded border border-warning-500/20 bg-warning-500/8 p-3">
                  <div className="mb-2 flex items-center gap-2 text-[10px] uppercase tracking-widest text-warning-300">
                    <Sparkles size={12} />
                    Warnings
                  </div>
                  <ul className="space-y-1 text-[11px] text-warning-100">
                    {response.warnings.map((warning) => (
                      <li key={warning}>{warning}</li>
                    ))}
                  </ul>
                </div>
              )}

              <div className="overflow-hidden rounded border border-white/6">
                <table className="w-full text-left text-[11px]">
                  <thead className="bg-white/[0.03] text-[9px] uppercase tracking-widest text-surface-500">
                    <tr>
                      <th className="px-3 py-2 font-medium">Source</th>
                      <th className="px-3 py-2 font-medium">Status</th>
                      <th className="px-3 py-2 font-medium">Endpoint</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-white/5">
                    {response.sources.map((source) => (
                      <tr key={source.id}>
                        <td className="px-3 py-2 text-surface-200">{source.label || source.id}</td>
                        <td className="px-3 py-2">
                          <Badge variant={statusVariant(source.status)}>{source.status}</Badge>
                        </td>
                        <td className="px-3 py-2 font-mono text-[10px] text-surface-500">
                          {source.endpoint}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          ) : (
            <div className="flex h-full min-h-[260px] items-center justify-center text-center text-xs text-surface-600">
              <div>
                <Bot size={22} className="mx-auto mb-2 text-surface-500" />
                <p>No copilot answer yet.</p>
              </div>
            </div>
          )}
        </div>
      </div>
    </section>
  )
}
