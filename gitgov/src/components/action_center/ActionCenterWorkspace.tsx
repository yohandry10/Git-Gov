import { useEffect, useMemo, useState } from 'react'
import { Link } from 'react-router-dom'
import clsx from 'clsx'
import {
  Bot,
  CheckCircle2,
  ClipboardCheck,
  Compass,
  ExternalLink,
  FileCheck2,
  GitBranch,
  RefreshCw,
  Rocket,
  ShieldCheck,
  UserCog,
  Workflow,
} from 'lucide-react'
import { Badge } from '@/components/shared/Badge'
import { Button } from '@/components/shared/Button'
import { useControlPlaneStore } from '@/store/useControlPlaneStore'
import {
  DEFAULT_ENTERPRISE_ADOPTION_PROFILE,
  buildEnterpriseAdoptionPack,
  buildEnterpriseOnboardingGuide,
  buildEnterpriseOnboardingReadinessReport,
  buildEnterpriseOnboardingRemediationPlan,
  buildEnterpriseProviderHealth,
  normalizeEnterpriseAdoptionProfile,
  validateEnterpriseAdoptionProfile,
} from '@/components/control_plane/dashboard-helpers'
import {
  ACTION_CENTER_GOALS,
  ACTION_CENTER_LENSES,
  buildActionCenterGuidance,
  type ActionCenterAction,
  type ActionCenterConfidence,
  type ActionCenterGoal,
  type ActionCenterLens,
  type ActionCenterRecommendation,
  type ActionCenterRecommendationStatus,
} from './action-center-helpers'

const GOAL_ICONS: Record<ActionCenterGoal, typeof Rocket> = {
  'quick-onboarding': Rocket,
  'prepare-release': ShieldCheck,
  'export-evidence': FileCheck2,
}

const LENS_ICONS: Record<ActionCenterLens, typeof UserCog> = {
  founder: Compass,
  developer: GitBranch,
  executive: ClipboardCheck,
  platform: Workflow,
  auditor: FileCheck2,
}

function statusVariant(status: ActionCenterRecommendationStatus): 'success' | 'warning' | 'danger' | 'info' {
  if (status === 'ready') return 'success'
  if (status === 'blocked') return 'danger'
  return 'warning'
}

function statusLabel(status: ActionCenterRecommendationStatus): string {
  if (status === 'ready') return 'Ready'
  if (status === 'blocked') return 'Blocked'
  return 'Action'
}

function confidenceVariant(confidence: ActionCenterConfidence): 'success' | 'warning' | 'info' {
  if (confidence === 'high') return 'success'
  if (confidence === 'medium') return 'info'
  return 'warning'
}

function actionLabel(action: ActionCenterAction): string {
  if (action.kind === 'export') return 'Export'
  if (action.kind === 'review') return 'Review'
  return 'Open'
}

function RecommendationPanel({
  recommendation,
  primary = false,
}: {
  recommendation: ActionCenterRecommendation
  primary?: boolean
}) {
  return (
    <section
      className={clsx(
        'rounded-lg border p-4 bg-white/[0.03]',
        primary
          ? 'border-brand-500/30 shadow-sm shadow-brand-950/20'
          : 'border-white/8',
      )}
    >
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="min-w-0">
          <div className="flex flex-wrap items-center gap-2">
            <Badge variant={statusVariant(recommendation.status)}>
              {statusLabel(recommendation.status)}
            </Badge>
            <Badge variant={confidenceVariant(recommendation.confidence)}>
              {recommendation.confidence}
            </Badge>
            <Badge variant={recommendation.permission.canAct ? 'success' : 'warning'}>
              {recommendation.permission.label}
            </Badge>
          </div>
          <h2 className={clsx('mt-3 font-semibold text-white tracking-normal', primary ? 'text-lg' : 'text-sm')}>
            {recommendation.title}
          </h2>
          <p className="mt-2 text-xs leading-5 text-surface-400">
            {recommendation.outcome}
          </p>
        </div>
        <Link
          to={recommendation.primaryAction.to}
          className="inline-flex items-center justify-center gap-1.5 rounded-lg border border-brand-500/40 px-3 py-2 text-xs font-medium text-brand-300 hover:border-brand-400 hover:bg-brand-500/10 transition-colors"
          title={recommendation.primaryAction.label}
        >
          <ExternalLink size={13} />
          {actionLabel(recommendation.primaryAction)}
        </Link>
      </div>

      <div className="mt-4 rounded border border-white/6 bg-surface-950/40 p-3">
        <div className="text-[10px] uppercase tracking-widest text-surface-600">Reason</div>
        <p className="mt-1 text-xs leading-5 text-surface-300">{recommendation.reason}</p>
        <p className="mt-2 text-[11px] text-surface-500">{recommendation.permission.detail}</p>
      </div>

      <div className="mt-3 grid grid-cols-1 md:grid-cols-2 gap-2">
        {recommendation.evidence.map((line) => (
          <div key={`${recommendation.id}-${line.label}`} className="rounded border border-white/6 bg-white/[0.02] p-3">
            <div className="flex items-center justify-between gap-2">
              <span className="text-[10px] uppercase tracking-widest text-surface-600">{line.label}</span>
              <Badge variant={statusVariant(line.state)}>{statusLabel(line.state)}</Badge>
            </div>
            <div className="mt-1 text-xs text-surface-200 break-words">{line.value}</div>
          </div>
        ))}
      </div>
    </section>
  )
}

