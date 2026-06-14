import { NavLink, useParams } from 'react-router-dom'
import { useEffect, useState, type ComponentType } from 'react'
import { useTranslation } from 'react-i18next'
import {
  AlertTriangle,
  Bot,
  CheckCircle2,
  ClipboardCheck,
  FileText,
  Landmark,
  ShieldCheck,
} from 'lucide-react'
import clsx from 'clsx'
import { useControlPlaneStore } from '@/store/useControlPlaneStore'
import { DeveloperAccessPanel } from '@/components/control_plane/DeveloperAccessPanel'
import { EvidencePacketPanel } from '@/components/control_plane/EvidencePacketPanel'
import { EventBreakdownGrid } from '@/components/control_plane/EventBreakdownGrid'
import { ExportPanel } from '@/components/control_plane/ExportPanel'
import { GitHubEvidenceTrendWidget } from '@/components/control_plane/GitHubEvidenceTrendWidget'
import { GovernanceCopilotPanel } from '@/components/control_plane/GovernanceCopilotPanel'
import { PolicyEditorPanel } from '@/components/control_plane/PolicyEditorPanel'
import { RecentCommitsTable } from '@/components/control_plane/RecentCommitsTable'
import { EnterpriseAdoptionPanel } from '@/components/control_plane/EnterpriseAdoptionPanel'
import { FirstGovernedRepoSetupPanel } from '@/components/control_plane/FirstGovernedRepoSetupPanel'
import { ReleaseApprovalPanel } from '@/components/control_plane/ReleaseApprovalPanel'
import { DeploymentGateHistoryPanel } from '@/components/control_plane/DeploymentGateHistoryPanel'
import { ComplianceEvidenceFlowPanel } from '@/components/control_plane/ComplianceEvidenceFlowPanel'
import {
  appendGitHubEvidenceTrendPoint,
  buildGitHubEvidenceSummary,
  buildGitHubEvidenceTrendPoint,
  type GitHubEvidenceTrendPoint,
} from '@/components/control_plane/dashboard-helpers'
import {
  computeReleaseReadiness,
  getRepoTierProfile,
} from '@/components/control_plane/risk-scoring'

type GovernanceSection = 'evidence' | 'policy' | 'adoption' | 'releases' | 'copilot'

const GOVERNANCE_LOG_LIMIT = 120
const GITHUB_EVIDENCE_TREND_STORAGE_KEY = 'gitgov.dashboard.github_evidence_trend'

const GOVERNANCE_SECTIONS: Array<{
  id: GovernanceSection
  labelKey: string
  descriptionKey: string
  icon: ComponentType<{ size?: number; className?: string }>
}> = [
  {
    id: 'evidence',
    labelKey: 'governance.sections.evidence.label',
    descriptionKey: 'governance.sections.evidence.description',
    icon: FileText,
  },
  {
    id: 'policy',
    labelKey: 'governance.sections.policy.label',
    descriptionKey: 'governance.sections.policy.description',
    icon: ShieldCheck,
  },
  {
    id: 'adoption',
    labelKey: 'governance.sections.adoption.label',
    descriptionKey: 'governance.sections.adoption.description',
    icon: ClipboardCheck,
  },
  {
    id: 'releases',
    labelKey: 'governance.sections.releases.label',
    descriptionKey: 'governance.sections.releases.description',
    icon: Landmark,
  },
  {
    id: 'copilot',
    labelKey: 'governance.sections.copilot.label',
    descriptionKey: 'governance.sections.copilot.description',
    icon: Bot,
  },
]

function readStoredGitHubEvidenceTrend(): GitHubEvidenceTrendPoint[] {
  if (typeof window === 'undefined') return []
  const raw = window.localStorage.getItem(GITHUB_EVIDENCE_TREND_STORAGE_KEY)
  if (!raw) return []
  try {
    const parsed = JSON.parse(raw) as GitHubEvidenceTrendPoint[]
    if (!Array.isArray(parsed)) return []
    return parsed.filter((point) =>
      typeof point.capturedAt === 'string' &&
      typeof point.activeSignals === 'number' &&
      typeof point.totalSignals === 'number' &&
      typeof point.executiveStatus === 'string' &&
      Array.isArray(point.missingSignals),
    )
  } catch {
    return []
  }
}

