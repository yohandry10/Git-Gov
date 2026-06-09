import { parseCommandError, tauriInvoke } from '@/lib/tauri'
import type { CombinedEvent, ServerStats } from '@/lib/types'
import type {
  CommitPipelineCorrelation,
  ControlPlaneActions,
  DailyActivityPoint,
  EvidencePacketResponse,
  JiraCorrelateResponse,
  JiraTicketDetailResponse,
  PrMergeEvidenceEntry,
  TicketCoverageStats,
} from '../types'
import type { ControlPlaneGet, ControlPlaneSet } from '../store-types'
import {
  DEFAULT_GOVERNANCE_LOG_WINDOW,
  DEV_ACTIVITY_WINDOW_MS,
  HEAVY_DASHBOARD_REFRESH_MS,
  JIRA_TICKET_CACHE_MAX,
  JIRA_TICKET_DETAIL_TTL_MS,
} from '../constants'
import { controlPlaneStoreRuntime } from '../runtime'
import {
  buildActiveDevs7dFromLogs,
  fetchLogsKeysetWindow,
  mergeRecentLogs,
  persistJiraCoverageFilters,
} from '../helpers'

type DashboardActionKeys =
  | 'refreshDashboardData'
  | 'loadStats'
  | 'loadDailyActivity'
  | 'loadLogs'
  | 'loadLogsIncremental'
  | 'loadActiveDevs7d'
  | 'setLogsPage'
  | 'loadJenkinsCorrelations'
  | 'loadPrMergeEvidence'
  | 'loadTicketCoverage'
  | 'applyTicketCoverageFilters'
  | 'correlateJiraTickets'
  | 'loadJiraTicketDetail'
  | 'loadTicketEvidencePacket'

