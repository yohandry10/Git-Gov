import { isTauriDesktop } from '@/lib/tauri'

export type DesktopUpdateChannel = 'stable' | 'beta'

export interface DesktopUpdateInfo {
  currentVersion: string
  version: string
  date?: string
  body?: string
  rawJson?: Record<string, unknown>
}

export type DesktopUpdateEnforcementReason = 'none' | 'force-update' | 'min-supported-version'

export interface DesktopUpdateEnforcement {
  required: boolean
  reason: DesktopUpdateEnforcementReason
  forceUpdate: boolean
  currentBelowMinSupported: boolean
  minSupportedVersion: string | null
  note: string | null
}

export interface DesktopUpdateProgress {
  downloadedBytes: number
  totalBytes?: number
}

const PUBLIC_REPO_URL =
  (import.meta.env.VITE_PUBLIC_REPO_URL as string | undefined)?.trim() ||
  'https://github.com'
const DEFAULT_FALLBACK_DOWNLOAD_URL =
  /^https?:\/\/github\.com\/?$/i.test(PUBLIC_REPO_URL)
    ? PUBLIC_REPO_URL
    : /\/releases\/latest$/i.test(PUBLIC_REPO_URL)
      ? PUBLIC_REPO_URL
      : `${PUBLIC_REPO_URL.replace(/\/+$/, '')}/releases/latest`
const UPDATE_CHANNEL_HEADER = 'x-gitgov-update-channel'

function normalizeChannel(channel: string | undefined): DesktopUpdateChannel {
  return channel === 'beta' ? 'beta' : 'stable'
}

function coerceRecord(value: unknown): Record<string, unknown> | null {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    return null
  }
  return value as Record<string, unknown>
}

function readString(value: unknown): string | null {
  if (typeof value !== 'string') return null
  const normalized = value.trim()
  return normalized.length > 0 ? normalized : null
}

function readBoolean(value: unknown): boolean {
  if (typeof value === 'boolean') return value
  if (typeof value === 'number') return value === 1
  if (typeof value !== 'string') return false
  const normalized = value.trim().toLowerCase()
  return normalized === '1' || normalized === 'true' || normalized === 'yes' || normalized === 'on'
}

interface ParsedVersion {
  parts: number[]
  prerelease: string | null
}

function parseVersion(input: string): ParsedVersion | null {
  const normalized = input.trim().replace(/^v/i, '')
  if (!normalized) return null

  const [withoutBuild] = normalized.split('+', 1)
  const [core, prereleaseRaw] = withoutBuild.split('-', 2)
  if (!core) return null

  const coreTokens = core.split('.')
  if (coreTokens.length === 0) return null

  const parts = coreTokens.map((token) => {
    if (!/^\d+$/.test(token.trim())) return Number.NaN
    return Number.parseInt(token, 10)
  })
  if (parts.some((part) => Number.isNaN(part))) return null

  while (parts.length < 3) parts.push(0)
  return {
    parts,
    prerelease: prereleaseRaw?.trim() || null,
  }
}

function comparePrerelease(a: string | null, b: string | null): number {
  if (!a && !b) return 0
  if (!a && b) return 1
  if (a && !b) return -1
  if (!a || !b) return 0

  const aParts = a.split('.')
  const bParts = b.split('.')
  const maxLen = Math.max(aParts.length, bParts.length)

  for (let i = 0; i < maxLen; i += 1) {
    const aPart = aParts[i]
    const bPart = bParts[i]
    if (aPart === undefined) return -1
    if (bPart === undefined) return 1
    if (aPart === bPart) continue

    const aNumeric = /^\d+$/.test(aPart)
    const bNumeric = /^\d+$/.test(bPart)
    if (aNumeric && bNumeric) {
      return Number.parseInt(aPart, 10) < Number.parseInt(bPart, 10) ? -1 : 1
    }
    if (aNumeric && !bNumeric) return -1
    if (!aNumeric && bNumeric) return 1
    return aPart < bPart ? -1 : 1
  }

  return 0
}

export function compareAppVersions(currentVersion: string, targetVersion: string): number {
  const current = parseVersion(currentVersion)
  const target = parseVersion(targetVersion)

  if (current && target) {
    const maxLen = Math.max(current.parts.length, target.parts.length)
    for (let i = 0; i < maxLen; i += 1) {
      const left = current.parts[i] ?? 0
      const right = target.parts[i] ?? 0
      if (left === right) continue
      return left < right ? -1 : 1
    }
    return comparePrerelease(current.prerelease, target.prerelease)
  }

  const normalizedCurrent = currentVersion.trim().toLowerCase()
  const normalizedTarget = targetVersion.trim().toLowerCase()
  if (normalizedCurrent === normalizedTarget) return 0
  return normalizedCurrent < normalizedTarget ? -1 : 1
}

