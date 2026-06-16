export interface NativeTerminalGitContext {
  cwd: string
  is_git_repo: boolean
  is_detached: boolean
  repo_name?: string | null
  branch?: string | null
  commit_short?: string | null
  detected_at_ms: number
}

export function shouldRefreshTerminalGitContext(command: string): boolean {
  const trimmed = command.trim()
  if (!trimmed || /&&|\|\||[;|]/.test(trimmed)) return false

  return /^(cd|chdir|sl|set-location)(\s+.+)?$/i.test(trimmed)
}

export function formatTerminalGitContextLabel(context: NativeTerminalGitContext | null): string {
  if (!context) return 'context pending'
  if (!context.is_git_repo) return 'No git repo'

  const repo = context.repo_name?.trim() || 'repo'
  if (context.is_detached) {
    return `${repo}:detached${context.commit_short ? `@${context.commit_short}` : ''}`
  }

  return `${repo}:${context.branch?.trim() || 'unknown'}`
}

export function terminalGitContextTitle(context: NativeTerminalGitContext | null): string {
  if (!context) return 'Git context has not been detected for this terminal session yet'
  if (!context.is_git_repo) return 'Current terminal directory is not inside a Git repository'

  if (context.is_detached) {
    return `Git repository ${context.repo_name || 'repo'} is in detached HEAD${context.commit_short ? ` at ${context.commit_short}` : ''}`
  }

  return `Git repository ${context.repo_name || 'repo'} on branch ${context.branch || 'unknown'}`
}
