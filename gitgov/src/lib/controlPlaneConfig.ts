export const DEFAULT_CONTROL_PLANE_URL = 'http://127.0.0.1:3000'

const CONNECTION_ERROR_PATTERNS = [
  'error sending request',
  'failed to fetch',
  'network error',
  'could not connect',
  'connection refused',
  'connection reset',
  'connection closed',
  'no es posible conectar',
  'se produjo un error durante el intento de conexion',
  'econnrefused',
]

export function normalizeControlPlaneUrl(url: string | null | undefined): string {
  const trimmed = (url ?? '').trim()
  if (!trimmed) return ''

  try {
    const parsed = new URL(trimmed)
    if (parsed.hostname === 'localhost') {
      parsed.hostname = '127.0.0.1'
    }
    // Control Plane config is a base URL. Route paths such as /health must not persist.
    parsed.pathname = '/'
    parsed.search = ''
    parsed.hash = ''
    return parsed.origin
  } catch {
    // Keep invalid user input intact; the backend command reports validation/connectivity errors.
    return trimmed
  }
}

export function getEnvControlPlaneUrl(): string {
  return normalizeControlPlaneUrl(import.meta.env.VITE_SERVER_URL as string | undefined)
}

export function isDefaultLocalControlPlaneUrl(url: string | null | undefined): boolean {
  return normalizeControlPlaneUrl(url) === DEFAULT_CONTROL_PLANE_URL
}

export function isLocalControlPlaneUrl(url: string | null | undefined): boolean {
  const normalized = normalizeControlPlaneUrl(url)
  if (!normalized) return false
  try {
    const parsed = new URL(normalized)
    return ['127.0.0.1', 'localhost', '::1', '[::1]'].includes(parsed.hostname)
  } catch {
    return false
  }
}

export function resolveControlPlaneUrl(options: {
  inputUrl?: string | null
  previousUrl?: string | null
  storedUrl?: string | null
  envUrl?: string | null
} = {}): string {
  const inputUrl = normalizeControlPlaneUrl(options.inputUrl)
  if (inputUrl) return inputUrl

  const envUrl = normalizeControlPlaneUrl(options.envUrl ?? getEnvControlPlaneUrl())
  const previousUrl = normalizeControlPlaneUrl(options.previousUrl)
  const storedUrl = normalizeControlPlaneUrl(options.storedUrl)

  // Migrate the old dev behavior that persisted a forced localhost even when env had a real target.
  if (envUrl && (isDefaultLocalControlPlaneUrl(previousUrl) || isDefaultLocalControlPlaneUrl(storedUrl))) {
    return envUrl
  }

  return previousUrl || storedUrl || envUrl || DEFAULT_CONTROL_PLANE_URL
}

export function formatControlPlaneConnectionError(message: string, url: string | null | undefined): string {
  const normalizedMessage = message.trim()
  const lower = normalizedMessage.toLowerCase()
  const target = normalizeControlPlaneUrl(url) || DEFAULT_CONTROL_PLANE_URL
  const isConnectivityError = CONNECTION_ERROR_PATTERNS.some((pattern) => lower.includes(pattern))

  if (!isConnectivityError) return normalizedMessage

  const localHint = isLocalControlPlaneUrl(target)
    ? 'No hay un Control Plane local escuchando en esa direccion. Levanta el servidor local o cambia la URL a tu Control Plane configurado.'
    : 'Verifica que la URL del Control Plane sea correcta y que el servidor este accesible desde Desktop.'

  return `No se pudo conectar al Control Plane en ${target}. ${localHint}`
}
