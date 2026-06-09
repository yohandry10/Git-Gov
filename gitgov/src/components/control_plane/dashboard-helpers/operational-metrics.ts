import { formatDurationMs } from './event-log'

export interface OperationalPipelineEvidence {
  commit_created_at: number
  pipeline?: {
    pipeline_event_id?: string | null
    pipeline_id?: string | null
    job_name: string
    status: string
    ingested_at: number
  } | null
}

export interface OperationalEvidenceMetrics {
  timeToEvidenceMs: number | null
  timeToEvidenceSamples: number
  mttrMs: number | null
  mttrSamples: number
}

const SUCCESS_PIPELINE_STATUSES = new Set(['success', 'ok', 'passed'])
const RECOVERABLE_FAILURE_PIPELINE_STATUSES = new Set([
  'failure',
  'failed',
  'error',
  'unstable',
  'aborted',
  'cancelled',
  'canceled',
])

function normalizePipelineStatus(status?: string | null): string {
  return (status ?? '').trim().toLowerCase()
}

function isSuccessPipelineStatus(status?: string | null): boolean {
  return SUCCESS_PIPELINE_STATUSES.has(normalizePipelineStatus(status))
}

function isRecoverableFailurePipelineStatus(status?: string | null): boolean {
  return RECOVERABLE_FAILURE_PIPELINE_STATUSES.has(normalizePipelineStatus(status))
}

function averageMs(values: number[]): number | null {
  if (values.length === 0) return null
  return Math.round(values.reduce((total, value) => total + value, 0) / values.length)
}

function pipelineEvidenceKey(evidence: OperationalPipelineEvidence, index: number): string {
  const pipeline = evidence.pipeline
  return (
    pipeline?.pipeline_event_id ||
    pipeline?.pipeline_id ||
    `${pipeline?.job_name ?? 'unknown'}:${pipeline?.ingested_at ?? index}:${pipeline?.status ?? 'unknown'}`
  )
}

export function formatOperationalMetricDuration(ms: number | null, samples: number): string {
  if (samples <= 0 || ms === null) return 'N/A'
  if (ms <= 0) return '0s'
  return formatDurationMs(ms)
}

export function buildOperationalEvidenceMetrics(
  correlations: OperationalPipelineEvidence[],
): OperationalEvidenceMetrics {
  const seenPipelines = new Set<string>()
  const pipelines = correlations.flatMap((entry, index) => {
    const pipeline = entry.pipeline
    if (!pipeline) return []
    if (!Number.isFinite(entry.commit_created_at) || !Number.isFinite(pipeline.ingested_at)) return []

    const key = pipelineEvidenceKey(entry, index)
    if (seenPipelines.has(key)) return []
    seenPipelines.add(key)

    return [{
      commitCreatedAt: entry.commit_created_at,
      ingestedAt: pipeline.ingested_at,
      jobName: pipeline.job_name.trim() || 'unknown',
      status: normalizePipelineStatus(pipeline.status),
    }]
  })

  const timeToEvidenceDeltas = pipelines
    .map((pipeline) => pipeline.ingestedAt - pipeline.commitCreatedAt)
    .filter((delta) => Number.isFinite(delta) && delta >= 0)

  const pipelinesByJob = new Map<string, typeof pipelines>()
  for (const pipeline of pipelines) {
    const jobKey = pipeline.jobName.toLowerCase()
    const jobRuns = pipelinesByJob.get(jobKey) ?? []
    jobRuns.push(pipeline)
    pipelinesByJob.set(jobKey, jobRuns)
  }

  const recoveryDeltas: number[] = []
  for (const jobRuns of pipelinesByJob.values()) {
    const orderedRuns = [...jobRuns].sort((a, b) => a.ingestedAt - b.ingestedAt)
    for (let index = 0; index < orderedRuns.length; index++) {
      const run = orderedRuns[index]
      if (!isRecoverableFailurePipelineStatus(run.status)) continue
      const recovery = orderedRuns
        .slice(index + 1)
        .find((nextRun) => nextRun.ingestedAt >= run.ingestedAt && isSuccessPipelineStatus(nextRun.status))
      if (recovery) {
        recoveryDeltas.push(recovery.ingestedAt - run.ingestedAt)
      }
    }
  }

  return {
    timeToEvidenceMs: averageMs(timeToEvidenceDeltas),
    timeToEvidenceSamples: timeToEvidenceDeltas.length,
    mttrMs: averageMs(recoveryDeltas),
    mttrSamples: recoveryDeltas.length,
  }
}
