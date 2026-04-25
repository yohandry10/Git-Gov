import {
  buildOperationalEvidenceMetrics,
  formatOperationalMetricDuration,
  type OperationalPipelineEvidence,
} from '@/components/control_plane/dashboard-helpers'

describe('dashboard-helpers operational evidence metrics', () => {
  it('computes average time-to-evidence from commit to pipeline ingestion', () => {
    const correlations: OperationalPipelineEvidence[] = [
      {
        commit_created_at: 1_000,
        pipeline: {
          pipeline_event_id: 'pipe-1',
          pipeline_id: 'jenkins-1',
          job_name: 'gitgov-demo-pipeline',
          status: 'success',
          ingested_at: 11_000,
        },
      },
      {
        commit_created_at: 2_000,
        pipeline: {
          pipeline_event_id: 'pipe-2',
          pipeline_id: 'jenkins-2',
          job_name: 'gitgov-demo-pipeline',
          status: 'success',
          ingested_at: 22_000,
        },
      },
      {
        commit_created_at: 30_000,
        pipeline: {
          pipeline_event_id: 'pipe-3',
          pipeline_id: 'jenkins-3',
          job_name: 'gitgov-demo-pipeline',
          status: 'success',
          ingested_at: 20_000,
        },
      },
    ]

    const metrics = buildOperationalEvidenceMetrics(correlations)

    expect(metrics.timeToEvidenceSamples).toBe(2)
    expect(metrics.timeToEvidenceMs).toBe(15_000)
  })

  it('deduplicates pipeline evidence before calculating samples', () => {
    const correlations: OperationalPipelineEvidence[] = [
      {
        commit_created_at: 1_000,
        pipeline: {
          pipeline_event_id: 'pipe-1',
          pipeline_id: 'jenkins-1',
          job_name: 'gitgov-demo-pipeline',
          status: 'success',
          ingested_at: 11_000,
        },
      },
      {
        commit_created_at: 1_000,
        pipeline: {
          pipeline_event_id: 'pipe-1',
          pipeline_id: 'jenkins-1',
          job_name: 'gitgov-demo-pipeline',
          status: 'success',
          ingested_at: 11_000,
        },
      },
    ]

    const metrics = buildOperationalEvidenceMetrics(correlations)

    expect(metrics.timeToEvidenceSamples).toBe(1)
    expect(metrics.timeToEvidenceMs).toBe(10_000)
  })

  it('computes MTTR from recoverable pipeline failure to next success for the same job', () => {
    const correlations: OperationalPipelineEvidence[] = [
      {
        commit_created_at: 1_000,
        pipeline: {
          pipeline_event_id: 'pipe-1',
          pipeline_id: 'jenkins-1',
          job_name: 'gitgov-demo-pipeline',
          status: 'failure',
          ingested_at: 10_000,
        },
      },
      {
        commit_created_at: 1_000,
        pipeline: {
          pipeline_event_id: 'pipe-2',
          pipeline_id: 'jenkins-2',
          job_name: 'gitgov-demo-pipeline',
          status: 'success',
          ingested_at: 70_000,
        },
      },
      {
        commit_created_at: 1_000,
        pipeline: {
          pipeline_event_id: 'pipe-3',
          pipeline_id: 'jenkins-3',
          job_name: 'sonar-governance',
          status: 'error',
          ingested_at: 100_000,
        },
      },
      {
        commit_created_at: 1_000,
        pipeline: {
          pipeline_event_id: 'pipe-4',
          pipeline_id: 'jenkins-4',
          job_name: 'sonar-governance',
          status: 'passed',
          ingested_at: 220_000,
        },
      },
    ]

    const metrics = buildOperationalEvidenceMetrics(correlations)

    expect(metrics.mttrSamples).toBe(2)
    expect(metrics.mttrMs).toBe(90_000)
  })

  it('returns empty operational metrics when evidence is insufficient', () => {
    const metrics = buildOperationalEvidenceMetrics([
      {
        commit_created_at: 10_000,
        pipeline: null,
      },
    ])

    expect(metrics).toEqual({
      timeToEvidenceMs: null,
      timeToEvidenceSamples: 0,
      mttrMs: null,
      mttrSamples: 0,
    })
    expect(formatOperationalMetricDuration(metrics.mttrMs, metrics.mttrSamples)).toBe('N/A')
  })
})
