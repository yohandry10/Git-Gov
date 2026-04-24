import {
  clampPercent,
  computeCompositeRisk,
  computeReleaseReadiness,
  computeWeightedScore,
  getRepoTierProfile,
} from '@/components/control_plane/risk-scoring'

describe('risk-scoring', () => {
  it('clamps percentage values safely', () => {
    expect(clampPercent(-15)).toBe(0)
    expect(clampPercent(150)).toBe(100)
    expect(clampPercent(Number.NaN)).toBe(0)
    expect(clampPercent(42.5)).toBe(42.5)
  })

  it('returns zero score when no weighted signal is available', () => {
    const result = computeWeightedScore([
      { value: 80, weight: 0.5, available: false },
      { value: 60, weight: 0.5, available: false },
    ])
    expect(result).toEqual({ score: 0, available: 0, total: 2 })
  })

  it('computes release readiness with tier-aware weights and bands', () => {
    const standard = computeReleaseReadiness({
      tier: 'standard',
      pipelineSuccessRate: 90,
      ticketCoveragePercent: 80,
      sonarPassRate: 70,
      pipelineAvailable: true,
      ticketCoverageAvailable: true,
      sonarAvailable: true,
    })
    expect(standard.score).toBe(82)
    expect(standard.band).toBe('Vigilancia')

    const critical = computeReleaseReadiness({
      tier: 'critical',
      pipelineSuccessRate: 90,
      ticketCoveragePercent: 80,
      sonarPassRate: 70,
      pipelineAvailable: true,
      ticketCoverageAvailable: true,
      sonarAvailable: true,
    })
    expect(critical.score).toBe(82)
    expect(critical.band).toBe('Vigilancia')
  })

  it('uses stricter risk banding for critical tier', () => {
    const standard = computeCompositeRisk({
      tier: 'standard',
      blockedPushRate: 55,
      ticketGapRate: 55,
      pipelineFailureRate: 55,
      sonarFailureRate: 55,
      unresolvedViolationRate: 55,
      blockedPushAvailable: true,
      ticketGapAvailable: true,
      pipelineFailureAvailable: true,
      sonarFailureAvailable: true,
      unresolvedViolationAvailable: true,
    })

    const critical = computeCompositeRisk({
      tier: 'critical',
      blockedPushRate: 55,
      ticketGapRate: 55,
      pipelineFailureRate: 55,
      sonarFailureRate: 55,
      unresolvedViolationRate: 55,
      blockedPushAvailable: true,
      ticketGapAvailable: true,
      pipelineFailureAvailable: true,
      sonarFailureAvailable: true,
      unresolvedViolationAvailable: true,
    })

    expect(standard.score).toBe(55)
    expect(standard.band).toBe('Medio')
    expect(critical.score).toBe(55)
    expect(critical.band).toBe('Alto')
  })

  it('exposes expected standard-tier readiness target', () => {
    const profile = getRepoTierProfile('standard')
    expect(profile.risk.sla.minReadinessScore).toBe(75)
  })
})
