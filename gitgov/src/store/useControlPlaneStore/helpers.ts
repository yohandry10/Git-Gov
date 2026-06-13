import { tauriInvoke, parseCommandError } from '@/lib/tauri'
import { useAuthStore } from '@/store/useAuthStore'
import { getEnvControlPlaneUrl, normalizeControlPlaneUrl, resolveControlPlaneUrl } from '@/lib/controlPlaneConfig'
import type { CombinedEvent } from '@/lib/types'
import type { ActiveDev7dEntry, ChatMessage, ChatSession, JiraCoverageFilters, ServerConfig } from './types'
import {
  ALLOW_LEGACY_DEFAULT_API_KEY,
  CONTROL_PLANE_ACTIVE_ORG_STORAGE_KEY_PREFIX,
  CHAT_MESSAGES_STORAGE_KEY_PREFIX,
  CONTROL_PLANE_CONFIG_STORAGE_KEY,
  DEFAULT_CHAT_SESSION_TITLE,
  DEV_ACTIVITY_WINDOW_MS,
  JIRA_COVERAGE_FILTERS_STORAGE_KEY,
  LEGACY_CHAT_MESSAGES_STORAGE_KEY,
  LEGACY_DEFAULT_API_KEY,
  LOGS_KEYSET_MAX_PAGES,
  LOGS_KEYSET_PAGE_SIZE,
  MAX_CHAT_MESSAGES_PER_SESSION,
  MAX_CHAT_SESSIONS,
} from './constants'

export let cachedSecureControlPlaneApiKey: string | undefined

interface StoredChatStateV2 {
  version: 2
  active_session_id: string
  sessions: ChatSession[]
}

export function isLikelySyntheticLogin(login: string): boolean {
  return /^(alias_|erase_ok_|hb_user_|user_[0-9a-f]{6,}|test_?user|golden_?test|smoke|manual-check|victim_)/i.test(login)
}

export function buildActiveDevs7dFromLogs(logs: CombinedEvent[], now: number): ActiveDev7dEntry[] {
  const start = now - DEV_ACTIVITY_WINDOW_MS
  const grouped = new Map<string, {
    events: number
    last_seen: number
    sample_repo_empty_count: number
  }>()

  for (const log of logs) {
    if (log.created_at < start || log.created_at > now) continue
    const login = (log.user_login ?? '').trim()
    if (!login) continue
    const prev = grouped.get(login) ?? { events: 0, last_seen: 0, sample_repo_empty_count: 0 }
    prev.events += 1
    if (log.created_at > prev.last_seen) prev.last_seen = log.created_at
    if (!log.repo_name && !log.branch) prev.sample_repo_empty_count += 1
    grouped.set(login, prev)
  }

  return Array.from(grouped.entries())
    .map(([user_login, agg]) => {
      const allEmptyRepoBranch = agg.sample_repo_empty_count === agg.events
      return {
        user_login,
        events: agg.events,
        last_seen: agg.last_seen,
        suspicious_test_data: isLikelySyntheticLogin(user_login) || allEmptyRepoBranch,
        sample_repo_empty_count: agg.sample_repo_empty_count,
      }
    })
    .sort((a, b) => b.events - a.events || b.last_seen - a.last_seen)
}

export function compareCombinedEventDesc(a: CombinedEvent, b: CombinedEvent): number {
  if (a.created_at !== b.created_at) return b.created_at - a.created_at
  return b.id.localeCompare(a.id)
}

export function mergeRecentLogs(existing: CombinedEvent[], incoming: CombinedEvent[], limit: number): CombinedEvent[] {
  if (incoming.length === 0) {
    // Return same reference to avoid invalidating downstream useMemo hooks
    return existing.length <= limit ? existing : existing.slice(0, limit)
  }
  const merged = [...incoming, ...existing]
  merged.sort(compareCombinedEventDesc)

  const deduped: CombinedEvent[] = []
  const seen = new Set<string>()
  for (const item of merged) {
    if (seen.has(item.id)) continue
    seen.add(item.id)
    deduped.push(item)
    if (deduped.length >= limit) break
  }
  return deduped
}

interface LogsKeysetCursor {
  before_created_at: number
  before_id: string
}

