export type RepoTier = 'critical' | 'standard' | 'internal'

export interface WeightedSignal {
  value: number
  weight: number
  available: boolean
}

export interface RepoTierProfile {
  id: RepoTier
  label: string
  readiness: {
    weights: {
      pipeline: number
      traceability: number
      sonar: number
    }
    bands: {
      healthy: number
      watch: number
    }
  }
  risk: {
    weights: {
      blockedPush: number
      ticketGap: number
      pipelineFailure: number
      sonarFailure: number
      unresolvedViolations: number
    }
    bands: {
      lowUpper: number
      mediumUpper: number
    }
    sla: {
      blockedPushRateMax: number
      ticketGapRateMax: number
      pipelineFailureRateMax: number
      sonarFailureRateMax: number
      unresolvedViolationRateMax: number
      minReadinessScore: number
    }
  }
}

export const REPO_TIER_PROFILES: Record<RepoTier, RepoTierProfile> = {
  critical: {
    id: 'critical',
    label: 'Critical',
    readiness: {
      weights: { pipeline: 0.5, traceability: 0.2, sonar: 0.3 },
      bands: { healthy: 90, watch: 78 },
    },
    risk: {
      weights: {
        blockedPush: 0.25,
        ticketGap: 0.25,
        pipelineFailure: 0.2,
        sonarFailure: 0.2,
        unresolvedViolations: 0.1,
      },
      bands: { lowUpper: 30, mediumUpper: 50 },
      sla: {
        blockedPushRateMax: 5,
        ticketGapRateMax: 15,
        pipelineFailureRateMax: 10,
        sonarFailureRateMax: 12,
        unresolvedViolationRateMax: 30,
        minReadinessScore: 85,
      },
    },
  },
  standard: {
    id: 'standard',
    label: 'Standard',
    readiness: {
      weights: { pipeline: 0.45, traceability: 0.25, sonar: 0.3 },
      bands: { healthy: 85, watch: 70 },
    },
    risk: {
      weights: {
        blockedPush: 0.2,
        ticketGap: 0.2,
        pipelineFailure: 0.2,
        sonarFailure: 0.2,
        unresolvedViolations: 0.2,
      },
      bands: { lowUpper: 35, mediumUpper: 60 },
      sla: {
        blockedPushRateMax: 10,
        ticketGapRateMax: 25,
        pipelineFailureRateMax: 20,
        sonarFailureRateMax: 20,
        unresolvedViolationRateMax: 40,
        minReadinessScore: 75,
      },
    },
  },
  internal: {
    id: 'internal',
    label: 'Internal',
    readiness: {
      weights: { pipeline: 0.4, traceability: 0.2, sonar: 0.4 },
      bands: { healthy: 80, watch: 65 },
    },
    risk: {
      weights: {
        blockedPush: 0.15,
        ticketGap: 0.15,
        pipelineFailure: 0.25,
        sonarFailure: 0.2,
        unresolvedViolations: 0.25,
      },
      bands: { lowUpper: 40, mediumUpper: 65 },
      sla: {
        blockedPushRateMax: 15,
        ticketGapRateMax: 35,
        pipelineFailureRateMax: 30,
        sonarFailureRateMax: 30,
        unresolvedViolationRateMax: 50,
        minReadinessScore: 65,
      },
    },
  },
}

export function getRepoTierProfile(tier: RepoTier): RepoTierProfile {
  return REPO_TIER_PROFILES[tier] ?? REPO_TIER_PROFILES.standard
}

export function clampPercent(value: number): number {
  if (!Number.isFinite(value)) return 0
  if (value < 0) return 0
  if (value > 100) return 100
  return value
}

export function computeWeightedScore(signals: WeightedSignal[]): { score: number; available: number; total: number } {
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

export interface ReleaseReadinessInput {
  tier: RepoTier
  pipelineSuccessRate: number
  ticketCoveragePercent: number
  sonarPassRate: number
  pipelineAvailable: boolean
  ticketCoverageAvailable: boolean
  sonarAvailable: boolean
}

export type ReleaseReadinessBand = 'Insuficiente' | 'Fuerte' | 'Vigilancia' | 'Crítico'

export function computeReleaseReadiness(input: ReleaseReadinessInput): {
  score: number
  available: number
  total: number
  band: ReleaseReadinessBand
} {
  const profile = getRepoTierProfile(input.tier)
  const score = computeWeightedScore([
    {
      value: clampPercent(input.pipelineSuccessRate),
      weight: profile.readiness.weights.pipeline,
      available: input.pipelineAvailable,
    },
    {
      value: clampPercent(input.ticketCoveragePercent),
      weight: profile.readiness.weights.traceability,
      available: input.ticketCoverageAvailable,
    },
    {
      value: clampPercent(input.sonarPassRate),
      weight: profile.readiness.weights.sonar,
      available: input.sonarAvailable,
    },
  ])

  if (score.available === 0) {
    return { ...score, band: 'Insuficiente' }
  }
  if (score.score >= profile.readiness.bands.healthy) {
    return { ...score, band: 'Fuerte' }
  }
  if (score.score >= profile.readiness.bands.watch) {
    return { ...score, band: 'Vigilancia' }
  }
  return { ...score, band: 'Crítico' }
}

export interface CompositeRiskInput {
  tier: RepoTier
  blockedPushRate: number
  ticketGapRate: number
  pipelineFailureRate: number
  sonarFailureRate: number
  unresolvedViolationRate: number
  blockedPushAvailable: boolean
  ticketGapAvailable: boolean
  pipelineFailureAvailable: boolean
  sonarFailureAvailable: boolean
  unresolvedViolationAvailable: boolean
}

export type CompositeRiskBand = 'Insuficiente' | 'Bajo' | 'Medio' | 'Alto'

export function computeCompositeRisk(input: CompositeRiskInput): {
  score: number
  available: number
  total: number
  band: CompositeRiskBand
} {
  const profile = getRepoTierProfile(input.tier)
  const score = computeWeightedScore([
    {
      value: clampPercent(input.blockedPushRate),
      weight: profile.risk.weights.blockedPush,
      available: input.blockedPushAvailable,
    },
    {
      value: clampPercent(input.ticketGapRate),
      weight: profile.risk.weights.ticketGap,
      available: input.ticketGapAvailable,
    },
    {
      value: clampPercent(input.pipelineFailureRate),
      weight: profile.risk.weights.pipelineFailure,
      available: input.pipelineFailureAvailable,
    },
    {
      value: clampPercent(input.sonarFailureRate),
      weight: profile.risk.weights.sonarFailure,
      available: input.sonarFailureAvailable,
    },
    {
      value: clampPercent(input.unresolvedViolationRate),
      weight: profile.risk.weights.unresolvedViolations,
      available: input.unresolvedViolationAvailable,
    },
  ])

  if (score.available === 0) {
    return { ...score, band: 'Insuficiente' }
  }
  if (score.score >= profile.risk.bands.mediumUpper) {
    return { ...score, band: 'Alto' }
  }
  if (score.score >= profile.risk.bands.lowUpper) {
    return { ...score, band: 'Medio' }
  }
  return { ...score, band: 'Bajo' }
}
