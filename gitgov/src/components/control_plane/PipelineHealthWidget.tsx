import { Workflow } from 'lucide-react'
import { formatDurationMs } from './dashboard-helpers'
import { Bar } from './Bar'
import type { ReleaseReadinessBand } from './risk-scoring'

interface PipelineHealthWidgetProps {
  total: number
  failure: number
  avgDurationMs: number
  reposWithFailures: number
  successRate: string
  sonarTotal: number
  sonarPassed: number
  sonarFailed: number
  sonarUnstable: number
  sonarPassRate: string
  releaseReadinessScore: number
  releaseReadinessSignals: number
  releaseReadinessBand: ReleaseReadinessBand
  readinessTierLabel: string
  readinessTargetScore: number
  githubPrEvents: number
  githubPrReviewEvents: number
  githubPrCommentEvents: number
  githubStatusCheckEvents: number
  githubEvidenceSignals: number
}

export function PipelineHealthWidget({
  total,
  failure,
  avgDurationMs,
  reposWithFailures,
  successRate,
  sonarTotal,
  sonarPassed,
  sonarFailed,
  sonarUnstable,
  sonarPassRate,
  releaseReadinessScore,
  releaseReadinessSignals,
  releaseReadinessBand,
  readinessTierLabel,
  readinessTargetScore,
  githubPrEvents,
  githubPrReviewEvents,
  githubPrCommentEvents,
  githubStatusCheckEvents,
  githubEvidenceSignals,
}: PipelineHealthWidgetProps) {
  const readinessClass = releaseReadinessBand === 'Insuficiente'
    ? 'text-surface-500'
    : releaseReadinessBand === 'Fuerte'
      ? 'text-emerald-300'
      : releaseReadinessBand === 'Vigilancia'
        ? 'text-amber-300'
        : 'text-danger-300'

  return (
    <div className="glass-panel p-5">
      <div className="card-header mb-4">
        <Workflow size={11} strokeWidth={1.5} className="text-surface-400" />
        Pipeline Health (7d)
      </div>
      {total > 0 ? (
        <div className="space-y-3">
          <div className="flex items-baseline gap-3">
            <span className="text-4xl font-bold text-white tracking-tighter mono-data leading-none">{successRate}%</span>
            <span className="text-xs text-surface-400 uppercase tracking-wide">success rate</span>
          </div>
          <Bar value={parseFloat(successRate)} color="success" />
          <div className="grid grid-cols-2 gap-x-8 gap-y-2 pt-2">
            {([
              ['Pipelines', total, ''],
              ['Failures', failure, 'text-danger-400'],
              ['Avg duration', formatDurationMs(avgDurationMs), ''],
              ['Repos w/ failures', reposWithFailures, ''],
              ['Release readiness', `${releaseReadinessScore}/100`, readinessClass],
              ['Readiness band', releaseReadinessBand, readinessClass],
              ['Readiness signals', `${releaseReadinessSignals}/3`, releaseReadinessSignals < 3 ? 'text-amber-300' : 'text-emerald-300'],
              ['Readiness tier', readinessTierLabel, 'text-surface-200'],
              ['Readiness SLA target', `>= ${readinessTargetScore}`, releaseReadinessScore >= readinessTargetScore ? 'text-emerald-300' : 'text-amber-300'],
              ['Sonar scans (sample)', sonarTotal, ''],
              ['Sonar pass rate', `${sonarPassRate}%`, sonarTotal > 0 ? 'text-emerald-300' : ''],
              ['Sonar failed', sonarFailed, sonarFailed > 0 ? 'text-danger-400' : ''],
              ['Sonar unstable', sonarUnstable, sonarUnstable > 0 ? 'text-amber-300' : ''],
              ['Sonar passed', sonarPassed, sonarPassed > 0 ? 'text-emerald-300' : ''],
              ['GitHub PR events', githubPrEvents, githubPrEvents > 0 ? 'text-emerald-300' : 'text-surface-500'],
              ['GitHub review events', githubPrReviewEvents, githubPrReviewEvents > 0 ? 'text-emerald-300' : 'text-surface-500'],
              ['GitHub PR comment events', githubPrCommentEvents, githubPrCommentEvents > 0 ? 'text-emerald-300' : 'text-surface-500'],
              ['GitHub status-check events', githubStatusCheckEvents, githubStatusCheckEvents > 0 ? 'text-emerald-300' : 'text-surface-500'],
              ['GitHub evidence signals', `${githubEvidenceSignals}/4`, githubEvidenceSignals < 4 ? 'text-amber-300' : 'text-emerald-300'],
            ] as const).map(([label, val, cls]) => (
              <div key={label} className="flex items-center justify-between text-xs">
                <span className="text-surface-400">{label}</span>
                <span className={`mono-data font-medium ${cls || 'text-surface-200'}`}>{val}</span>
              </div>
            ))}
          </div>
        </div>
      ) : (
        <div className="py-10 text-center">
          <Workflow size={20} strokeWidth={1.5} className="mx-auto text-surface-700 mb-2" />
          <p className="text-xs text-surface-400">Sin datos de pipelines</p>
        </div>
      )}
    </div>
  )
}