export function getOldestLogsCursor(events: CombinedEvent[]): LogsKeysetCursor | null {
  if (events.length === 0) return null
  const tail = events[events.length - 1]
  if (!tail?.id || !Number.isFinite(tail.created_at)) return null
  return {
    before_created_at: tail.created_at,
    before_id: tail.id,
  }
}

export function sanitizeLogsWindow(limit: number, offset: number): { safeLimit: number; safeOffset: number } {
  const safeLimit = Number.isFinite(limit) ? Math.max(1, Math.min(500, Math.floor(limit))) : 500
  const safeOffset = Number.isFinite(offset) ? Math.max(0, Math.floor(offset)) : 0
  return { safeLimit, safeOffset }
}

export async function fetchLogsByFilter(
  serverConfig: ServerConfig,
  filter: Record<string, unknown>,
): Promise<CombinedEvent[]> {
  return tauriInvoke<CombinedEvent[]>('cmd_server_get_logs', {
    config: serverConfig,
    filter,
  })
}

export async function fetchLogsKeysetWindow(
  serverConfig: ServerConfig,
  limit: number,
  offset: number,
): Promise<CombinedEvent[]> {
  const { safeLimit, safeOffset } = sanitizeLogsWindow(limit, offset)
  if (safeOffset === 0) {
    return fetchLogsByFilter(serverConfig, { limit: safeLimit, offset: 0 })
  }

  let remainingOffset = safeOffset
  const collected: CombinedEvent[] = []
  let cursor: LogsKeysetCursor | null = null
  let pageCount = 0

  while (pageCount < LOGS_KEYSET_MAX_PAGES && collected.length < safeLimit) {
    pageCount += 1
    const requested = Math.min(LOGS_KEYSET_PAGE_SIZE, safeLimit + remainingOffset)
    const filter: Record<string, unknown> = {
      limit: requested,
      offset: 0,
    }
    if (cursor) {
      filter.before_created_at = cursor.before_created_at
      filter.before_id = cursor.before_id
    }

    const page = await fetchLogsByFilter(serverConfig, filter)
    if (page.length === 0) break

    let consumeFrom = 0
    if (remainingOffset > 0) {
      const skipped = Math.min(remainingOffset, page.length)
      remainingOffset -= skipped
      consumeFrom = skipped
    }
    if (consumeFrom < page.length) {
      collected.push(...page.slice(consumeFrom))
    }

    cursor = getOldestLogsCursor(page)
    if (!cursor || page.length < requested) break
  }

  // Compatibility fallback for very deep legacy offsets.
  if (remainingOffset > 0) {
    return fetchLogsByFilter(serverConfig, { limit: safeLimit, offset: safeOffset })
  }

  return collected.slice(0, safeLimit)
}

export function readStoredServerConfig(): ServerConfig | null {
  try {
    const raw = window.localStorage.getItem(CONTROL_PLANE_CONFIG_STORAGE_KEY)
    if (!raw) return null
    const parsed = JSON.parse(raw) as Partial<ServerConfig>
    if (!parsed || typeof parsed.url !== 'string') return null
    return {
      url: parsed.url,
      // Legacy v1: api_key could still exist in localStorage and must be migrated to keyring.
      api_key: typeof parsed.api_key === 'string' && parsed.api_key.trim() ? parsed.api_key : undefined,
    }
  } catch {
    return null
  }
}

export function persistServerConfig(config: ServerConfig | null) {
  try {
    if (!config) {
      window.localStorage.removeItem(CONTROL_PLANE_CONFIG_STORAGE_KEY)
      return
    }
    // Persist only non-secret fields in localStorage.
    window.localStorage.setItem(CONTROL_PLANE_CONFIG_STORAGE_KEY, JSON.stringify({
      url: config.url,
    }))
  } catch {
    // ignore storage errors
  }
}

function stableConfigFingerprint(value: string | null | undefined): string {
  const input = (value ?? '').trim()
  let hash = 2166136261
  for (let i = 0; i < input.length; i += 1) {
    hash ^= input.charCodeAt(i)
    hash = Math.imul(hash, 16777619)
  }
  return (hash >>> 0).toString(16).padStart(8, '0')
}

