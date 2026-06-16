import type { NativeTerminalGitContext } from './terminalGitContext'

export interface TerminalQuickCommand {
  id: string
  label: string
  command: string
  description: string
  requiresGitRepo: boolean
}

export interface TerminalQuickCommandView extends TerminalQuickCommand {
  disabled: boolean
  disabledReason?: string
}

export const SAFE_TERMINAL_QUICK_COMMANDS: TerminalQuickCommand[] = [
  {
    id: 'git-status-short',
    label: 'Status',
    command: 'git status --short',
    description: 'Show changed files without modifying the repository.',
    requiresGitRepo: true,
  },
  {
    id: 'git-branch-current',
    label: 'Branch',
    command: 'git branch --show-current',
    description: 'Print the current branch name.',
    requiresGitRepo: true,
  },
  {
    id: 'git-log-five',
    label: 'Recent commits',
    command: 'git log --oneline -5',
    description: 'Show the last five commits in compact form.',
    requiresGitRepo: true,
  },
  {
    id: 'git-diff-stat',
    label: 'Diff stat',
    command: 'git diff --stat',
    description: 'Summarize local diff size without printing file contents.',
    requiresGitRepo: true,
  },
  {
    id: 'git-remote-verbose',
    label: 'Remotes',
    command: 'git remote -v',
    description: 'List configured remote URLs locally for inspection.',
    requiresGitRepo: true,
  },
]

const MUTATING_COMMAND_PATTERN =
  /\b(push|pull|merge|rebase|commit|checkout|switch|reset|clean|apply|am|cherry-pick|revert|tag|fetch|deploy|destroy|delete|remove|rm|mv|kubectl|terraform|helm|gh)\b/i

export function isReadOnlyTerminalQuickCommand(command: string): boolean {
  const normalized = command.trim()
  if (!normalized) return false
  if (/[\r\n]/.test(normalized)) return false
  if (/&&|\|\||[;|`$<>]/.test(normalized)) return false
  if (!normalized.startsWith('git ')) return false
  return !MUTATING_COMMAND_PATTERN.test(normalized)
}

export function buildTerminalQuickCommandViews(
  context: NativeTerminalGitContext | null,
): TerminalQuickCommandView[] {
  const isGitRepo = context?.is_git_repo === true

  return SAFE_TERMINAL_QUICK_COMMANDS.map((quickCommand) => {
    const structurallySafe = isReadOnlyTerminalQuickCommand(quickCommand.command)
    const disabled = !structurallySafe || (quickCommand.requiresGitRepo && !isGitRepo)
    return {
      ...quickCommand,
      disabled,
      disabledReason: !structurallySafe
        ? 'Command is not in the read-only allowlist'
        : quickCommand.requiresGitRepo && !isGitRepo
          ? 'Open a Git repository terminal to insert this command'
          : undefined,
    }
  })
}

export function buildTerminalQuickCommandInsertInput(command: string): string {
  const normalized = command.trim()
  if (!isReadOnlyTerminalQuickCommand(normalized)) {
    throw new Error('Terminal quick command is not read-only')
  }
  return normalized
}