export function createDashboardActions(
  set: ControlPlaneSet,
  get: ControlPlaneGet,
): Pick<ControlPlaneActions, DashboardActionKeys> {
  return {
  refreshDashboardData: async (params) => {
    const { serverConfig, jiraCoverageFilters } = get()
    if (!serverConfig) return

    set({ isRefreshingDashboard: true })
    try {
      const runStartedAt = Date.now()
      await Promise.all([
        get().loadStats(),
        get().loadLogsIncremental(params?.logLimit ?? DEFAULT_GOVERNANCE_LOG_WINDOW),
      ])

      const shouldRunHeavyRefresh =
        Boolean(params?.forceHeavy) ||
        controlPlaneStoreRuntime.lastHeavyDashboardRefreshAt === 0 ||
        runStartedAt - controlPlaneStoreRuntime.lastHeavyDashboardRefreshAt >= HEAVY_DASHBOARD_REFRESH_MS

      if (shouldRunHeavyRefresh) {
        await Promise.all([
          get().loadJenkinsCorrelations(50),
          get().loadPrMergeEvidence(200),
          get().loadTicketCoverage({
            hours: jiraCoverageFilters.hours,
            repo_full_name: jiraCoverageFilters.repo_full_name.trim() || undefined,
            branch: jiraCoverageFilters.branch.trim() || undefined,
          }),
        ])
        controlPlaneStoreRuntime.lastHeavyDashboardRefreshAt = Date.now()
      }

      const now = Date.now()
      const activeDevs7d = buildActiveDevs7dFromLogs(get().serverLogs, now)
      set({ activeDevs7d, activeDevs7dUpdatedAt: now })
    } finally {
      set({ isRefreshingDashboard: false })
    }
  },

  loadStats: async () => {
    const { serverConfig } = get()
    if (!serverConfig) return

    try {
      const stats = await tauriInvoke<ServerStats>('cmd_server_get_stats', { config: serverConfig })
      set({ serverStats: stats })
    } catch (e) {
      set({ error: parseCommandError(String(e)).message })
    }
  },

  loadDailyActivity: async (days = 14) => {
    const { serverConfig } = get()
    if (!serverConfig) return

    const safeDays = Number.isFinite(days) ? Math.max(1, Math.min(90, Math.floor(days))) : 14
    try {
      const points = await tauriInvoke<DailyActivityPoint[]>('cmd_server_get_daily_activity', {
        config: serverConfig,
        filter: { days: safeDays },
      })
      set({ dailyActivity: points })
    } catch {
      // Non-fatal: this widget should not break dashboard core flow.
    }
  },

  loadLogs: async (limit = 500, offset = 0) => {
    const { serverConfig } = get()
    if (!serverConfig) return
    try {
      const logs = await fetchLogsKeysetWindow(serverConfig, limit, offset)
      set({ serverLogs: logs })
    } catch (e) {
      set({ error: parseCommandError(String(e)).message })
    }
  },

  loadLogsIncremental: async (limit = 500) => {
    const { serverConfig, serverLogs } = get()
    if (!serverConfig) return

    const safeLimit = Math.max(1, Math.min(500, Math.floor(limit)))
    if (controlPlaneStoreRuntime.loadLogsIncrementalInFlight) {
      const inFlightLimit = controlPlaneStoreRuntime.loadLogsIncrementalInFlightLimit
      await controlPlaneStoreRuntime.loadLogsIncrementalInFlight
      if (safeLimit <= inFlightLimit) return
      await get().loadLogsIncremental(safeLimit)
      return
    }

    const run = (async () => {
      if (serverLogs.length === 0) {
        await get().loadLogs(safeLimit, 0)
        return
      }

      const latestTs = serverLogs.reduce((max, log) => log.created_at > max ? log.created_at : max, 0)
      try {
        const incoming = await tauriInvoke<CombinedEvent[]>('cmd_server_get_logs', {
          config: serverConfig,
          filter: {
            limit: Math.min(200, safeLimit),
            offset: 0,
            start_date: latestTs,
          },
        })

        if (incoming.length === 0) return
        const merged = mergeRecentLogs(serverLogs, incoming, safeLimit)
        set({ serverLogs: merged })
      } catch (e) {
        set({ error: parseCommandError(String(e)).message })
        // Conservative fallback: recover with full window if incremental call fails.
        await get().loadLogs(safeLimit, 0)
      }
    })()

    controlPlaneStoreRuntime.loadLogsIncrementalInFlight = run
    controlPlaneStoreRuntime.loadLogsIncrementalInFlightLimit = safeLimit
    try {
      await run
    } finally {
      if (controlPlaneStoreRuntime.loadLogsIncrementalInFlight === run) {
        controlPlaneStoreRuntime.loadLogsIncrementalInFlight = null
        controlPlaneStoreRuntime.loadLogsIncrementalInFlightLimit = 0
      }
    }
  },

  loadActiveDevs7d: async () => {
    const { serverConfig } = get()
    if (!serverConfig) return

    const now = Date.now()
    const start = now - DEV_ACTIVITY_WINDOW_MS
    try {
      const logs = await tauriInvoke<CombinedEvent[]>('cmd_server_get_logs', {
        config: serverConfig,
        filter: {
          limit: 500,
          offset: 0,
          start_date: start,
          end_date: now,
        },
      })
      const activeDevs7d = buildActiveDevs7dFromLogs(logs, now)

      set({ activeDevs7d, activeDevs7dUpdatedAt: now })
    } catch {
      // Non-fatal fallback: keep existing list if request fails.
    }
  },

  setLogsPage: (page) => set({ logsPage: page }),

  loadJenkinsCorrelations: async (limit = 50) => {
    const { serverConfig } = get()
    if (!serverConfig) return

    try {
      const correlations = await tauriInvoke<CommitPipelineCorrelation[]>('cmd_server_get_jenkins_correlations', {
        config: serverConfig,
        filter: { limit, offset: 0 },
      })
      set({ jenkinsCorrelations: correlations })
    } catch {
      // Non-fatal for the dashboard core flow; leave existing data as-is.
    }
  },

  loadPrMergeEvidence: async (limit = 200) => {
    const { serverConfig } = get()
    if (!serverConfig) return

    try {
      const entries = await tauriInvoke<PrMergeEvidenceEntry[]>('cmd_server_get_pr_merges', {
        config: serverConfig,
        filter: { limit, offset: 0 },
      })
      set({ prMergeEvidence: entries })
    } catch {
      // Non-fatal: PR evidence is additive to the dashboard core flow.
    }
  },

  loadTicketCoverage: async (params) => {
    const { serverConfig } = get()
    if (!serverConfig) return

    const hours = params?.hours ?? 72
    try {
      const coverage = await tauriInvoke<TicketCoverageStats>('cmd_server_get_jira_ticket_coverage', {
        config: serverConfig,
        query: {
          hours,
          repo_full_name: params?.repo_full_name,
          branch: params?.branch,
          org_name: params?.org_name,
        },
      })
      set({ ticketCoverage: coverage })
    } catch {
      // Non-fatal for dashboard core flow
    }
  },

  applyTicketCoverageFilters: async (filters) => {
    const next = {
      ...get().jiraCoverageFilters,
      ...filters,
    }
    persistJiraCoverageFilters(next)
    set({ jiraCoverageFilters: next })
    await get().loadTicketCoverage({
      hours: next.hours,
      repo_full_name: next.repo_full_name || undefined,
      branch: next.branch || undefined,
    })
  },

  correlateJiraTickets: async (params) => {
    const { serverConfig } = get()
    if (!serverConfig) return null

    try {
      const response = await tauriInvoke<JiraCorrelateResponse>('cmd_server_correlate_jira_tickets', {
        config: serverConfig,
        request: {
          hours: params?.hours ?? 72,
          limit: params?.limit ?? 500,
          repo_full_name: params?.repo_full_name,
          org_name: params?.org_name,
        },
      })
      await get().loadTicketCoverage({
        hours: params?.hours ?? 72,
        repo_full_name: params?.repo_full_name,
        branch: undefined,
        org_name: params?.org_name,
      })
      return response
    } catch (e) {
      set({ error: parseCommandError(String(e)).message })
      return null
    }
  },

  loadJiraTicketDetail: async (ticketId) => {
    const { serverConfig, jiraTicketDetails, jiraTicketDetailFetchedAt } = get()
    if (!serverConfig) return null
    const normalized = ticketId.trim().toUpperCase()
    if (!normalized) return null
    const fetchedAt = jiraTicketDetailFetchedAt[normalized] ?? 0
    const isFresh = Date.now() - fetchedAt < JIRA_TICKET_DETAIL_TTL_MS
    if (isFresh && Object.prototype.hasOwnProperty.call(jiraTicketDetails, normalized)) {
      return jiraTicketDetails[normalized] ?? null
    }
    set((state) => ({
      jiraTicketDetailLoading: {
        ...state.jiraTicketDetailLoading,
        [normalized]: true,
      },
    }))
    try {
      const resp = await tauriInvoke<JiraTicketDetailResponse>('cmd_server_get_jira_ticket_detail', {
        config: serverConfig,
        ticketId: normalized,
      })
      const ticket = resp.found ? resp.ticket ?? null : null
      set((state) => {
        const nextDetails = { ...state.jiraTicketDetails, [normalized]: ticket }
        const nextFetchedAt = { ...state.jiraTicketDetailFetchedAt, [normalized]: Date.now() }
        const nextLoading = { ...state.jiraTicketDetailLoading, [normalized]: false }

        // Evict oldest entries when cache exceeds limit
        const keys = Object.keys(nextFetchedAt)
        if (keys.length > JIRA_TICKET_CACHE_MAX) {
          const sorted = keys.sort((a, b) => (nextFetchedAt[a] ?? 0) - (nextFetchedAt[b] ?? 0))
          const toRemove = sorted.slice(0, keys.length - JIRA_TICKET_CACHE_MAX)
          for (const k of toRemove) {
            delete nextDetails[k]
            delete nextFetchedAt[k]
            delete nextLoading[k]
          }
        }

        return {
          jiraTicketDetails: nextDetails,
          jiraTicketDetailFetchedAt: nextFetchedAt,
          jiraTicketDetailLoading: nextLoading,
        }
      })
      return ticket
    } catch {
      set((state) => ({
        jiraTicketDetails: {
          ...state.jiraTicketDetails,
          [normalized]: null,
        },
        jiraTicketDetailFetchedAt: {
          ...state.jiraTicketDetailFetchedAt,
          [normalized]: Date.now(),
        },
        jiraTicketDetailLoading: {
          ...state.jiraTicketDetailLoading,
          [normalized]: false,
        },
      }))
      return null
    }
  },

  loadTicketEvidencePacket: async (ticketId, params) => {
    const { serverConfig, jiraCoverageFilters } = get()
    if (!serverConfig) return null
    const normalized = ticketId.trim().toUpperCase()
    if (!normalized) {
      set({ error: 'Ingresa un ticket válido para generar el evidence packet.' })
      return null
    }

    set({ isEvidencePacketLoading: true, error: null, evidencePacketTicketId: normalized })
    try {
      const response = await tauriInvoke<EvidencePacketResponse>('cmd_server_get_ticket_evidence_packet', {
        config: serverConfig,
        ticketId: normalized,
        query: {
          hours: params?.hours ?? jiraCoverageFilters.hours,
          repo_full_name: params?.repo_full_name ?? (jiraCoverageFilters.repo_full_name.trim() || undefined),
          branch: params?.branch ?? (jiraCoverageFilters.branch.trim() || undefined),
          org_name: params?.org_name,
        },
      })
      const packet = response.found ? response.packet ?? null : null
      set({ evidencePacket: packet, isEvidencePacketLoading: false })
      if (!packet) {
        set({ error: `No hay evidencia para ${normalized} en la ventana seleccionada.` })
      }
      return packet
    } catch (e) {
      set({
        error: parseCommandError(String(e)).message,
        evidencePacket: null,
        isEvidencePacketLoading: false,
      })
      return null
    }
  },
  }
}