export function evaluateDesktopUpdateEnforcement(info: DesktopUpdateInfo): DesktopUpdateEnforcement {
  const raw = coerceRecord(info.rawJson)
  const minSupportedVersion =
    readString(raw?.min_supported_version) ??
    readString(raw?.minSupportedVersion) ??
    readString(raw?.minimum_supported_version) ??
    readString(raw?.minimumSupportedVersion)
  const forceUpdate =
    readBoolean(raw?.force_update) ||
    readBoolean(raw?.forceUpdate) ||
    readBoolean(raw?.mandatory_update) ||
    readBoolean(raw?.mandatoryUpdate) ||
    readBoolean(raw?.critical_update) ||
    readBoolean(raw?.criticalUpdate)
  const note =
    readString(raw?.force_update_reason) ??
    readString(raw?.forceUpdateReason) ??
    readString(raw?.update_notice) ??
    readString(raw?.updateNotice)

  const currentBelowMinSupported = Boolean(
    minSupportedVersion && compareAppVersions(info.currentVersion, minSupportedVersion) < 0
  )
  const reason: DesktopUpdateEnforcementReason = forceUpdate
    ? 'force-update'
    : currentBelowMinSupported
      ? 'min-supported-version'
      : 'none'

  return {
    required: forceUpdate || currentBelowMinSupported,
    reason,
    forceUpdate,
    currentBelowMinSupported,
    minSupportedVersion: minSupportedVersion ?? null,
    note: note ?? null,
  }
}

export function getDesktopUpdateFallbackUrl(channel?: DesktopUpdateChannel): string {
  const envValue = (import.meta.env.VITE_DESKTOP_DOWNLOAD_FALLBACK_URL as string | undefined)?.trim()
  const selectedChannel = normalizeChannel(channel)
  const base = envValue || DEFAULT_FALLBACK_DOWNLOAD_URL
  if (base.includes('{channel}')) {
    return base.replaceAll('{channel}', selectedChannel)
  }
  if (/^https?:\/\/github\.com\/?$/i.test(base)) {
    return base
  }
  if (/\/releases\/latest$/i.test(base) || /\.exe$/i.test(base) || /\.json$/i.test(base)) {
    return base
  }
  return `${base.replace(/\/+$/, '')}/${selectedChannel}`
}

export function canUseDesktopUpdater(): boolean {
  return isTauriDesktop()
}

export function isUpdaterNotConfiguredError(error: unknown): boolean {
  const message = String(error ?? '').toLowerCase()
  return (
    message.includes('updater') &&
    (message.includes('config') ||
      message.includes('endpoint') ||
      message.includes('pubkey') ||
      message.includes('not configured'))
  )
}

export function normalizeUpdaterErrorMessage(error: unknown): string {
  const raw = String(error ?? '')
  const message = raw.toLowerCase()

  if (message.includes('error decoding response body')) {
    return 'No se pudo leer la respuesta del servidor de actualizaciones (latest.json inválido o ausente). Usa "Descarga manual" mientras se corrige el release metadata.'
  }

  if (message.includes('404') && message.includes('latest.json')) {
    return 'No se encontró latest.json para el updater. Publica ese archivo en la release o usa "Descarga manual".'
  }

  if (message.includes('tls') || message.includes('certificate')) {
    return 'No se pudo conectar de forma segura al servidor de actualizaciones (TLS/certificado). Revisa red/certificados o usa "Descarga manual".'
  }

  return raw
}

function buildChannelHeaders(channel: DesktopUpdateChannel) {
  return {
    [UPDATE_CHANNEL_HEADER]: normalizeChannel(channel),
  }
}

export async function checkDesktopUpdate(channel: DesktopUpdateChannel = 'stable') {
  if (!canUseDesktopUpdater()) {
    throw new Error('Updater disponible solo en GitGov Desktop (Tauri).')
  }
  const updater = await import('@tauri-apps/plugin-updater')
  return updater.check({
    headers: buildChannelHeaders(channel),
  })
}

export async function downloadAndInstallDesktopUpdate(
  update: Awaited<ReturnType<typeof checkDesktopUpdate>> extends infer T
    ? T extends { downloadAndInstall: unknown }
      ? T
      : never
    : never,
  onProgress: (progress: DesktopUpdateProgress) => void,
  options?: { channel?: DesktopUpdateChannel }
) {
  let downloadedBytes = 0
  const channel = normalizeChannel(options?.channel)
  await update.downloadAndInstall((event) => {
    if (event.event === 'Started') {
      downloadedBytes = 0
      onProgress({ downloadedBytes, totalBytes: event.data.contentLength })
      return
    }
    if (event.event === 'Progress') {
      downloadedBytes += event.data.chunkLength
      onProgress({ downloadedBytes })
      return
    }
    if (event.event === 'Finished') {
      onProgress({ downloadedBytes })
    }
  }, {
    headers: buildChannelHeaders(channel),
  })
}