function persistGitHubEvidenceTrend(points: GitHubEvidenceTrendPoint[]) {
  if (typeof window === 'undefined') return
  window.localStorage.setItem(GITHUB_EVIDENCE_TREND_STORAGE_KEY, JSON.stringify(points))
}

function normalizeGovernanceSection(section?: string): GovernanceSection {
  if (
    section === 'evidence' ||
    section === 'policy' ||
    section === 'adoption' ||
    section === 'releases' ||
    section === 'copilot'
  ) {
    return section
  }
  return 'evidence'
}

function formatPercent(value: number | null | undefined): string {
  if (value === null || value === undefined || Number.isNaN(value)) return 'N/A'
  return `${value.toFixed(1)}%`
}

function SummaryMetric({
  title,
  value,
  detail,
  healthy,
}: {
  title: string
  value: string
  detail: string
  healthy?: boolean
}) {
  const Icon = healthy ? CheckCircle2 : AlertTriangle
  return (
    <div className={`rounded-lg border p-3 ${healthy ? 'border-success-500/20 bg-success-500/8' : 'border-warning-500/20 bg-warning-500/8'}`}>
      <div className="flex items-center gap-2">
        <Icon size={13} className={healthy ? 'text-success-400' : 'text-warning-400'} />
        <p className="text-[10px] font-medium uppercase tracking-widest text-surface-500">{title}</p>
      </div>
      <p className="mt-2 mono-data text-lg font-semibold text-surface-100">{value}</p>
      <p className="mt-1 text-[11px] leading-5 text-surface-400">{detail}</p>
    </div>
  )
}

function GovernanceAccessNotice() {
  const { t } = useTranslation()
  return (
    <div className="glass-panel p-5">
      <div className="flex items-center gap-2 text-sm font-semibold text-surface-100">
        <ShieldCheck size={16} className="text-warning-400" />
        {t('governance.accessTitle')}
      </div>
      <p className="mt-2 max-w-2xl text-xs leading-5 text-surface-400">
        {t('governance.accessBody')}
      </p>
    </div>
  )
}

