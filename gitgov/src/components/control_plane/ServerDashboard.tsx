import { useEffect, useRef, useState } from 'react'
import { useControlPlaneStore } from '@/store/useControlPlaneStore'
import { Server } from 'lucide-react'
import { formatTs } from '@/lib/timezone'
import { DashboardHeader } from './DashboardHeader'
import { MetricsGrid } from './MetricsGrid'
import { PipelineHealthWidget } from './PipelineHealthWidget'
import { DailyActivityWidget } from './DailyActivityWidget'
import { TicketCoverageWidget } from './TicketCoverageWidget'
import { EventBreakdownGrid } from './EventBreakdownGrid'
import { RiskOutcomesWidget } from './RiskOutcomesWidget'
import { RecentCommitsTable } from './RecentCommitsTable'
import { DeveloperAccessPanel } from './DeveloperAccessPanel'
import { ConversationalChatPanel } from './ConversationalChatPanel'
import { PolicyEditorPanel } from './PolicyEditorPanel'
import { ExportPanel } from './ExportPanel'
import { MaintenanceOverlay } from './MaintenanceOverlay'
import { Modal } from '@/components/shared/Modal'
import { Badge } from '@/components/shared/Badge'

const DASHBOARD_LOG_LIMIT = 500

export function ServerDashboard() {
  const serverStats = useControlPlaneStore((s) => s.serverStats)
  const dailyActivity = useControlPlaneStore((s) => s.dailyActivity)
  const ticketCoverage = useControlPlaneStore((s) => s.ticketCoverage)
  const jenkinsCorrelations = useControlPlaneStore((s) => s.jenkinsCorrelations)
  const userRole = useControlPlaneStore((s) => s.userRole)
  const isConnected = useControlPlaneStore((s) => s.isConnected)
  const connectionStatus = useControlPlaneStore((s) => s.connectionStatus)
  const isRefreshingDashboard = useControlPlaneStore((s) => s.isRefreshingDashboard)
  const refreshForCurrentRole = useControlPlaneStore((s) => s.refreshForCurrentRole)
  const loadLogs = useControlPlaneStore((s) => s.loadLogs)
  const loadLogsIncremental = useControlPlaneStore((s) => s.loadLogsIncremental)
  const activeDevs7d = useControlPlaneStore((s) => s.activeDevs7d)
  const activeDevs7dUpdatedAt = useControlPlaneStore((s) => s.activeDevs7dUpdatedAt)
  const loadActiveDevs7d = useControlPlaneStore((s) => s.loadActiveDevs7d)
  const displayTimezone = useControlPlaneStore((s) => s.displayTimezone)
  const isChatLoading = useControlPlaneStore((s) => s.isChatLoading)
  const sseConnected = useControlPlaneStore((s) => s.sseConnected)
  const connectSse = useControlPlaneStore((s) => s.connectSse)
  const disconnectSse = useControlPlaneStore((s) => s.disconnectSse)

  const isAdmin = userRole === 'Admin'
  const canUseGovernanceChat =
    userRole === 'Admin' || userRole === 'Architect' || userRole === 'PM'

  const [autoRefresh, setAutoRefresh] = useState(true)
  const [showActiveDevsModal, setShowActiveDevsModal] = useState(false)
  const [isWindowVisible, setIsWindowVisible] = useState(
    typeof document === 'undefined' ? true : document.visibilityState === 'visible',
  )
  const isChatLoadingRef = useRef(isChatLoading)

  useEffect(() => {
    isChatLoadingRef.current = isChatLoading
  }, [isChatLoading])

  useEffect(() => {
    const onVisibilityChange = () => {
      setIsWindowVisible(document.visibilityState === 'visible')
    }
    document.addEventListener('visibilitychange', onVisibilityChange)
    return () => document.removeEventListener('visibilitychange', onVisibilityChange)
  }, [])

  // Connect SSE when dashboard mounts + connected; disconnect on unmount
  useEffect(() => {
    if (!isConnected) return
    void connectSse()
    return () => disconnectSse()
  }, [isConnected, connectSse, disconnectSse])

  // Initial data load on connect
  useEffect(() => {
    if (!isConnected) return
    if (userRole === 'Admin') {
      void refreshForCurrentRole()
    } else {
      void loadLogsIncremental(DASHBOARD_LOG_LIMIT)
    }
  }, [isConnected, refreshForCurrentRole, loadLogsIncremental, userRole])

  // Polling fallback: only active when SSE is NOT connected
  useEffect(() => {
    if (!isConnected || !autoRefresh || sseConnected) return

    const interval = setInterval(() => {
      if (isChatLoadingRef.current) return
      if (!isWindowVisible) return
      if (userRole === 'Admin') {
        void refreshForCurrentRole()
      } else {
        void loadLogsIncremental(DASHBOARD_LOG_LIMIT)
      }
    }, 30000)
    return () => clearInterval(interval)
  }, [isConnected, autoRefresh, sseConnected, refreshForCurrentRole, loadLogsIncremental, userRole, isWindowVisible])

  // Heavy refresh (Jenkins/Jira/PR correlations) — every 5 min regardless of SSE
  useEffect(() => {
    if (!isConnected || !autoRefresh || userRole !== 'Admin') return
    const interval = setInterval(() => {
      if (!isWindowVisible) return
      void refreshForCurrentRole({ forceHeavy: true })
    }, 5 * 60 * 1000)
    return () => clearInterval(interval)
  }, [isConnected, autoRefresh, userRole, refreshForCurrentRole, isWindowVisible])

  /* ── maintenance mode ── */
  if (connectionStatus === 'maintenance') {
    return <MaintenanceOverlay />
  }

  /* ── not connected ── */
  if (!isConnected) {
    return (
      <div className="flex flex-col items-center justify-center h-64 animate-fade-in">
        <Server size={32} strokeWidth={1.5} className="text-surface-700 mb-3" />
        <p className="text-xs font-medium text-surface-400">Conecta al Control Plane</p>
        <p className="text-[10px] text-surface-600 mt-1">Configura la conexión para ver el dashboard</p>
      </div>
    )
  }

  /* ── derived data ── */
  const successRate = serverStats
    ? serverStats.github_events.pushes_today + serverStats.client_events.blocked_today > 0
      ? ((serverStats.github_events.pushes_today / (serverStats.github_events.pushes_today + serverStats.client_events.blocked_today)) * 100).toFixed(1)
      : '100.0'
    : '0'
  const githubPushesToday = serverStats?.github_events.pushes_today ?? 0
  const desktopPushesToday = serverStats?.client_events.desktop_pushes_today ?? 0
  const totalTrackedPushesToday = githubPushesToday + desktopPushesToday
  const pipeline = serverStats?.pipeline
  const pipelineTotal = pipeline?.total_7d ?? 0
  const pipelineSuccessRate = pipelineTotal > 0 ? (((pipeline?.success_7d ?? 0) / pipelineTotal) * 100).toFixed(1) : '0.0'
  const pipelineSuccessRateValue = Number.parseFloat(pipelineSuccessRate)
  const sonarPipelines = jenkinsCorrelations.filter(
    (entry) => entry.pipeline && entry.pipeline.job_name.toLowerCase().includes('sonar'),
  )
  const sonarTotal = sonarPipelines.length
  const sonarPassed = sonarPipelines.filter((entry) => entry.pipeline?.status === 'success').length
  const sonarFailed = sonarPipelines.filter((entry) => entry.pipeline?.status === 'failure').length
  const sonarUnstable = sonarPipelines.filter(
    (entry) =>
      entry.pipeline?.status !== 'success' && entry.pipeline?.status !== 'failure',
  ).length
  const sonarPassRate = sonarTotal > 0 ? ((sonarPassed / sonarTotal) * 100).toFixed(1) : '0.0'
  const sonarPassRateValue = Number.parseFloat(sonarPassRate)
  const ticketCoveragePercent = ticketCoverage?.coverage_percentage ?? 0
  const readinessSignals = [
    { value: pipelineSuccessRateValue, weight: 0.45, available: pipelineTotal > 0 },
    { value: ticketCoveragePercent, weight: 0.25, available: (ticketCoverage?.total_commits ?? 0) > 0 },
    { value: sonarPassRateValue, weight: 0.30, available: sonarTotal > 0 },
  ]
  const readinessWeight = readinessSignals.reduce((acc, signal) => (
    signal.available ? acc + signal.weight : acc
  ), 0)
  const releaseReadinessScore = readinessWeight > 0
    ? Math.round(
      readinessSignals.reduce((acc, signal) => (
        signal.available ? acc + (signal.value * signal.weight) : acc
      ), 0) / readinessWeight,
    )
    : 0
  const releaseReadinessSignals = readinessSignals.filter((signal) => signal.available).length
  const commitsWithoutTicket = (ticketCoverage?.commits_without_ticket ?? []).slice(0, 5)
  const likelyTestActiveDevs = activeDevs7d.filter((d) => d.suspicious_test_data).length
  const activeDevCoverage = serverStats ? `${activeDevs7d.length}/${serverStats.active_devs_week}` : `${activeDevs7d.length}/-`
  const violationsTotal = serverStats?.violations.total ?? 0
  const unresolvedViolations = serverStats?.violations.unresolved ?? 0
  const criticalViolations = serverStats?.violations.critical ?? 0

  return (
    <div className="space-y-3 animate-fade-in">
      <DashboardHeader
        autoRefresh={autoRefresh}
        onAutoRefreshChange={setAutoRefresh}
        onRefresh={() => {
          if (isChatLoading) return
          if (userRole === 'Admin') {
            void refreshForCurrentRole({ forceHeavy: true })
          } else {
            void loadLogs(DASHBOARD_LOG_LIMIT, 0)
          }
        }}
        isRefreshing={isRefreshingDashboard}
      />
      <div className="flex justify-end">
        <span className="text-[9px] text-surface-500 uppercase tracking-widest bg-white/4 px-2 py-0.5 rounded font-medium">TZ: {displayTimezone}</span>
      </div>

      {isAdmin && serverStats && (
        <>
          <MetricsGrid
            totalGithubEvents={serverStats.github_events.total}
            successRate={successRate}
            activeRepos={serverStats.active_repos}
            desktopPushesToday={desktopPushesToday}
            githubPushesToday={githubPushesToday}
            totalTrackedPushesToday={totalTrackedPushesToday}
            blockedToday={serverStats.client_events.blocked_today}
            activeDevsWeek={serverStats.active_devs_week}
            onOpenActiveDevs={() => setShowActiveDevsModal(true)}
          />

          <div className="grid grid-cols-1 xl:grid-cols-3 gap-3">
            <PipelineHealthWidget
              total={pipelineTotal}
              failure={pipeline?.failure_7d ?? 0}
              avgDurationMs={pipeline?.avg_duration_ms_7d ?? 0}
              reposWithFailures={pipeline?.repos_with_failures_7d ?? 0}
              successRate={pipelineSuccessRate}
              sonarTotal={sonarTotal}
              sonarPassed={sonarPassed}
              sonarFailed={sonarFailed}
              sonarUnstable={sonarUnstable}
              sonarPassRate={sonarPassRate}
              releaseReadinessScore={releaseReadinessScore}
              releaseReadinessSignals={releaseReadinessSignals}
            />
            <DailyActivityWidget points={dailyActivity} />
            <TicketCoverageWidget />
          </div>

          <RiskOutcomesWidget
            trackedPushesToday={totalTrackedPushesToday}
            blockedPushesToday={serverStats.client_events.blocked_today}
            ticketCoveragePercent={ticketCoveragePercent}
            pipelineTotal7d={pipelineTotal}
            pipelineFailure7d={pipeline?.failure_7d ?? 0}
            sonarTotal={sonarTotal}
            sonarFailed={sonarFailed}
            unresolvedViolations={unresolvedViolations}
            totalViolations={violationsTotal}
            criticalViolations={criticalViolations}
            releaseReadinessScore={releaseReadinessScore}
          />

          <EventBreakdownGrid
            githubByType={serverStats.github_events.by_type}
            clientByStatus={serverStats.client_events.by_status}
            commitsWithoutTicket={commitsWithoutTicket}
            ticketsWithoutCommits={(ticketCoverage?.tickets_without_commits ?? []).slice(0, 5)}
            totalCommitsWithoutTicket={ticketCoverage?.commits_without_ticket.length ?? 0}
            totalTicketsWithoutCommits={ticketCoverage?.tickets_without_commits.length ?? 0}
          />

          <RecentCommitsTable />

          <PolicyEditorPanel />

          <ExportPanel />

          <ConversationalChatPanel />

          <Modal
            isOpen={showActiveDevsModal}
            onClose={() => setShowActiveDevsModal(false)}
            title="Detalle: Devs Activos 7d"
            size="xl"
          >
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <div className="text-[11px] text-surface-400">
                  Visibles en muestra: <span className="mono-data text-surface-200">{activeDevCoverage}</span>
                  <span className="ml-2 text-surface-600">(ventana de logs, no forense completa)</span>
                </div>
                <button
                  type="button"
                  onClick={() => void loadActiveDevs7d()}
                  className="text-[10px] text-brand-400 hover:text-brand-300 transition-colors"
                >
                  Actualizar lista
                </button>
              </div>

              <div className="flex items-center gap-2 text-[10px]">
                <Badge variant="neutral">
                  al parecer de test: {likelyTestActiveDevs}
                </Badge>
                {activeDevs7dUpdatedAt && (
                  <span className="text-surface-600">actualizado: {formatTs(activeDevs7dUpdatedAt, displayTimezone)}</span>
                )}
              </div>

              <div className="max-h-[420px] overflow-auto border border-white/6 rounded-lg">
                <table className="w-full">
                  <thead className="sticky top-0 bg-surface-800">
                    <tr className="text-left text-[9px] text-surface-600 uppercase tracking-widest">
                      <th className="py-2 px-3 font-medium">Usuario</th>
                      <th className="py-2 px-3 font-medium">Eventos 7d</th>
                      <th className="py-2 px-3 font-medium">Último evento</th>
                      <th className="py-2 px-3 font-medium">Señal</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-white/3">
                    {activeDevs7d.map((dev) => (
                      <tr key={dev.user_login} className="hover:bg-white/2">
                        <td className="py-2 px-3 text-[11px] text-surface-200 font-medium">{dev.user_login}</td>
                        <td className="py-2 px-3 text-[11px] text-surface-300 mono-data">{dev.events}</td>
                        <td className="py-2 px-3 text-[10px] text-surface-500">{formatTs(dev.last_seen, displayTimezone)}</td>
                        <td className="py-2 px-3">
                          {dev.suspicious_test_data
                            ? <Badge variant="neutral">aparente test</Badge>
                            : <Badge variant="success">ok</Badge>}
                        </td>
                      </tr>
                    ))}
                    {activeDevs7d.length === 0 && (
                      <tr>
                        <td colSpan={4} className="py-8 text-center text-[11px] text-surface-600">Sin datos en la ventana actual.</td>
                      </tr>
                    )}
                  </tbody>
                </table>
              </div>
            </div>
          </Modal>
        </>
      )}

      {!isAdmin && (
        <>
          <DeveloperAccessPanel />
          <RecentCommitsTable />
          {canUseGovernanceChat && <ConversationalChatPanel />}
        </>
      )}
    </div>
  )
}
