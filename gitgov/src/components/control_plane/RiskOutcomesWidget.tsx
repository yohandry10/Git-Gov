import { ShieldAlert } from 'lucide-react'
import { Bar } from './Bar'
import { clampPercent, computeCompositeRisk, getRepoTierProfile, type RepoTier } from './risk-scoring'

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
  repoTier: RepoTier
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
  repoTier,
}: RiskOutcomesWidgetProps) {
  const tierProfile = getRepoTierProfile(repoTier)
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

  const composite = computeCompositeRisk({
    tier: repoTier,
    blockedPushRate,
    ticketGapRate,
    pipelineFailureRate,
    sonarFailureRate,
    unresolvedViolationRate,
    blockedPushAvailable: pushAttempts > 0,
    ticketGapAvailable: true,
    pipelineFailureAvailable: pipelineTotal7d > 0,
    sonarFailureAvailable: sonarTotal > 0,
    unresolvedViolationAvailable: totalViolations > 0,
  })
  const compositeScore = composite.score

  const riskBand = composite.band
  const riskBandClass = riskBand === 'Insuficiente'
    ? 'text-surface-500'
    : riskBand === 'Alto'
      ? 'text-danger-300'
      : riskBand === 'Medio'
        ? 'text-amber-300'
        : 'text-emerald-300'
  const riskBarColor: 'danger' | 'warning' | 'success' = riskBand === 'Alto'
    ? 'danger'
    : riskBand === 'Medio'
      ? 'warning'
      : 'success'
  const releaseReadinessClass = releaseReadinessScore >= tierProfile.risk.sla.minReadinessScore
    ? 'text-emerald-300'
    : releaseReadinessScore >= (tierProfile.risk.sla.minReadinessScore - 10)
      ? 'text-amber-300'
      : 'text-danger-300'

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
        <Bar value={compositeScore} color={riskBarColor} />

        <div className="grid grid-cols-2 gap-x-8 gap-y-2 pt-2">
          <div className="flex items-center justify-between text-xs">
            <span className="text-surface-400">Ruta confiable</span>
            <span className="mono-data font-medium text-emerald-300">{trustedPathRate.toFixed(1)}%</span>
          </div>
          <div className="flex items-center justify-between text-xs">
            <span className="text-surface-400">Pushes bloqueados</span>
            <span className={`mono-data font-medium ${blockedPushRate > tierProfile.risk.sla.blockedPushRateMax ? 'text-danger-300' : 'text-surface-200'}`}>
              {blockedPushRate.toFixed(1)}%
            </span>
          </div>
          <div className="flex items-center justify-between text-xs">
            <span className="text-surface-400">Gap de trazabilidad</span>
            <span className={`mono-data font-medium ${ticketGapRate > tierProfile.risk.sla.ticketGapRateMax ? 'text-amber-300' : 'text-surface-200'}`}>
              {ticketGapRate.toFixed(1)}%
            </span>
          </div>
          <div className="flex items-center justify-between text-xs">
            <span className="text-surface-400">Fallos pipeline (7d)</span>
            <span className={`mono-data font-medium ${pipelineFailureRate > tierProfile.risk.sla.pipelineFailureRateMax ? 'text-danger-300' : 'text-surface-200'}`}>
              {pipelineFailureRate.toFixed(1)}%
            </span>
          </div>
          <div className="flex items-center justify-between text-xs">
            <span className="text-surface-400">Fallos Sonar (sample)</span>
            <span className={`mono-data font-medium ${sonarTotal > 0 && sonarFailureRate > tierProfile.risk.sla.sonarFailureRateMax ? 'text-danger-300' : 'text-surface-200'}`}>
              {sonarTotal > 0 ? `${sonarFailureRate.toFixed(1)}%` : 'N/A'}
            </span>
          </div>
          <div className="flex items-center justify-between text-xs">
            <span className="text-surface-400">Violaciones abiertas</span>
            <span className={`mono-data font-medium ${unresolvedViolationRate > tierProfile.risk.sla.unresolvedViolationRateMax ? 'text-danger-300' : 'text-surface-200'}`}>
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
            <span className={`mono-data font-medium ${releaseReadinessClass}`}>
              {releaseReadinessScore}/100
            </span>
          </div>
        </div>

        <div className="pt-1 text-[10px] text-surface-500">
          Tier {tierProfile.label}. Señales activas para score: {composite.available}/{composite.total}. MTTR y Time-to-Evidence quedan para fase siguiente.
        </div>
      </div>
    </div>
  )
}