export function buildActiveOrgStorageKey(config: ServerConfig | null): string | null {
  if (!config) return null
  const login = (useAuthStore.getState().user?.login ?? '').trim().toLowerCase()
  if (!login) return null
  const urlPart = stableConfigFingerprint(config.url)
  const keyPart = stableConfigFingerprint(config.api_key)
  return `${CONTROL_PLANE_ACTIVE_ORG_STORAGE_KEY_PREFIX}${encodeURIComponent(login)}.${urlPart}.${keyPart}`
}

export function readStoredSelectedOrgName(config: ServerConfig | null): string {
  try {
    const key = buildActiveOrgStorageKey(config)
    if (!key) return ''
    const raw = window.localStorage.getItem(key)
    return typeof raw === 'string' ? raw.trim() : ''
  } catch {
    return ''
  }
}

export function persistSelectedOrgName(config: ServerConfig | null, orgName: string) {
  try {
    const key = buildActiveOrgStorageKey(config)
    if (!key) return
    const normalized = orgName.trim()
    if (!normalized) {
      window.localStorage.removeItem(key)
      return
    }
    window.localStorage.setItem(key, normalized)
  } catch {
    // ignore storage errors
  }
}

export function normalizeSecretValue(input: string | null | undefined): string | undefined {
  const normalized = (input ?? '').trim()
  return normalized || undefined
}

export async function readSecureControlPlaneApiKey(): Promise<string | undefined> {
  try {
    const value = await tauriInvoke<string | null>('cmd_cp_get_api_key')
    const normalized = normalizeSecretValue(value)
    cachedSecureControlPlaneApiKey = normalized
    return normalized
  } catch {
    return cachedSecureControlPlaneApiKey
  }
}

export async function persistSecureControlPlaneApiKey(apiKey?: string): Promise<void> {
  const normalized = normalizeSecretValue(apiKey)
  try {
    if (!normalized) {
      await tauriInvoke('cmd_cp_clear_api_key')
      cachedSecureControlPlaneApiKey = undefined
      return
    }
    await tauriInvoke('cmd_cp_set_api_key', { apiKey: normalized })
    cachedSecureControlPlaneApiKey = normalized
  } catch (e) {
    throw new Error(parseCommandError(String(e)).message || 'No se pudo guardar la API key en el almacenamiento seguro.')
  }
}

export function readStoredJiraCoverageFilters(): JiraCoverageFilters {
  try {
    const raw = window.localStorage.getItem(JIRA_COVERAGE_FILTERS_STORAGE_KEY)
    if (!raw) return { hours: 72, repo_full_name: '', branch: '' }
    const parsed = JSON.parse(raw) as Partial<JiraCoverageFilters>
    return {
      hours: typeof parsed.hours === 'number' && Number.isFinite(parsed.hours) ? parsed.hours : 72,
      repo_full_name: typeof parsed.repo_full_name === 'string' ? parsed.repo_full_name : '',
      branch: typeof parsed.branch === 'string' ? parsed.branch : '',
    }
  } catch {
    return { hours: 72, repo_full_name: '', branch: '' }
  }
}

export function persistJiraCoverageFilters(filters: JiraCoverageFilters) {
  try {
    window.localStorage.setItem(JIRA_COVERAGE_FILTERS_STORAGE_KEY, JSON.stringify(filters))
  } catch {
    // ignore
  }
}

export function sanitizeChatMessages(raw: unknown): ChatMessage[] {
  if (!Array.isArray(raw)) return []
  return raw
    .filter((item): item is ChatMessage => {
      if (!item || typeof item !== 'object') return false
      const candidate = item as Partial<ChatMessage>
      return (
        typeof candidate.id === 'string' &&
        (candidate.role === 'user' || candidate.role === 'assistant') &&
        typeof candidate.content === 'string' &&
        typeof candidate.timestamp === 'number'
      )
    })
    .slice(-MAX_CHAT_MESSAGES_PER_SESSION)
}

export function parseStoredChatMessages(raw: string | null): ChatMessage[] {
  if (!raw) return []
  try {
    return sanitizeChatMessages(JSON.parse(raw))
  } catch {
    return []
  }
}

