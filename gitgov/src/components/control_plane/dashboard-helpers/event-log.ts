import type { CombinedEvent } from '@/lib/types'

export function readDetailString(log: CombinedEvent, key: string): string | null {
  const value = log.details?.[key]
  if (typeof value === 'string' && value.trim().length > 0) return value
  const metadata = log.details && typeof log.details === 'object' ? (log.details['metadata'] as Record<string, unknown> | undefined) : undefined
  const nested = metadata?.[key]
  if (typeof nested === 'string' && nested.trim().length > 0) return nested
  const legacyDetails = log.details && typeof log.details === 'object' ? (log.details['legacy_details'] as Record<string, unknown> | undefined) : undefined
  const legacyMetadata = legacyDetails && typeof legacyDetails === 'object' ? (legacyDetails['metadata'] as Record<string, unknown> | undefined) : undefined
  const nestedLegacy = legacyMetadata?.[key]
  return typeof nestedLegacy === 'string' && nestedLegacy.trim().length > 0 ? nestedLegacy : null
}

export function getLogDetailPreview(log: CombinedEvent): string | null {
  if (log.event_type === 'commit') return readDetailString(log, 'commit_message')
  if (log.status === 'failed' || log.status === 'blocked') return readDetailString(log, 'reason')
  return null
}

export function getShortCommitSha(log: CombinedEvent): string | null {
  const sha = readDetailString(log, 'commit_sha')
  return sha ? sha.slice(0, 7) : null
}

export function extractTicketIdsFromCommitLog(log: CombinedEvent): string[] {
  const values = [readDetailString(log, 'commit_message'), log.branch ?? null].filter((v): v is string => typeof v === 'string' && v.trim().length > 0)
  const regex = /\b([A-Z][A-Z0-9]{1,15}-\d{1,9})\b/g
  const result: string[] = []
  const seen = new Set<string>()
  for (const value of values) {
    let match: RegExpExecArray | null
    regex.lastIndex = 0
    while ((match = regex.exec(value)) !== null) {
      const ticket = match[1].toUpperCase()
      if (!seen.has(ticket)) { seen.add(ticket); result.push(ticket) }
    }
  }
  return result
}

export function formatDurationMs(ms?: number): string {
  if (!ms || ms <= 0) return '-'
  const totalSeconds = Math.floor(ms / 1000)
  const minutes = Math.floor(totalSeconds / 60)
  const seconds = totalSeconds % 60
  if (minutes <= 0) return `${seconds}s`
  return `${minutes}m ${seconds}s`
}

export function readDetailFiles(log: CombinedEvent): string[] {
  const direct = log.details?.['files']
  if (Array.isArray(direct)) return direct.filter((v): v is string => typeof v === 'string')
  return []
}

export interface DashboardRow { log: CombinedEvent; attachedFiles: string[] }

export interface GitHubEvidenceSummary {
  prLifecycleCount: number
  prReviewCount: number
  prCommentCount: number
  statusCheckCount: number
  activeSignals: number
  totalSignals: number
  executiveStatus: 'Completo' | 'Parcial' | 'Sin evidencia'
  missingSignals: string[]
}

export interface GitHubEvidenceTrendPoint {
  capturedAt: string
  activeSignals: number
  totalSignals: number
  executiveStatus: GitHubEvidenceSummary['executiveStatus']
  missingSignals: string[]
}

interface AuditExportResponse {
  id: string
  export_type: string
  record_count: number
  content_hash: string
  data?: unknown
  created_at: number
}

export interface AuditExportPackage {
  export_id: string
  export_type: string
  record_count: number
  source_content_hash: string
  created_at: number
  packaged_at: string
  executive_summary: {
    github_evidence: GitHubEvidenceSummary
    scope_note: string
  }
  data: unknown
}

