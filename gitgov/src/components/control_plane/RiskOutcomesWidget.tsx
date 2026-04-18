import { ShieldAlert } from 'lucide-react'
import { Bar } from './Bar'

interface RiskOutcomesWidgetProps {
  trackedPushesToday: number
  blockedPushesToday: number
  ticketCoveragePercent: number
  pipelineTotal7d: number
  pipelineFailure7d: number
  sonarTotal: number
  sonarFailed: number
  unresolvedViolations: number
  totalViolations: number
  criticalViolations: number
  releaseReadinessScore: number
}

interface WeightedSignal {
  value: number
  weight: number
  available: boolean
}

function clampPercent(value: number): number {
  if (!Number.isFinite(value)) return 0
  if (value < 0) return 0
  if (value > 100) return 100
  return value
}

function computeCompositeRisk(signals: WeightedSignal[]): { score: number; available: number; total: number } {
  const activeSignals = signals.filter((signal) => signal.available)
  if (activeSignals.length === 0) {
    return { score: 0, available: 0, total: signals.length }
  }
  const totalWeight = activeSignals.reduce((acc, signal) => acc + signal.weight, 0)
  if (totalWeight <= 0) {
    return { score: 0, available: activeSignals.length, total: signals.length }
  }
  const weightedSum = activeSignals.reduce((acc, signal) => acc + (signal.value * signal.weight), 0)
  return {
    score: Math.round(weightedSum / totalWeight),
    available: activeSignals.length,
    total: signals.length,
  }
}

export function RiskOutcomesWidget({
  trackedPushesToday,
  blockedPushesToday,
  ticketCoveragePercent,
  pipelineTotal7d,
  pipelineFailure7d,
  sonarTotal,
  sonarFailed,
  unresolvedViolations,
  totalViolations,
  criticalViolations,
  releaseReadinessScore,
}: RiskOutcomesWidgetProps) {
  const pushAttempts = trackedPushesToday + blockedPushesToday
  const trustedPathRate = pushAttempts > 0
    ? clampPercent((trackedPushesToday / pushAttempts) * 100)
    : 100
  const blockedPushRate = pushAttempts > 0
    ? clampPercent((blockedPushesToday / pushAttempts) * 100)
    : 0
  const ticketCoverage = clampPercent(ticketCoveragePercent)
  const ticketGapRate = clampPercent(100 - ticketCoverage)
  const pipelineFailureRate = pipelineTotal7d > 0
    ? clampPercent((pipelineFailure7d / pipelineTotal7d) * 100)
    : 0
  const sonarFailureRate = sonarTotal > 0
    ? clampPercent((sonarFailed / sonarTotal) * 100)
    : 0
  const unresolvedViolationRate = totalViolations > 0
    ? clampPercent((unresolvedViolations / totalViolations) * 100)
    : 0

  const riskSignals: WeightedSignal[] = [
    { value: blockedPushRate, weight: 0.2, available: pushAttempts > 0 },
    { value: ticketGapRate, weight: 0.2, available: true },
    { value: pipelineFailureRate, weight: 0.2, available: pipelineTotal7d > 0 },
    { value: sonarFailureRate, weight: 0.2, available: sonarTotal > 0 },
    { value: unresolvedViolationRate, weight: 0.2, available: totalViolations > 0 },
  ]
  const composite = computeCompositeRisk(riskSignals)
  const compositeScore = composite.score

  const riskBand = composite.available === 0
    ? 'Insuficiente'
    : compositeScore >= 60
      ? 'Alto'
      : compositeScore >= 35
        ? 'Medio'
        : 'Bajo'
  const riskBandClass = composite.available === 0
    ? 'text-surface-500'
    : compositeScore >= 60
      ? 'text-danger-300'
      : compositeScore >= 35
        ? 'text-amber-300'
        : 'text-emerald-300'

  return (
    <div className="glass-panel p-5">
      <div className="card-header mb-4">
        <ShieldAlert size={11} strokeWidth={1.5} className="text-surface-400" />
        Risk Outcomes (operativo)
      </div>

      <div className="space-y-3">
        <div className="flex items-baseline gap-3">
          <span className="text-4xl font-bold text-white tracking-tighter mono-data leading-none">
            {compositeScore}
          </span>
          <span className="text-xs text-surface-400 uppercase tracking-wide">risk score / 100</span>
          <span className={`text-xs font-semibold uppercase tracking-wide ${riskBandClass}`}>
            {riskBand}
          </span>
        </div>
        <Bar value={compositeScore} color={compositeScore >= 60 ? 'danger' : compositeScore >= 35 ? 'warning' : 'success'} />

        <div className="grid grid-cols-2 gap-x-8 gap-y-2 pt-2">
          <div className="flex items-center justify-between text-xs">
            <span className="text-surface-400">Ruta confiable</span>
            <span className="mono-data font-medium text-emerald-300">{trustedPathRate.toFixed(1)}%</span>
          </div>
          <div className="flex items-center justify-between text-xs">
            <span className="text-surface-400">Pushes bloqueados</span>
            <span className={`mono-data font-medium ${blockedPushRate > 10 ? 'text-danger-300' : 'text-surface-200'}`}>
              {blockedPushRate.toFixed(1)}%
            </span>
          </div>
          <div className="flex items-center justify-between text-xs">
            <span className="text-surface-400">Gap de trazabilidad</span>
            <span className={`mono-data font-medium ${ticketGapRate > 30 ? 'text-amber-300' : 'text-surface-200'}`}>
              {ticketGapRate.toFixed(1)}%
            </span>
          </div>
          <div className="flex items-center justify-between text-xs">
            <span className="text-surface-400">Fallos pipeline (7d)</span>
            <span className={`mono-data font-medium ${pipelineFailureRate > 20 ? 'text-danger-300' : 'text-surface-200'}`}>
              {pipelineFailureRate.toFixed(1)}%
            </span>
          </div>
          <div className="flex items-center justify-between text-xs">
            <span className="text-surface-400">Fallos Sonar (sample)</span>
            <span className={`mono-data font-medium ${sonarTotal > 0 && sonarFailureRate > 20 ? 'text-danger-300' : 'text-surface-200'}`}>
              {sonarTotal > 0 ? `${sonarFailureRate.toFixed(1)}%` : 'N/A'}
            </span>
          </div>
          <div className="flex items-center justify-between text-xs">
            <span className="text-surface-400">Violaciones abiertas</span>
            <span className={`mono-data font-medium ${unresolvedViolationRate > 40 ? 'text-danger-300' : 'text-surface-200'}`}>
              {totalViolations > 0 ? `${unresolvedViolationRate.toFixed(1)}%` : 'N/A'}
            </span>
          </div>
          <div className="flex items-center justify-between text-xs">
            <span className="text-surface-400">Violaciones críticas</span>
            <span className={`mono-data font-medium ${criticalViolations > 0 ? 'text-danger-300' : 'text-surface-200'}`}>
              {criticalViolations}
            </span>
          </div>
          <div className="flex items-center justify-between text-xs">
            <span className="text-surface-400">Release readiness</span>
            <span className={`mono-data font-medium ${releaseReadinessScore >= 80 ? 'text-emerald-300' : releaseReadinessScore >= 65 ? 'text-amber-300' : 'text-danger-300'}`}>
              {releaseReadinessScore}/100
            </span>
          </div>
        </div>

        <div className="pt-1 text-[10px] text-surface-500">
          Señales activas para score: {composite.available}/{composite.total}. MTTR y Time-to-Evidence quedan para fase siguiente.
        </div>
      </div>
    </div>
  )
}
