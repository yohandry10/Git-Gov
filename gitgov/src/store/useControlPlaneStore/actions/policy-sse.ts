import { parseCommandError, tauriInvoke, tauriListen } from '@/lib/tauri'
import { notifyNewEvents } from '@/lib/notifications'
import { persistTimezone } from '@/lib/timezone'
import type { ControlPlaneActions, PolicyHistoryEntry, PolicyResponseData } from '../types'
import type { ControlPlaneGet, ControlPlaneSet } from '../store-types'
import { SSE_GOVERNANCE_LOG_WINDOW, SSE_REFRESH_DEBOUNCE_MS } from '../constants'
import { controlPlaneStoreRuntime } from '../runtime'

type PolicySseActionKeys =
  | 'setDisplayTimezone'
  | 'loadPolicy'
  | 'savePolicy'
  | 'loadPolicyHistory'
  | 'connectSse'
  | 'disconnectSse'

export function createPolicySseActions(
  set: ControlPlaneSet,
  get: ControlPlaneGet,
): Pick<ControlPlaneActions, PolicySseActionKeys> {
  return {
  setDisplayTimezone: (tz: string) => {
    persistTimezone(tz)
    set({ displayTimezone: tz })
  },

  loadPolicy: async (repoName: string) => {
    const serverConfig = get().serverConfig
    if (!serverConfig) return
    set({ isPolicyLoading: true, policyError: null })
    try {
      const config = { url: serverConfig.url, api_key: serverConfig.api_key }
      const result = await tauriInvoke<PolicyResponseData | null>('cmd_server_get_policy', {
        config,
        repoName,
      })
      set({ policyData: result ?? null, isPolicyLoading: false })
    } catch (e) {
      const msg = parseCommandError(String(e))
      set({ policyError: msg.message, isPolicyLoading: false })
    }
  },

  savePolicy: async (repoName: string, policyConfig: import('@/lib/types').GitGovConfig) => {
    const serverConfig = get().serverConfig
    if (!serverConfig) return false
    const source = get().policyData?.source
    if (source?.source_mode === 'repo-policy-as-code') {
      const policyPath = source.source_path ?? 'archivo de política del repo'
      set({
        policyError: `Esta política está gestionada desde ${policyPath}. Crea un cambio en el repo y pásalo por PR/review; no se permite override directo desde Governance.`,
        isPolicySaving: false,
      })
      return false
    }
    set({ isPolicySaving: true, policyError: null })
    try {
      const config = { url: serverConfig.url, api_key: serverConfig.api_key }
      const result = await tauriInvoke<PolicyResponseData>('cmd_server_override_policy', {
        config,
        repoName,
        policyConfig,
      })
      set({ policyData: result, isPolicySaving: false })
      return true
    } catch (e) {
      const msg = parseCommandError(String(e))
      set({ policyError: msg.message, isPolicySaving: false })
      return false
    }
  },

  loadPolicyHistory: async (repoName: string) => {
    const serverConfig = get().serverConfig
    if (!serverConfig) return
    try {
      const config = { url: serverConfig.url, api_key: serverConfig.api_key }
      const history = await tauriInvoke<PolicyHistoryEntry[]>('cmd_server_get_policy_history', {
        config,
        repoName,
      })
      set({ policyHistory: history })
    } catch {
      // non-fatal
    }
  },

  connectSse: async () => {
    const { serverConfig, sseConnected } = get()
    if (!serverConfig || sseConnected) return

    // Debounce: track whether an SSE refresh is already scheduled
    let sseRefreshScheduled = false

    // Listen for SSE events from Tauri backend
    const unlistenEvent = await tauriListen<{ type: string; count?: number }>('gitgov:sse-event', (payload) => {
      const eventType = payload?.type
      if ((eventType === 'new_events' || eventType === 'stats_updated') && !sseRefreshScheduled) {
        // Debounce: batch rapid SSE notifications into a single refresh
        sseRefreshScheduled = true
        setTimeout(() => {
          sseRefreshScheduled = false
          void get().loadLogsIncremental(SSE_GOVERNANCE_LOG_WINDOW)
          void get().loadStats()
        }, SSE_REFRESH_DEBOUNCE_MS)
        // Desktop notification for new events (fire-and-forget)
        if (eventType === 'new_events' && payload.count) {
          void notifyNewEvents(payload.count)
        }
      }
      // heartbeat — no action needed
    })

    const unlistenConnected = await tauriListen<unknown>('gitgov:sse-connected', () => {
      set({ sseConnected: true })
    })

    const unlistenDisconnected = await tauriListen<unknown>('gitgov:sse-disconnected', () => {
      set({ sseConnected: false })
      unlistenEvent()
      unlistenConnected()
      unlistenDisconnected()
      controlPlaneStoreRuntime.sseUnlisteners = []
      // Auto-reconnect after 5s if still connected to server
      controlPlaneStoreRuntime.sseReconnectTimer = setTimeout(() => {
        controlPlaneStoreRuntime.sseReconnectTimer = null
        if (get().isConnected) {
          void get().connectSse()
        }
      }, 5000)
    })

    // Store unlisten functions for cleanup on disconnect
    controlPlaneStoreRuntime.sseUnlisteners = [unlistenEvent, unlistenConnected, unlistenDisconnected]

    // Fire-and-forget: the command runs until connection drops or is cancelled
    const config = { url: serverConfig.url, api_key: serverConfig.api_key }
    tauriInvoke('cmd_server_sse_connect', { config }).catch(() => {
      // Connection failed — clean up listeners
      set({ sseConnected: false })
      for (const fn of controlPlaneStoreRuntime.sseUnlisteners) fn()
      controlPlaneStoreRuntime.sseUnlisteners = []
    })
  },

  disconnectSse: () => {
    set({ sseConnected: false })
    // Cancel pending reconnect timer
    if (controlPlaneStoreRuntime.sseReconnectTimer !== null) {
      clearTimeout(controlPlaneStoreRuntime.sseReconnectTimer)
      controlPlaneStoreRuntime.sseReconnectTimer = null
    }
    // Signal the Tauri backend to stop the stream loop (bumps generation)
    tauriInvoke('cmd_server_sse_disconnect', {}).catch(() => {})
    for (const fn of controlPlaneStoreRuntime.sseUnlisteners) fn()
    controlPlaneStoreRuntime.sseUnlisteners = []
  },
  }
}
