export type GitIdentityScope =
  | 'programdata'
  | 'system'
  | 'xdg'
  | 'global'
  | 'local'
  | 'worktree'
  | 'app'
  | 'highest'
  | 'unknown'

export interface GitIdentity {
  name: string | null
  email: string | null
  name_scope?: GitIdentityScope | null
  email_scope?: GitIdentityScope | null
  name_source?: string | null
  email_source?: string | null
}

export interface GitIdentityUser {
  login: string
  name?: string | null
  email?: string | null
}

export type GitIdentityFindingReason = 'incomplete' | 'not_provably_aligned'

export interface GitIdentityFinding {
  reason: GitIdentityFindingReason
  effectiveName: string
  effectiveEmail: string
  nameScope: GitIdentityScope | null
  emailScope: GitIdentityScope | null
  nameSource: string | null
  emailSource: string | null
  suggestedName: string
  suggestedEmail: string
}

export interface GitIdentityEvidenceLine {
  lineType: 'command' | 'stdout' | 'stderr' | 'system'
  text: string
  auditable?: boolean
}

function normalize(value?: string | null): string {
  return value?.trim() ?? ''
}

function isPlaceholderIdentityValue(value?: string | null): boolean {
  const normalizedValue = normalize(value).toLowerCase()
  return (
    normalizedValue === '' ||
    normalizedValue === 'unknown' ||
    normalizedValue === 'null' ||
    normalizedValue === 'undefined' ||
    normalizedValue === 'n/a'
  )
}

function normalizeIdentitySignal(value?: string | null): string {
  return isPlaceholderIdentityValue(value) ? '' : normalize(value)
}

function signalsEqual(left?: string | null, right?: string | null): boolean {
  const normalizedLeft = normalizeIdentitySignal(left).toLowerCase()
  const normalizedRight = normalizeIdentitySignal(right).toLowerCase()
  return normalizedLeft.length > 0 && normalizedLeft === normalizedRight
}

function isGitHubNoReplyEmail(email: string, login: string): boolean {
  const normalizedEmail = email.toLowerCase()
  const normalizedLogin = login.toLowerCase()
  return (
    normalizedEmail === `${normalizedLogin}@users.noreply.github.com` ||
    normalizedEmail.endsWith(`+${normalizedLogin}@users.noreply.github.com`)
  )
}

function isGitIdentityAligned(identity: GitIdentity, user: GitIdentityUser): boolean {
  const name = normalize(identity.name)
  const email = normalize(identity.email)
  const login = normalizeIdentitySignal(user.login)
  const publicName = normalizeIdentitySignal(user.name)
  const publicEmail = normalizeIdentitySignal(user.email)
  const noreplyEmail = login ? `${login}@users.noreply.github.com` : ''

  return (
    signalsEqual(name, login) ||
    signalsEqual(name, publicName) ||
    (publicEmail.length > 0 && email.toLowerCase() === publicEmail.toLowerCase()) ||
    (noreplyEmail.length > 0 && isGitHubNoReplyEmail(email, login))
  )
}

export function evaluateGitIdentity(
  identity: GitIdentity | null,
  user: GitIdentityUser | null,
): GitIdentityFinding | null {
  if (!identity || !user) return null

  const effectiveName = normalize(identity.name)
  const effectiveEmail = normalize(identity.email)
  const suggestedName = normalizeIdentitySignal(user.name) || normalizeIdentitySignal(user.login)
  const loginSignal = normalizeIdentitySignal(user.login)
  const suggestedEmail = loginSignal ? `${loginSignal}@users.noreply.github.com` : ''

  if (!effectiveName || !effectiveEmail) {
    return {
      reason: 'incomplete',
      effectiveName,
      effectiveEmail,
      nameScope: identity.name_scope ?? null,
      emailScope: identity.email_scope ?? null,
      nameSource: identity.name_source ?? null,
      emailSource: identity.email_source ?? null,
      suggestedName,
      suggestedEmail,
    }
  }

  if (!isGitIdentityAligned(identity, user)) {
    return {
      reason: 'not_provably_aligned',
      effectiveName,
      effectiveEmail,
      nameScope: identity.name_scope ?? null,
      emailScope: identity.email_scope ?? null,
      nameSource: identity.name_source ?? null,
      emailSource: identity.email_source ?? null,
      suggestedName,
      suggestedEmail,
    }
  }

  return null
}

export function formatGitIdentityScope(scope?: GitIdentityScope | null): string {
  switch (scope) {
    case 'local':
      return 'local del repo'
    case 'worktree':
      return 'worktree del repo'
    case 'global':
      return 'global de Git'
    case 'xdg':
      return 'global XDG'
    case 'system':
      return 'system de Git'
    case 'programdata':
      return 'ProgramData de Git'
    case 'app':
      return 'app'
    case 'highest':
      return 'highest'
    case 'unknown':
    default:
      return 'origen no reportado'
  }
}

export function formatGitIdentityValue(
  value?: string | null,
  scope?: GitIdentityScope | null,
  source?: string | null,
): string {
  const normalizedValue = normalize(value)
  if (!normalizedValue) return '(no configurado)'

  const sourceSuffix = source ? `; ${source}` : ''
  return `"${normalizedValue}" [${formatGitIdentityScope(scope)}${sourceSuffix}]`
}

export function formatGitIdentityBlockToast(
  action: 'Commit' | 'Push',
  finding: GitIdentityFinding,
  login?: string | null,
): string {
  if (finding.reason === 'incomplete') {
    return `${action} bloqueado: Git no tiene user.name y user.email efectivos para este repo. Usa "Ver prueba" y configura la identidad local antes de continuar.`
  }

  return `${action} bloqueado: Git resolverá "${finding.effectiveName} <${finding.effectiveEmail}>" para CLI/manual, pero GitGov Desktop está autenticado como @${normalize(login) || 'usuario'}. Usa "Ver prueba" para revisar el origen.`
}

export function buildGitIdentityEvidenceLines(
  identity: GitIdentity,
  user: GitIdentityUser,
  finding: GitIdentityFinding | null,
): GitIdentityEvidenceLine[] {
  const result =
    finding?.reason === 'incomplete'
      ? 'Git identity incomplete: configure user.name and user.email.'
      : finding?.reason === 'not_provably_aligned'
        ? 'Git identity is not provably aligned with the authenticated GitHub user.'
        : 'Git identity is aligned with the authenticated GitHub user.'

  return [
    { lineType: 'system', text: '[GitGov] Git identity proof (read-only)', auditable: false },
    { lineType: 'command', text: '$ git config --get user.name', auditable: false },
    {
      lineType: identity.name ? 'stdout' : 'stderr',
      text: formatGitIdentityValue(identity.name, identity.name_scope, identity.name_source),
      auditable: false,
    },
    { lineType: 'command', text: '$ git config --get user.email', auditable: false },
    {
      lineType: identity.email ? 'stdout' : 'stderr',
      text: formatGitIdentityValue(identity.email, identity.email_scope, identity.email_source),
      auditable: false,
    },
    { lineType: 'system', text: `[GitGov] Authenticated GitHub user: @${user.login}`, auditable: false },
    { lineType: finding ? 'stderr' : 'stdout', text: `[GitGov] ${result}`, auditable: false },
  ]
}