export function normalizeChatTitle(input: string): string {
  const compact = input.replace(/\s+/g, ' ').trim()
  if (!compact) return DEFAULT_CHAT_SESSION_TITLE
  if (compact.length <= 36) return compact
  return `${compact.slice(0, 36)}...`
}

export function deriveSessionTitleFromQuestion(question: string): string {
  return normalizeChatTitle(question)
}

export function buildChatSession(messages: ChatMessage[] = [], title?: string): ChatSession {
  const now = Date.now()
  return {
    id: crypto.randomUUID(),
    title: title?.trim() ? normalizeChatTitle(title) : DEFAULT_CHAT_SESSION_TITLE,
    created_at: now,
    updated_at: now,
    messages: messages.slice(-MAX_CHAT_MESSAGES_PER_SESSION),
  }
}

export function sanitizeChatSession(input: unknown, fallbackIndex: number): ChatSession | null {
  if (!input || typeof input !== 'object') return null
  const candidate = input as Partial<ChatSession>
  if (typeof candidate.id !== 'string') return null
  const messages = sanitizeChatMessages(candidate.messages)
  const createdAt = typeof candidate.created_at === 'number' && Number.isFinite(candidate.created_at)
    ? candidate.created_at
    : Date.now()
  const updatedAt = typeof candidate.updated_at === 'number' && Number.isFinite(candidate.updated_at)
    ? candidate.updated_at
    : createdAt
  const inferredTitle =
    typeof candidate.title === 'string' && candidate.title.trim()
      ? candidate.title
      : (messages.find((m) => m.role === 'user')?.content ?? `${DEFAULT_CHAT_SESSION_TITLE} ${fallbackIndex + 1}`)
  return {
    id: candidate.id,
    title: normalizeChatTitle(inferredTitle),
    created_at: createdAt,
    updated_at: updatedAt,
    messages,
  }
}

export function normalizeChatSessions(input: unknown): ChatSession[] {
  if (!Array.isArray(input)) return []
  const sessions: ChatSession[] = []
  for (let i = 0; i < input.length; i += 1) {
    const normalized = sanitizeChatSession(input[i], i)
    if (normalized) sessions.push(normalized)
  }
  sessions.sort((a, b) => a.created_at - b.created_at)
  return sessions.slice(-MAX_CHAT_SESSIONS)
}

export function readStoredChatStateFromRaw(raw: string | null): { sessions: ChatSession[]; activeSessionId: string | null } {
  if (!raw) return { sessions: [], activeSessionId: null }
  try {
    const parsed = JSON.parse(raw) as StoredChatStateV2 | ChatMessage[]
    if (Array.isArray(parsed)) {
      const legacyMessages = sanitizeChatMessages(parsed)
      if (!legacyMessages.length) return { sessions: [], activeSessionId: null }
      const single = buildChatSession(legacyMessages, legacyMessages.find((m) => m.role === 'user')?.content)
      return { sessions: [single], activeSessionId: single.id }
    }
    if (!parsed || typeof parsed !== 'object') return { sessions: [], activeSessionId: null }
    const sessions = normalizeChatSessions((parsed as StoredChatStateV2).sessions)
    if (!sessions.length) return { sessions: [], activeSessionId: null }
    const requested = (parsed as StoredChatStateV2).active_session_id
    const activeSessionId = sessions.some((s) => s.id === requested) ? requested : sessions[sessions.length - 1].id
    return { sessions, activeSessionId }
  } catch {
    return { sessions: [], activeSessionId: null }
  }
}

export function deriveActiveChatMessages(sessions: ChatSession[], activeSessionId: string | null): ChatMessage[] {
  if (!activeSessionId) return []
  return sessions.find((session) => session.id === activeSessionId)?.messages ?? []
}

export function ensureAtLeastOneSession(sessions: ChatSession[], activeSessionId: string | null): { sessions: ChatSession[]; activeSessionId: string } {
  if (sessions.length > 0 && activeSessionId && sessions.some((s) => s.id === activeSessionId)) {
    return { sessions, activeSessionId }
  }
  if (sessions.length > 0) {
    return { sessions, activeSessionId: sessions[sessions.length - 1].id }
  }
  const session = buildChatSession()
  return { sessions: [session], activeSessionId: session.id }
}