export function ActionCenterWorkspace() {
  const [goal, setGoal] = useState<ActionCenterGoal>('quick-onboarding')
  const [lens, setLens] = useState<ActionCenterLens>('founder')

  const isConnected = useControlPlaneStore((state) => state.isConnected)
  const userRole = useControlPlaneStore((state) => state.userRole)
  const selectedOrgName = useControlPlaneStore((state) => state.selectedOrgName)
  const serverStats = useControlPlaneStore((state) => state.serverStats)
  const ticketCoverage = useControlPlaneStore((state) => state.ticketCoverage)
  const jenkinsCorrelations = useControlPlaneStore((state) => state.jenkinsCorrelations)
  const evidencePacket = useControlPlaneStore((state) => state.evidencePacket)
  const releaseApprovalsTotal = useControlPlaneStore((state) => state.releaseApprovalsTotal)
  const persistedProfile = useControlPlaneStore((state) => state.enterpriseAdoptionProfile)
  const isRefreshingDashboard = useControlPlaneStore((state) => state.isRefreshingDashboard)
  const refreshForCurrentRole = useControlPlaneStore((state) => state.refreshForCurrentRole)
  const loadEnterpriseAdoptionProfile = useControlPlaneStore((state) => state.loadEnterpriseAdoptionProfile)
  const loadEnterpriseOnboardingChecklistTracking = useControlPlaneStore((state) => state.loadEnterpriseOnboardingChecklistTracking)

  useEffect(() => {
    if (!isConnected) return
    void refreshForCurrentRole({ forceHeavy: true })
    void loadEnterpriseAdoptionProfile(selectedOrgName || undefined)
    void loadEnterpriseOnboardingChecklistTracking(selectedOrgName || undefined)
  }, [
    isConnected,
    loadEnterpriseAdoptionProfile,
    loadEnterpriseOnboardingChecklistTracking,
    refreshForCurrentRole,
    selectedOrgName,
  ])

  const profile = useMemo(
    () => normalizeEnterpriseAdoptionProfile(persistedProfile ?? DEFAULT_ENTERPRISE_ADOPTION_PROFILE),
    [persistedProfile],
  )
  const pack = useMemo(() => buildEnterpriseAdoptionPack(profile), [profile])
  const validation = useMemo(() => validateEnterpriseAdoptionProfile(profile), [profile])
  const sonarRuns = useMemo(
    () => jenkinsCorrelations.filter((entry) => entry.pipeline?.job_name.toLowerCase().includes('sonar')).length,
    [jenkinsCorrelations],
  )
  const sonarSuccessful = useMemo(
    () => jenkinsCorrelations.filter((entry) =>
      entry.pipeline?.job_name.toLowerCase().includes('sonar') && entry.pipeline.status === 'success',
    ).length,
    [jenkinsCorrelations],
  )
  const providerHealth = useMemo(() => buildEnterpriseProviderHealth(profile, {
    githubEventsTotal: serverStats?.github_events.total,
    githubEventTypes: serverStats?.github_events.by_type,
    jiraCommitsWithTicket: ticketCoverage?.commits_with_ticket,
    jiraCoveragePercentage: ticketCoverage?.coverage_percentage,
    pipelineRuns7d: serverStats?.pipeline?.total_7d,
    pipelineSuccess7d: serverStats?.pipeline?.success_7d,
    sonarRuns,
    sonarSuccessful,
    activeRepos: serverStats?.active_repos,
  }, pack), [pack, profile, serverStats, sonarRuns, sonarSuccessful, ticketCoverage])
  const readiness = useMemo(
    () => buildEnterpriseOnboardingReadinessReport(profile, providerHealth),
    [profile, providerHealth],
  )
  const remediationPlan = useMemo(
    () => buildEnterpriseOnboardingRemediationPlan(readiness, pack),
    [pack, readiness],
  )
  const guide = useMemo(
    () => buildEnterpriseOnboardingGuide(readiness, remediationPlan),
    [readiness, remediationPlan],
  )
  const guidance = useMemo(() => buildActionCenterGuidance({
    goal,
    lens,
    isConnected,
    userRole,
    profile,
    pack,
    validation,
    providerHealth,
    readiness,
    remediationPlan,
    guide,
    pipeline: serverStats?.pipeline ?? null,
    ticketCoverage,
    evidencePacket,
    releaseApprovalsTotal,
  }), [
    evidencePacket,
    goal,
    guide,
    isConnected,
    lens,
    pack,
    profile,
    providerHealth,
    readiness,
    releaseApprovalsTotal,
    remediationPlan,
    serverStats,
    ticketCoverage,
    userRole,
    validation,
  ])

  const refresh = () => {
    if (!isConnected) return
    void refreshForCurrentRole({ forceHeavy: true })
    void loadEnterpriseAdoptionProfile(selectedOrgName || undefined)
    void loadEnterpriseOnboardingChecklistTracking(selectedOrgName || undefined)
  }

  return (
    <div className="p-5 space-y-4 animate-fade-in">
      <div className="flex flex-col xl:flex-row xl:items-start xl:justify-between gap-4">
        <div className="min-w-0">
          <div className="flex flex-wrap items-center gap-2">
            <Compass size={16} className="text-brand-400" />
            <h1 className="text-lg font-semibold text-white tracking-normal">Action Center</h1>
            <Badge variant="info">Advisory</Badge>
            <Badge variant={isConnected ? 'success' : 'warning'}>
              {isConnected ? userRole ?? 'Connected' : 'Disconnected'}
            </Badge>
          </div>
          <p className="mt-2 max-w-3xl text-xs leading-5 text-surface-400">
            {guidance.lensNote}
          </p>
        </div>
        <Button
          variant="secondary"
          size="sm"
          loading={isRefreshingDashboard}
          disabled={!isConnected}
          onClick={refresh}
          title="Refresh Action Center evidence"
        >
          <RefreshCw size={14} />
          Refresh
        </Button>
      </div>

      <div className="grid grid-cols-1 2xl:grid-cols-[minmax(0,1fr)_360px] gap-4">
        <div className="space-y-4">
          <section className="rounded-lg border border-white/8 bg-white/[0.02] p-3">
            <div className="grid grid-cols-1 xl:grid-cols-[minmax(0,1fr)_minmax(0,1fr)] gap-3">
              <div>
                <div className="mb-2 text-[10px] uppercase tracking-widest text-surface-600">Goal</div>
                <div className="grid grid-cols-1 sm:grid-cols-3 gap-2">
                  {ACTION_CENTER_GOALS.map((option) => {
                    const Icon = GOAL_ICONS[option.id]
                    const selected = goal === option.id
                    return (
                      <button
                        key={option.id}
                        type="button"
                        onClick={() => setGoal(option.id)}
                        className={clsx(
                          'min-h-16 rounded-lg border p-3 text-left transition-colors',
                          selected
                            ? 'border-brand-500/50 bg-brand-500/10'
                            : 'border-white/8 bg-white/[0.02] hover:border-white/20',
                        )}
                      >
                        <span className="flex items-center gap-2 text-xs font-medium text-surface-100">
                          <Icon size={14} className={selected ? 'text-brand-300' : 'text-surface-500'} />
                          {option.label}
                        </span>
                        <span className="mt-1 block text-[11px] leading-4 text-surface-500">{option.description}</span>
                      </button>
                    )
                  })}
                </div>
              </div>

              <div>
                <div className="mb-2 text-[10px] uppercase tracking-widest text-surface-600">Lens</div>
                <div className="grid grid-cols-1 sm:grid-cols-5 xl:grid-cols-3 gap-2">
                  {ACTION_CENTER_LENSES.map((option) => {
                    const Icon = LENS_ICONS[option.id]
                    const selected = lens === option.id
                    return (
                      <button
                        key={option.id}
                        type="button"
                        onClick={() => setLens(option.id)}
                        className={clsx(
                          'min-h-10 rounded-lg border px-2.5 py-2 text-left transition-colors',
                          selected
                            ? 'border-brand-500/50 bg-brand-500/10'
                            : 'border-white/8 bg-white/[0.02] hover:border-white/20',
                        )}
                        title={option.description}
                      >
                        <span className="flex items-center gap-2 text-[11px] font-medium text-surface-100">
                          <Icon size={13} className={selected ? 'text-brand-300' : 'text-surface-500'} />
                          {option.label}
                        </span>
                      </button>
                    )
                  })}
                </div>
              </div>
            </div>
          </section>

          <RecommendationPanel recommendation={guidance.primary} primary />

          <div className="grid grid-cols-1 xl:grid-cols-3 gap-3">
            {guidance.secondary.map((recommendation) => (
              <RecommendationPanel key={recommendation.id} recommendation={recommendation} />
            ))}
          </div>
        </div>

        <aside className="space-y-3">
          <section className="rounded-lg border border-white/8 bg-white/[0.02] p-4">
            <div className="flex items-center gap-2 mb-3">
              <CheckCircle2 size={14} className="text-success-400" />
              <h2 className="text-sm font-semibold text-white tracking-normal">Current State</h2>
            </div>
            <div className="space-y-3 text-xs">
              <div>
                <div className="text-[10px] uppercase tracking-widest text-surface-600">Customer</div>
                <div className="mt-1 text-surface-200 break-words">{guidance.summary.customerName}</div>
              </div>
              <div>
                <div className="text-[10px] uppercase tracking-widest text-surface-600">Repository</div>
                <div className="mt-1 text-surface-200 break-words">{guidance.summary.repositoryFullName}</div>
              </div>
              <div className="grid grid-cols-2 gap-2">
                <div className="rounded border border-white/6 bg-surface-950/40 p-3">
                  <div className="text-[10px] text-surface-600">Readiness</div>
                  <div className="mt-1 mono-data text-surface-100">{guidance.summary.readinessScore}/100</div>
                  <div className="mt-1 text-[10px] text-surface-500">{guidance.summary.readinessStatus}</div>
                </div>
                <div className="rounded border border-white/6 bg-surface-950/40 p-3">
                  <div className="text-[10px] text-surface-600">Providers</div>
                  <div className="mt-1 mono-data text-surface-100">
                    {guidance.summary.providersReady}/{guidance.summary.providersTotal}
                  </div>
                  <div className="mt-1 text-[10px] text-surface-500">ready</div>
                </div>
                <div className="rounded border border-white/6 bg-surface-950/40 p-3">
                  <div className="text-[10px] text-surface-600">Traceability</div>
                  <div className="mt-1 mono-data text-surface-100">
                    {guidance.summary.ticketCoveragePercentage === null
                      ? 'N/A'
                      : `${guidance.summary.ticketCoveragePercentage.toFixed(1)}%`}
                  </div>
                  <div className="mt-1 text-[10px] text-surface-500">coverage</div>
                </div>
                <div className="rounded border border-white/6 bg-surface-950/40 p-3">
                  <div className="text-[10px] text-surface-600">Pipeline</div>
                  <div className="mt-1 mono-data text-surface-100">
                    {guidance.summary.pipelineSuccessRate === null
                      ? 'N/A'
                      : `${guidance.summary.pipelineSuccessRate}%`}
                  </div>
                  <div className="mt-1 text-[10px] text-surface-500">success</div>
                </div>
              </div>
              <div className="rounded border border-white/6 bg-surface-950/40 p-3">
                <div className="flex items-center justify-between gap-2">
                  <span className="text-[10px] uppercase tracking-widest text-surface-600">Release Policy</span>
                  <Badge variant={guidance.summary.releaseGovernanceMode === 'record-only' ? 'neutral' : 'warning'}>
                    {guidance.summary.releaseGovernanceMode}
                  </Badge>
                </div>
                <div className="mt-2 text-[11px] text-surface-400">
                  {guidance.summary.workflowTemplateCount} workflow template(s) in the adoption pack.
                </div>
              </div>
            </div>
          </section>

          <section className="rounded-lg border border-white/8 bg-white/[0.02] p-4">
            <div className="flex items-center gap-2 mb-3">
              <Bot size={14} className="text-brand-400" />
              <h2 className="text-sm font-semibold text-white tracking-normal">Decision Boundary</h2>
            </div>
            <div className="space-y-2 text-xs text-surface-400 leading-5">
              <p>GitGov ranks the visible next move from loaded evidence and configured policy.</p>
              <p>The operator can open any destination, change the goal, or use the workspace manually.</p>
            </div>
          </section>
        </aside>
      </div>
    </div>
  )
}
