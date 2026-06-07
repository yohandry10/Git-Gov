import { useState } from 'react'
import { Download, FileCheck2, Search } from 'lucide-react'
import { Button } from '@/components/shared/Button'
import { formatTs } from '@/lib/timezone'
import { useControlPlaneStore } from '@/store/useControlPlaneStore'

function downloadJson(filename: string, data: unknown) {
  const content = JSON.stringify(data, null, 2)
  const blob = new Blob([content], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = filename
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
}

function safeDownloadName(value: string): string {
  return value.trim().replace(/[^A-Za-z0-9._-]/g, '_').slice(0, 80) || 'ticket'
}

function statusLabel(missing: string[]): string {
  if (missing.length === 0) return 'Completo'
  if (missing.length <= 2) return 'Parcial'
  return 'Incompleto'
}

export function EvidencePacketPanel() {
  const jiraCoverageFilters = useControlPlaneStore((s) => s.jiraCoverageFilters)
  const evidencePacket = useControlPlaneStore((s) => s.evidencePacket)
  const evidencePacketTicketId = useControlPlaneStore((s) => s.evidencePacketTicketId)
  const isEvidencePacketLoading = useControlPlaneStore((s) => s.isEvidencePacketLoading)
  const loadTicketEvidencePacket = useControlPlaneStore((s) => s.loadTicketEvidencePacket)
  const displayTimezone = useControlPlaneStore((s) => s.displayTimezone)
  const [ticketId, setTicketId] = useState(evidencePacketTicketId || 'KAN-23')

  const normalizedTicket = ticketId.trim().toUpperCase()
  const handleGenerate = async () => {
    await loadTicketEvidencePacket(normalizedTicket, {
      hours: jiraCoverageFilters.hours,
      repo_full_name: jiraCoverageFilters.repo_full_name.trim() || undefined,
      branch: jiraCoverageFilters.branch.trim() || undefined,
    })
  }

  const handleDownload = () => {
    if (!evidencePacket) return
    downloadJson(`gitgov-evidence-packet-${safeDownloadName(evidencePacket.subject)}.json`, evidencePacket)
  }

  const completeness = evidencePacket?.completeness
  const packetStatus = completeness ? statusLabel(completeness.missing) : 'Sin generar'

  return (
    <div id="evidence-packet" className="glass-panel p-5 scroll-mt-4">
      <div className="flex items-center justify-between gap-3 mb-4">
        <div className="flex items-center gap-2 min-w-0">
          <FileCheck2 size={14} className="text-surface-400 shrink-0" />
          <span className="card-header">Evidence Packet</span>
        </div>
        <span className="text-[10px] text-surface-500 uppercase tracking-widest">
          {packetStatus}
        </span>
      </div>

      <div className="flex flex-wrap gap-2 items-end mb-4">
        <div className="flex flex-col gap-1 min-w-[160px] flex-1">
          <label htmlFor="evidence-ticket-id" className="text-[10px] text-surface-500">Ticket</label>
          <input
            id="evidence-ticket-id"
            value={ticketId}
            onChange={(event) => setTicketId(event.target.value)}
            placeholder="KAN-23"
            className="bg-surface-800 border border-surface-600 rounded px-2 py-1.5 text-xs text-surface-200 focus:outline-none focus:border-surface-400"
          />
        </div>
        <Button
          variant="secondary"
          size="sm"
          loading={isEvidencePacketLoading}
          disabled={!normalizedTicket}
          onClick={() => void handleGenerate()}
          className="flex items-center gap-1.5"
        >
          <Search size={12} />
          Generar
        </Button>
        <Button
          variant="outline"
          size="sm"
          disabled={!evidencePacket}
          onClick={handleDownload}
          className="flex items-center gap-1.5"
        >
          <Download size={12} />
          JSON
        </Button>
      </div>

      {evidencePacket ? (
        <div className="space-y-3">
          <div className="grid grid-cols-2 md:grid-cols-5 gap-2 text-xs">
            <div className="rounded border border-white/6 bg-white/3 p-2">
              <div className="text-surface-500 text-[10px]">Commits</div>
              <div className="mono-data text-surface-100">{completeness?.commits ?? 0}</div>
            </div>
            <div className="rounded border border-white/6 bg-white/3 p-2">
              <div className="text-surface-500 text-[10px]">PRs</div>
              <div className="mono-data text-surface-100">{completeness?.pull_requests ?? 0}</div>
            </div>
            <div className="rounded border border-white/6 bg-white/3 p-2">
              <div className="text-surface-500 text-[10px]">Pipelines</div>
              <div className="mono-data text-surface-100">{completeness?.pipelines ?? 0}</div>
            </div>
            <div className="rounded border border-white/6 bg-white/3 p-2">
              <div className="text-surface-500 text-[10px]">Quality gates</div>
              <div className="mono-data text-surface-100">{completeness?.quality_gates ?? 0}</div>
            </div>
            <div className="rounded border border-white/6 bg-white/3 p-2">
              <div className="text-surface-500 text-[10px]">Ticket</div>
              <div className="mono-data text-surface-100">{completeness?.ticket_found ? 'ok' : 'missing'}</div>
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-2 text-[11px]">
            <div className="text-surface-400">
              Generado: <span className="text-surface-200">{formatTs(evidencePacket.generated_at, displayTimezone)}</span>
            </div>
            <div className="text-surface-400">
              Ventana: <span className="text-surface-200">{evidencePacket.period}</span>
            </div>
            <div className="text-surface-400 truncate" title={evidencePacket.content_hash}>
              Hash: <span className="text-surface-200 mono-data">{evidencePacket.content_hash.slice(0, 12)}</span>
            </div>
          </div>

          {evidencePacket.completeness.missing.length > 0 && (
            <p className="text-[10px] text-warning-400">
              Falta evidencia: {evidencePacket.completeness.missing.join(', ')}
            </p>
          )}
        </div>
      ) : (
        <p className="text-xs text-surface-600 py-2">
          Genera un packet para descargar evidencia auditable por ticket.
        </p>
      )}
    </div>
  )
}