export function getActiveChatStorageKey(): string {
  const login = (useAuthStore.getState().user?.login ?? '').trim().toLowerCase()
  const encodedLogin = login ? encodeURIComponent(login) : 'anonymous'
  return `${CHAT_MESSAGES_STORAGE_KEY_PREFIX}${encodedLogin}`
}

export function hasScopedChatStorageEntries(): boolean {
  try {
    for (let i = 0; i < window.localStorage.length; i += 1) {
      const key = window.localStorage.key(i)
      if (key?.startsWith(CHAT_MESSAGES_STORAGE_KEY_PREFIX)) return true
    }
  } catch {
    // ignore storage errors
  }
  return false
}

export function readStoredChatState(): { sessions: ChatSession[]; activeSessionId: string } {
  try {
    const userScopedKey = getActiveChatStorageKey()
    const userScopedRaw = window.localStorage.getItem(userScopedKey)
    if (userScopedRaw !== null) {
      const current = readStoredChatStateFromRaw(userScopedRaw)
      return ensureAtLeastOneSession(current.sessions, current.activeSessionId)
    }
    const legacyRaw = window.localStorage.getItem(LEGACY_CHAT_MESSAGES_STORAGE_KEY)
    if (!legacyRaw) return ensureAtLeastOneSession([], null)

    // Migrate legacy global history only when no scoped histories exist yet.
    // This prevents old mixed history from leaking to additional users.
    if (hasScopedChatStorageEntries()) return ensureAtLeastOneSession([], null)

    const legacyMessages = parseStoredChatMessages(legacyRaw)
    const migrated = ensureAtLeastOneSession(
      legacyMessages.length
        ? [buildChatSession(legacyMessages, legacyMessages.find((m) => m.role === 'user')?.content)]
        : [],
      null,
    )
    try {
      window.localStorage.setItem(userScopedKey, JSON.stringify({
        version: 2,
        active_session_id: migrated.activeSessionId,
        sessions: migrated.sessions,
      } satisfies StoredChatStateV2))
      window.localStorage.removeItem(LEGACY_CHAT_MESSAGES_STORAGE_KEY)
    } catch {
      // ignore migration persistence errors
    }
    return migrated
  } catch {
    return ensureAtLeastOneSession([], null)
  }
}

let chatPersistTimeoutId: number | null = null
let chatPersistIdleId: number | null = null
export function clearPendingChatPersistJob() {
  if (chatPersistTimeoutId !== null) {
    window.clearTimeout(chatPersistTimeoutId)
    chatPersistTimeoutId = null
  }
  if (chatPersistIdleId !== null && typeof window.cancelIdleCallback === 'function') {
    window.cancelIdleCallback(chatPersistIdleId)
    chatPersistIdleId = null
  }
}

export function persistChatState(sessions: ChatSession[], activeSessionId: string) {
  try {
    const userScopedKey = getActiveChatStorageKey()
    clearPendingChatPersistJob()

    const writeToStorage = () => {
      const compactSessions = sessions.slice(-MAX_CHAT_SESSIONS).map((session) => {
        const compactMessages = session.messages.slice(-MAX_CHAT_MESSAGES_PER_SESSION).map((msg) => {
          const trimmedContent = msg.content.length > 4000 ? `${msg.content.slice(0, 4000)}\n...[recortado para rendimiento]` : msg.content
          if (!msg.response) {
            return { ...msg, content: trimmedContent }
          }
          const trimmedAnswer =
            msg.response.answer.length > 4000
              ? `${msg.response.answer.slice(0, 4000)}\n...[recortado para rendimiento]`
              : msg.response.answer
          return {
            ...msg,
            content: trimmedContent,
            response: {
              ...msg.response,
              answer: trimmedAnswer,
              data_refs: msg.response.data_refs.slice(0, 12),
            },
          }
        })
        const fallbackTitle = compactMessages.find((m) => m.role === 'user')?.content ?? session.title
        return {
          ...session,
          title: normalizeChatTitle(session.title || fallbackTitle),
          messages: compactMessages,
        }
      })
      const payload: StoredChatStateV2 = {
        version: 2,
        active_session_id: activeSessionId,
        sessions: compactSessions,
      }
      try {
        window.localStorage.setItem(userScopedKey, JSON.stringify(payload))
      } catch {
        // ignore
      }
    }

    const schedulePersist = () => {
      chatPersistTimeoutId = null
      writeToStorage()
    }

    // Defer heavy serialization to idle/debounced time to avoid UI hitch while typing.
    if (typeof window.requestIdleCallback === 'function') {
      chatPersistIdleId = window.requestIdleCallback(() => {
        chatPersistIdleId = null
        schedulePersist()
      }, { timeout: 500 })
      return
    }
    chatPersistTimeoutId = window.setTimeout(schedulePersist, 120)
  } catch {
    // ignore
  }
}