export function GovernancePage() {
  const { t } = useTranslation()
  const { section } = useParams()
  const activeSection = normalizeGovernanceSection(section)
  const serverStats = useControlPlaneStore((s) => s.serverStats)
  const ticketCoverage = useControlPlaneStore((s) => s.ticketCoverage)
  const jenkinsCorrelations = useControlPlaneStore((s) => s.jenkinsCorrelations)
  const userRole = useControlPlaneStore((s) => s.userRole)
  const isConnected = useControlPlaneStore((s) => s.isConnected)
  const loadStats = useControlPlaneStore((s) => s.loadStats)
  const loadLogsIncremental = useControlPlaneStore((s) => s.loadLogsIncremental)
  const connectSse = useControlPlaneStore((s) => s.connectSse)
  const disconnectSse = useControlPlaneStore((s) => s.disconnectSse)
  const isAdmin = userRole === 'Admin'
  const canUseCopilot = userRole === 'Admin' || userRole === 'Architect' || userRole === 'PM'
  const githubByType = serverStats?.github_events.by_type ?? {}
  const githubPrEvents = githubByType.pull_request ?? 0
  const githubPrReviewEvents = githubByType.pull_request_review ?? 0
  const githubPrCommentEvents =
    (githubByType.pull_request_review_comment ?? 0) +
    (githubByType.issue_comment ?? 0)
  const githubStatusCheckEvents =
    (githubByType.check_run ?? 0) +
    (githubByType.check_suite ?? 0) +
    (githubByType.status ?? 0)
  const githubEvidenceSummary = buildGitHubEvidenceSummary({
    pull_request: githubPrEvents,
    pull_request_review: githubPrReviewEvents,
    pull_request_review_comment: githubPrCommentEvents,
    check_run: githubStatusCheckEvents,
  })
  const pipeline = serverStats?.pipeline
  const pipelineTotal = pipeline?.total_7d ?? 0
  const pipelineFailures = pipeline?.failure_7d ?? 0
  const pipelineSuccessRate =
    pipelineTotal > 0 ? ((pipeline?.success_7d ?? 0) / pipelineTotal) * 100 : null
  const ticketCoveragePercent = ticketCoverage?.coverage_percentage ?? null
  const traceabilityGapCount =
    (ticketCoverage?.commits_without_ticket.length ?? 0) +
    (ticketCoverage?.tickets_without_commits.length ?? 0)
  const criticalViolations = serverStats?.violations.critical ?? 0
  const blockedToday = serverStats?.client_events.blocked_today ?? 0
  const evidenceGapCount = traceabilityGapCount + pipelineFailures + criticalViolations
  const sonarPipelines = jenkinsCorrelations.filter(
    (entry) => entry.pipeline && entry.pipeline.job_name.toLowerCase().includes('sonar'),
  )
  const sonarTotal = sonarPipelines.length
  const sonarPassed = sonarPipelines.filter((entry) => entry.pipeline?.status === 'success').length
  const sonarPassRate = sonarTotal > 0 ? (sonarPassed / sonarTotal) * 100 : null
  const releaseTierProfile = getRepoTierProfile('standard')
  const releaseReadiness = computeReleaseReadiness({
    tier: 'standard',
    pipelineSuccessRate: pipelineSuccessRate ?? 0,
    ticketCoveragePercent: ticketCoveragePercent ?? 0,
    sonarPassRate: sonarPassRate ?? 0,
    pipelineAvailable: pipelineTotal > 0,
    ticketCoverageAvailable: (ticketCoverage?.total_commits ?? 0) > 0,
    sonarAvailable: sonarTotal > 0,
  })
  const [githubEvidenceTrend, setGitHubEvidenceTrend] = useState<GitHubEvidenceTrendPoint[]>(() =>
    readStoredGitHubEvidenceTrend(),
  )
  const captureGitHubEvidenceSnapshot = () => {
    if (!serverStats) return
    const next = appendGitHubEvidenceTrendPoint(
      githubEvidenceTrend,
      buildGitHubEvidenceTrendPoint(githubEvidenceSummary),
    )
    setGitHubEvidenceTrend(next)
    persistGitHubEvidenceTrend(next)
  }

  useEffect(() => {
    if (!isConnected) return
    void connectSse()
    return () => disconnectSse()
  }, [connectSse, disconnectSse, isConnected])

  useEffect(() => {
    if (!isConnected) return
    if (userRole === 'Admin') {
      void loadStats()
    }
    if (activeSection === 'evidence') {
      void loadLogsIncremental(GOVERNANCE_LOG_LIMIT)
    }
  }, [activeSection, isConnected, loadLogsIncremental, loadStats, userRole])

  const renderSection = () => {
    if (activeSection === 'evidence') {
      return (
        <div className="space-y-3">
          <div className="grid grid-cols-1 gap-2 md:grid-cols-2 xl:grid-cols-4">
            <SummaryMetric
              title={t('governance.metrics.traceability')}
              value={formatPercent(ticketCoveragePercent)}
              detail={t('governance.metrics.traceabilityDetail', {
                withTicket: ticketCoverage?.commits_with_ticket ?? 0,
                total: ticketCoverage?.total_commits ?? 0,
              })}
              healthy={(ticketCoveragePercent ?? 0) >= 85}
            />
            <SummaryMetric
              title={t('governance.metrics.pipelineEvidence')}
              value={formatPercent(pipelineSuccessRate)}
              detail={t('governance.metrics.pipelineDetail', {
                total: pipelineTotal,
                failures: pipelineFailures,
              })}
              healthy={pipelineTotal > 0 && pipelineFailures === 0}
            />
            <SummaryMetric
              title={t('governance.metrics.githubSignals')}
              value={`${githubEvidenceSummary.activeSignals}/${githubEvidenceSummary.totalSignals}`}
              detail={githubEvidenceSummary.missingSignals.length > 0
                ? t('governance.metrics.githubSignalsMissing', { signals: githubEvidenceSummary.missingSignals.join(', ') })
                : t('governance.metrics.githubSignalsReady')}
              healthy={githubEvidenceSummary.activeSignals === githubEvidenceSummary.totalSignals}
            />
            <SummaryMetric
              title={t('governance.metrics.evidenceGaps')}
              value={`${evidenceGapCount}`}
              detail={t('governance.metrics.evidenceGapsDetail', {
                traceability: traceabilityGapCount,
                blocked: blockedToday,
                critical: criticalViolations,
              })}
              healthy={evidenceGapCount === 0 && blockedToday === 0}
            />
          </div>
          {isAdmin ? (
            <>
              <EvidencePacketPanel />
              <EventBreakdownGrid
                githubByType={githubByType}
                clientByStatus={serverStats?.client_events.by_status ?? {}}
                commitsWithoutTicket={(ticketCoverage?.commits_without_ticket ?? []).slice(0, 5)}
                ticketsWithoutCommits={(ticketCoverage?.tickets_without_commits ?? []).slice(0, 5)}
                totalCommitsWithoutTicket={ticketCoverage?.commits_without_ticket.length ?? 0}
                totalTicketsWithoutCommits={ticketCoverage?.tickets_without_commits.length ?? 0}
              />
              <GitHubEvidenceTrendWidget
                points={githubEvidenceTrend}
                onCapture={captureGitHubEvidenceSnapshot}
              />
              <RecentCommitsTable />
              <ExportPanel githubByType={githubByType} />
            </>
          ) : (
            <>
              <DeveloperAccessPanel />
              <RecentCommitsTable />
            </>
          )}
        </div>
      )
    }

    if (activeSection === 'policy') {
      return isAdmin ? <PolicyEditorPanel /> : <GovernanceAccessNotice />
    }

    if (activeSection === 'adoption') {
      return isAdmin ? (
        <div className="space-y-3">
          <FirstGovernedRepoSetupPanel />
          <EnterpriseAdoptionPanel />
        </div>
      ) : (
        <GovernanceAccessNotice />
      )
    }

    if (activeSection === 'releases') {
      return isAdmin ? (
        <div className="space-y-3">
          <div className="grid grid-cols-1 gap-2 md:grid-cols-3">
            <SummaryMetric
              title={t('governance.metrics.releaseReadiness')}
              value={`${releaseReadiness.score}/100`}
              detail={t('governance.metrics.releaseReadinessDetail', {
                band: releaseReadiness.band,
                target: releaseTierProfile.risk.sla.minReadinessScore,
              })}
              healthy={releaseReadiness.score >= releaseTierProfile.risk.sla.minReadinessScore}
            />
            <SummaryMetric
              title={t('governance.metrics.qualityEvidence')}
              value={formatPercent(sonarPassRate)}
              detail={t('governance.metrics.qualityEvidenceDetail', {
                passed: sonarPassed,
                total: sonarTotal,
              })}
              healthy={sonarTotal > 0 && sonarPassed === sonarTotal}
            />
            <SummaryMetric
              title={t('governance.metrics.releaseBlockers')}
              value={`${criticalViolations + pipelineFailures}`}
              detail={t('governance.metrics.releaseBlockersDetail', {
                critical: criticalViolations,
                failures: pipelineFailures,
              })}
              healthy={criticalViolations + pipelineFailures === 0}
            />
          </div>
          <ReleaseApprovalPanel />
          <ComplianceEvidenceFlowPanel />
          <DeploymentGateHistoryPanel />
        </div>
      ) : <GovernanceAccessNotice />
    }

    return canUseCopilot ? <GovernanceCopilotPanel /> : <GovernanceAccessNotice />
  }

  return (
    <div className="h-full overflow-auto bg-surface-950">
      <div className="p-5">
        <div className="mb-4 flex flex-col gap-3 xl:flex-row xl:items-end xl:justify-between">
          <div>
            <div className="flex items-center gap-2">
              <Landmark size={16} className="text-brand-400" />
              <h1 className="text-sm font-semibold text-white">{t('governance.title')}</h1>
            </div>
            <p className="mt-1 max-w-3xl text-xs text-surface-500">
              {t('governance.body')}
            </p>
          </div>
        </div>

        <nav className="mb-4 grid grid-cols-1 gap-2 md:grid-cols-3 xl:grid-cols-5">
          {GOVERNANCE_SECTIONS.map((item) => (
            <NavLink
              key={item.id}
              to={`/governance/${item.id}`}
              className={({ isActive }) =>
                clsx(
                  'rounded-lg border p-3 text-left transition-colors',
                  isActive || activeSection === item.id
                    ? 'border-brand-500/45 bg-brand-500/10 text-surface-100'
                    : 'border-white/8 bg-white/[0.02] text-surface-300 hover:border-white/20 hover:bg-white/[0.04]',
                )
              }
            >
              <span className="flex items-center gap-2 text-xs font-semibold">
                <item.icon size={14} className="text-brand-300" />
                {t(item.labelKey)}
              </span>
              <span className="mt-1 block text-[11px] leading-4 text-surface-500">
                {t(item.descriptionKey)}
              </span>
            </NavLink>
          ))}
        </nav>

        <div className="animate-fade-in">{renderSection()}</div>
      </div>
    </div>
  )
}
