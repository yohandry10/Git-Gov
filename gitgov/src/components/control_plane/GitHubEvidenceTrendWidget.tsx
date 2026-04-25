import { Badge } from '@/components/shared/Badge'
import type { GitHubEvidenceTrendPoint } from './dashboard-helpers'

interface GitHubEvidenceTrendWidgetProps {
  points: GitHubEvidenceTrendPoint[]
  onCapture: () => void
}

function formatShortTime(value: string): string {
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return '-'
  return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}

export function GitHubEvidenceTrendWidget({ points, onCapture }: GitHubEvidenceTrendWidgetProps) {
  const latest = points[points.length - 1]
  const oldest = points[0]
  const delta = latest && oldest ? latest.activeSignals - oldest.activeSignals : 0
  const badgeVariant =
    latest?.executiveStatus === 'Completo'
      ? 'success'
      : latest?.executiveStatus === 'Parcial'
        ? 'warning'
        : 'neutral'

  return (
    <div className="glass-panel p-4">
      <div className="flex items-center justify-between mb-3">
        <div>
          <div className="card-header">Trend evidencia GitHub</div>
          <p className="text-[10px] text-surface-500 mt-1">
            Historial local del dashboard; captura snapshots sin tokens de GitHub Actions.
          </p>
        </div>
        <div className="flex items-center gap-2">
          {latest && <Badge variant={badgeVariant}>{latest.executiveStatus}</Badge>}
          <button
            type="button"
            onClick={onCapture}
            className="text-[10px] text-brand-400 hover:text-brand-300 transition-colors"
          >
            Capturar
          </button>
        </div>
      </div>

      {latest ? (
        <div className="space-y-3">
          <div className="grid grid-cols-3 gap-2">
            <div className="rounded-lg border border-white/[0.06] bg-white/[0.03] px-3 py-2">
              <div className="text-[9px] uppercase tracking-wide text-surface-500">Actual</div>
              <div className="text-xs text-surface-200 mono-data font-semibold">
                {latest.activeSignals}/{latest.totalSignals}
              </div>
            </div>
            <div className="rounded-lg border border-white/[0.06] bg-white/[0.03] px-3 py-2">
              <div className="text-[9px] uppercase tracking-wide text-surface-500">Delta</div>
              <div className="text-xs text-surface-200 mono-data font-semibold">
                {delta > 0 ? `+${delta}` : delta}
              </div>
            </div>
            <div className="rounded-lg border border-white/[0.06] bg-white/[0.03] px-3 py-2">
              <div className="text-[9px] uppercase tracking-wide text-surface-500">Puntos</div>
              <div className="text-xs text-surface-200 mono-data font-semibold">{points.length}</div>
            </div>
          </div>

          <div className="flex items-end gap-1 h-16 rounded-lg border border-white/[0.06] bg-white/[0.02] p-2">
            {points.map((point) => {
              const height = `${Math.max(10, (point.activeSignals / point.totalSignals) * 100)}%`
              return (
                <div
                  key={point.capturedAt}
                  className="flex-1 rounded-t bg-brand-400/70 border border-brand-300/30"
                  style={{ height }}
                  title={`${formatShortTime(point.capturedAt)} - ${point.activeSignals}/${point.totalSignals}`}
                />
              )
            })}
          </div>

          <div className="flex items-center justify-between text-[10px] text-surface-500">
            <span>Inicio: {formatShortTime(oldest.capturedAt)}</span>
            <span>Último: {formatShortTime(latest.capturedAt)}</span>
          </div>

          <div className="text-[10px] text-surface-500">
            {latest.missingSignals.length > 0
              ? `Faltan: ${latest.missingSignals.join(', ')}`
              : 'Todas las familias de evidencia están activas.'}
          </div>
        </div>
      ) : (
        <p className="text-xs text-surface-400 text-center py-4">
          Sin snapshots todavía. Se registran automáticamente al cargar estadísticas.
        </p>
      )}
    </div>
  )
}