export function parseRetryAfterSeconds(message: string): number | null {
  const quoted = message.match(/"retry_after_seconds"\s*:\s*(\d+)/i)
  if (quoted) return Number.parseInt(quoted[1], 10)
  const plain = message.match(/retry[_ -]?after[_ -]?seconds?\s*[:=]?\s*(\d+)/i)
  if (plain) return Number.parseInt(plain[1], 10)
  return null
}

export function formatChatErrorMessage(rawMessage: string): string {
  const isRateLimited =
    /429/.test(rawMessage) ||
    /RATE_LIMITED/i.test(rawMessage) ||
    /Too many requests/i.test(rawMessage)

  if (!isRateLimited) return rawMessage

  const retryAfter = parseRetryAfterSeconds(rawMessage)
  if (retryAfter && Number.isFinite(retryAfter) && retryAfter > 0) {
    return `El chat está recibiendo demasiadas solicitudes ahora. Reintenta en ${retryAfter} segundos.`
  }
  return 'El chat está recibiendo demasiadas solicitudes ahora. Reintenta en unos segundos.'
}

export function resolveServerConfig(
  input?: Partial<ServerConfig> | null,
  previous?: ServerConfig | null,
  secureApiKey?: string | null,
): ServerConfig {
  const stored = readStoredServerConfig()
  const envUrl = getEnvControlPlaneUrl()
  const envApiKey = (import.meta.env.VITE_API_KEY || '').trim()
  const url = resolveControlPlaneUrl({
    inputUrl: input?.url,
    previousUrl: previous?.url,
    storedUrl: stored?.url,
    envUrl,
  })

  const apiKey =
    input?.api_key?.trim() ||
    previous?.api_key?.trim() ||
    envApiKey ||
    secureApiKey?.trim() ||
    cachedSecureControlPlaneApiKey?.trim() ||
    stored?.api_key?.trim() ||
    (ALLOW_LEGACY_DEFAULT_API_KEY ? LEGACY_DEFAULT_API_KEY : '')

  return {
    url: normalizeControlPlaneUrl(url),
    api_key: apiKey || undefined,
  }
}

export function isUnauthorizedError(message: string): boolean {
  const normalized = message.toLowerCase()
  return normalized.includes('401') || normalized.includes('unauthorized') || normalized.includes('invalid or expired api key')
}

export function isControlPlaneIdentityCompatible(
  clientId: string,
  githubLogin: string | null,
  role: string,
  principalType?: string | null,
): boolean {
  const cp = clientId.trim().toLowerCase()
  const normalizedRole = role.trim().toLowerCase()
  const normalizedPrincipalType = principalType?.trim().toLowerCase() || ''
  if (!cp) return false

  // Platform founder is a GitGov control-plane principal, not a tenant GitHub identity.
  if (normalizedPrincipalType === 'platform_founder' || cp === 'bootstrap-admin') {
    return true
  }

  if (!githubLogin) return true

  const gh = githubLogin.trim().toLowerCase()
  if (!gh) return false

  // Developers must always match GitHub login.
  if (normalizedRole === 'developer') {
    return cp === gh
  }

  // Admin/Architect/PM keys may target service users or scoped org admins.
  return true
}

export async function syncOutboxServerConfig(config: ServerConfig | null): Promise<void> {
  try {
    await tauriInvoke('cmd_server_sync_outbox', { config })
  } catch {
    // Non-fatal: dashboard connectivity should still work even if outbox sync fails.
  }
}