export function buildGitHubEvidenceSummary(githubByType: Record<string, number>): GitHubEvidenceSummary {
  const prLifecycleCount = githubByType.pull_request ?? 0
  const prReviewCount = githubByType.pull_request_review ?? 0
  const prCommentCount =
    (githubByType.pull_request_review_comment ?? 0) +
    (githubByType.issue_comment ?? 0)
  const statusCheckCount =
    (githubByType.check_run ?? 0) +
    (githubByType.check_suite ?? 0) +
    (githubByType.status ?? 0)

  const signals = [
    ['PR lifecycle', prLifecycleCount],
    ['Reviews', prReviewCount],
    ['Comentarios PR', prCommentCount],
    ['Checks/status', statusCheckCount],
  ] as const
  const activeSignals = signals.filter(([, count]) => count > 0).length
  const executiveStatus =
    activeSignals === signals.length
      ? 'Completo'
      : activeSignals > 0
        ? 'Parcial'
        : 'Sin evidencia'

  return {
    prLifecycleCount,
    prReviewCount,
    prCommentCount,
    statusCheckCount,
    activeSignals,
    totalSignals: signals.length,
    executiveStatus,
    missingSignals: signals
      .filter(([, count]) => count === 0)
      .map(([label]) => label),
  }
}

export function buildGitHubEvidenceTrendPoint(
  summary: GitHubEvidenceSummary,
  capturedAt = new Date().toISOString(),
): GitHubEvidenceTrendPoint {
  return {
    capturedAt,
    activeSignals: summary.activeSignals,
    totalSignals: summary.totalSignals,
    executiveStatus: summary.executiveStatus,
    missingSignals: summary.missingSignals,
  }
}

export function appendGitHubEvidenceTrendPoint(
  previous: GitHubEvidenceTrendPoint[],
  next: GitHubEvidenceTrendPoint,
  maxPoints = 12,
): GitHubEvidenceTrendPoint[] {
  const latest = previous[previous.length - 1]
  const shouldReplaceLatest =
    latest &&
    latest.activeSignals === next.activeSignals &&
    latest.totalSignals === next.totalSignals &&
    latest.executiveStatus === next.executiveStatus &&
    latest.missingSignals.join('|') === next.missingSignals.join('|')

  const merged = shouldReplaceLatest
    ? [...previous.slice(0, -1), next]
    : [...previous, next]

  return merged.slice(Math.max(0, merged.length - maxPoints))
}

export function buildAuditExportPackage(
  exportResponse: AuditExportResponse,
  githubByType: Record<string, number>,
  packagedAt = new Date().toISOString(),
): AuditExportPackage {
  return {
    export_id: exportResponse.id,
    export_type: exportResponse.export_type,
    record_count: exportResponse.record_count,
    source_content_hash: exportResponse.content_hash,
    created_at: exportResponse.created_at,
    packaged_at: packagedAt,
    executive_summary: {
      github_evidence: buildGitHubEvidenceSummary(githubByType),
      scope_note: 'Dashboard snapshot at export time; raw audit records remain in data.',
    },
    data: exportResponse.data ?? null,
  }
}

export function buildDashboardRows(logs: CombinedEvent[]): DashboardRow[] {
  const WINDOW_MS = 10 * 60 * 1000
  const rowsAscending: DashboardRow[] = []
  const pendingStageByUser = new Map<string, Array<{ created_at: number; files: string[] }>>()

  // Process oldest -> newest so each commit can consume the closest prior stage_files.
  for (let idx = logs.length - 1; idx >= 0; idx--) {
    const log = logs[idx]
    const login = (log.user_login ?? '').trim()

    if (log.event_type === 'stage_files') {
      if (!login) continue
      const files = readDetailFiles(log)
      if (!files.length) continue
      const queue = pendingStageByUser.get(login) ?? []
      queue.push({ created_at: log.created_at, files })
      pendingStageByUser.set(login, queue)
      continue
    }

    if (log.event_type !== 'commit') continue

    let attachedFiles: string[] = []
    if (login) {
      const queue = pendingStageByUser.get(login)
      if (queue && queue.length > 0) {
        // Drop stale candidates that are too old for this commit.
        while (queue.length > 0 && (log.created_at - queue[0].created_at) > WINDOW_MS) {
          queue.shift()
        }
        if (queue.length > 0) {
          const candidate = queue.pop()
          if (candidate && log.created_at >= candidate.created_at && (log.created_at - candidate.created_at) <= WINDOW_MS) {
            attachedFiles = candidate.files
          }
        }
        if (!queue.length) pendingStageByUser.delete(login)
      }
    }

    rowsAscending.push({ log, attachedFiles })
  }

  return rowsAscending.reverse()
}
