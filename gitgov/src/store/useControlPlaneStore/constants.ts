export const CONTROL_PLANE_CONFIG_STORAGE_KEY = 'gitgov.control_plane_config'
export const CONTROL_PLANE_ACTIVE_ORG_STORAGE_KEY_PREFIX = 'gitgov.control_plane.active_org.'
export const JIRA_COVERAGE_FILTERS_STORAGE_KEY = 'gitgov.jira_coverage_filters'
export const LEGACY_CHAT_MESSAGES_STORAGE_KEY = 'gitgov.chat_messages'
export const CHAT_MESSAGES_STORAGE_KEY_PREFIX = 'gitgov.chat_messages.v2.'
export const JIRA_TICKET_DETAIL_TTL_MS = 2 * 60 * 1000
export const IS_DEV_MODE = Boolean(import.meta.env.DEV)

// Compatibility fallback: can be provided explicitly via env when needed.
export const LEGACY_DEFAULT_API_KEY = (import.meta.env.VITE_LEGACY_DEFAULT_API_KEY || '').trim()
export const ALLOW_LEGACY_DEFAULT_API_KEY = (() => {
  const raw = (import.meta.env.VITE_ALLOW_LEGACY_DEFAULT_API_KEY || '').trim().toLowerCase()
  if (raw === '1' || raw === 'true' || raw === 'yes' || raw === 'on') return true
  if (raw === '0' || raw === 'false' || raw === 'no' || raw === 'off') return false
  return IS_DEV_MODE
})()
export const DEV_ACTIVITY_WINDOW_MS = 7 * 24 * 60 * 60 * 1000
export const HEAVY_DASHBOARD_REFRESH_MS = 5 * 60 * 1000
export const DEFAULT_GOVERNANCE_LOG_WINDOW = 120
export const SSE_GOVERNANCE_LOG_WINDOW = 120
export const SSE_REFRESH_DEBOUNCE_MS = 1000
export const JIRA_TICKET_CACHE_MAX = 50
export const LOGS_KEYSET_PAGE_SIZE = 500
export const LOGS_KEYSET_MAX_PAGES = 64
export const MAX_CHAT_SESSIONS = 8
export const MAX_CHAT_MESSAGES_PER_SESSION = 80
export const DEFAULT_CHAT_SESSION_TITLE = 'Chat nuevo'
