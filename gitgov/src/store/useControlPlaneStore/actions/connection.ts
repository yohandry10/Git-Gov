import { parseCommandError, tauriInvoke } from '@/lib/tauri'
import { formatControlPlaneConnectionError, validateControlPlaneUrl } from '@/lib/controlPlaneConfig'
import type { ControlPlaneActions, ServerConfig } from '../types'
import type { ControlPlaneGet, ControlPlaneSet } from '../store-types'
import { controlPlaneStoreRuntime } from '../runtime'
import {
  cachedSecureControlPlaneApiKey,
  persistSecureControlPlaneApiKey,
  persistServerConfig,
  readSecureControlPlaneApiKey,
  resolveServerConfig,
  syncOutboxServerConfig,
} from '../helpers'

type ConnectionActionKeys =
  | 'initFromEnv'
  | 'setServerConfig'
  | 'applyEnvApiKey'
  | 'applyApiKey'
  | 'markControlPlaneSessionValidated'
  | 'confirmControlPlaneSession'
  | 'resetControlPlaneAuthGate'
  | 'checkConnection'

export function createConnectionActions(
  set: ControlPlaneSet,
  get: ControlPlaneGet,
): Pick<ControlPlaneActions, ConnectionActionKeys> {
  return {
  initFromEnv: async () => {
    const secureApiKey = await readSecureControlPlaneApiKey()
    // Auto-connect with secure keyring storage, env vars, or compatibility fallback.
    const config = resolveServerConfig(undefined, undefined, secureApiKey)
    persistServerConfig(config)
    try {
      await persistSecureControlPlaneApiKey(config.api_key)
    } catch (e) {
      set({ error: parseCommandError(String(e)).message })
    }
    set({ serverConfig: config })
    await syncOutboxServerConfig(config)
    await get().checkConnection()
  },

  setServerConfig: (config) => {
    const merged = resolveServerConfig(config, get().serverConfig, cachedSecureControlPlaneApiKey)
    const urlError = validateControlPlaneUrl(merged.url)
    if (urlError) {
      set({
        error: urlError,
        isConnected: false,
        connectionStatus: 'disconnected',
        userRole: null,
        userClientId: null,
        userOrgId: null,
      })
      return
    }
    persistServerConfig(merged)
    set({ serverConfig: merged, error: null })
    void (async () => {
      try {
        await persistSecureControlPlaneApiKey(merged.api_key)
        await syncOutboxServerConfig(merged)
        await get().checkConnection()
      } catch (e) {
        set({
          isConnected: false,
          connectionStatus: 'disconnected',
          userRole: null,
          userClientId: null,
          userOrgId: null,
          error: parseCommandError(String(e)).message,
        })
      }
    })()
  },

  applyEnvApiKey: async () => {
    const { serverConfig } = get()
    const envApiKey = (import.meta.env.VITE_API_KEY || '').trim()
    if (!envApiKey) {
      set({ error: 'No existe VITE_API_KEY en el entorno actual.' })
      return false
    }

    const next = resolveServerConfig(
      {
        api_key: envApiKey,
      },
      serverConfig,
      cachedSecureControlPlaneApiKey,
    )
    persistServerConfig(next)
    try {
      await persistSecureControlPlaneApiKey(next.api_key)
    } catch (e) {
      set({ error: parseCommandError(String(e)).message })
      return false
    }
    set({ serverConfig: next, error: null })
    await syncOutboxServerConfig(next)
    await get().checkConnection()
    const state = get()
    return state.isConnected && state.userRole === 'Admin'
  },

  applyApiKey: async (apiKey, url) => {
    const { serverConfig } = get()
    const normalizedKey = apiKey.trim()
    if (!normalizedKey) {
      set({ error: 'Ingresa una API key válida.' })
      return false
    }
    const next = resolveServerConfig(
      {
        url: url?.trim() || undefined,
        api_key: normalizedKey,
      },
      serverConfig,
      cachedSecureControlPlaneApiKey,
    )
    const urlError = validateControlPlaneUrl(next.url)
    if (urlError) {
      set({ error: urlError })
      return false
    }
    persistServerConfig(next)
    try {
      await persistSecureControlPlaneApiKey(next.api_key)
    } catch (e) {
      set({ error: parseCommandError(String(e)).message })
      return false
    }
    set({ serverConfig: next, error: null })
    await syncOutboxServerConfig(next)
    await get().checkConnection()
    const state = get()
    return state.isConnected && Boolean(state.userRole)
  },

  markControlPlaneSessionValidated: (session) => {
    set({
      pendingControlPlaneSession: session,
      controlPlaneAuthConfirmed: false,
    })
  },

  confirmControlPlaneSession: () => {
    set({
      controlPlaneAuthConfirmed: true,
      pendingControlPlaneSession: null,
      error: null,
    })
  },

  resetControlPlaneAuthGate: () => {
    set({
      controlPlaneAuthConfirmed: true,
      pendingControlPlaneSession: null,
    })
  },

  checkConnection: async (options) => {
    if (controlPlaneStoreRuntime.checkConnectionInFlight) {
      await controlPlaneStoreRuntime.checkConnectionInFlight
      return
    }

    const run = (async () => {
      const { serverConfig, isConnected: wasConnected } = get()
      if (!serverConfig) return
      const isBackground = Boolean(options?.background)

      if (!isBackground) {
        set({ isLoading: true, error: null, connectionStatus: 'checking' })
      }
      try {
        const healthy = await tauriInvoke<boolean>('cmd_server_health', { config: serverConfig })
        if (healthy) {
          let hasRoleContext = await get().loadMe()

          if (!hasRoleContext) {
            const envApiKey = (import.meta.env.VITE_API_KEY || '').trim()
            const currentApiKey = serverConfig.api_key?.trim() || ''
            if (envApiKey && envApiKey !== currentApiKey) {
              const recoveredConfig: ServerConfig = { ...serverConfig, api_key: envApiKey }
              persistServerConfig(recoveredConfig)
              await persistSecureControlPlaneApiKey(recoveredConfig.api_key)
              await syncOutboxServerConfig(recoveredConfig)
              set({ serverConfig: recoveredConfig })
              hasRoleContext = await get().loadMe()
            }
          }

          if (hasRoleContext) {
            set({
              isConnected: true,
              isLoading: false,
              connectionStatus: 'connected',
              maintenanceDetectedAt: null,
              error: isBackground ? get().error : null,
            })
          } else {
            set({
              isConnected: false,
              isLoading: false,
              connectionStatus: 'disconnected',
              maintenanceDetectedAt: null,
              userRole: null,
              userClientId: null,
              userOrgId: null,
              controlPlaneAuthConfirmed: true,
              pendingControlPlaneSession: null,
              error: get().error ?? (isBackground ? null : 'No se pudo autenticar con el Control Plane. Verifica la API key.'),
            })
          }
        } else {
          // Health endpoint returned false — treat as maintenance if was previously connected
          if (wasConnected) {
            set((s) => ({
              isConnected: false,
              isLoading: false,
              connectionStatus: 'maintenance',
              maintenanceDetectedAt: s.maintenanceDetectedAt ?? Date.now(),
            }))
          } else {
            set({ isConnected: false, isLoading: false, connectionStatus: 'disconnected' })
          }
        }
      } catch (e) {
        const errMsg = formatControlPlaneConnectionError(
          parseCommandError(String(e)).message,
          serverConfig.url,
        )
        // If previously connected and now failing → server is likely restarting (maintenance)
        if (wasConnected) {
          set((s) => ({
            error: errMsg,
            isLoading: false,
            isConnected: false,
            connectionStatus: 'maintenance',
            maintenanceDetectedAt: s.maintenanceDetectedAt ?? Date.now(),
          }))
        } else {
          set({ error: errMsg, isLoading: false, isConnected: false, connectionStatus: 'disconnected' })
        }
      }
    })()

    controlPlaneStoreRuntime.checkConnectionInFlight = run
    try {
      await run
    } finally {
      if (controlPlaneStoreRuntime.checkConnectionInFlight === run) controlPlaneStoreRuntime.checkConnectionInFlight = null
    }
  },
  }
}
