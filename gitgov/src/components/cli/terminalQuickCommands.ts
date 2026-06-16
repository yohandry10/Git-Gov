import type { NativeTerminalGitContext } from './terminalGitContext'
import {
  terminalToolContextHasCommand,
  type NativeTerminalToolContext,
} from './terminalToolContext'

export interface TerminalQuickCommand {
  id: string
  label: string
  command: string
  description: string
  group: 'git' | 'provider-tool'
  tool: 'git' | 'terraform' | 'kubernetes' | 'docker-compose' | 'helm'
  enabled: boolean
  safetyLevel: 'local-read-only'
  requiresNetwork: boolean
  mayExposeSecrets: boolean
  requiresGitRepo: boolean
}

export interface TerminalQuickCommandView extends TerminalQuickCommand {
  availableInWorkspace: boolean
  disabled: boolean
  disabledReason?: string
}

export const SAFE_TERMINAL_QUICK_COMMANDS: TerminalQuickCommand[] = [
  {
    id: 'git-status-short',
    label: 'Status',
    command: 'git status --short',
    description: 'Show changed files without modifying the repository.',
    group: 'git',
    tool: 'git',
    enabled: true,
    safetyLevel: 'local-read-only',
    requiresNetwork: false,
    mayExposeSecrets: false,
    requiresGitRepo: true,
  },
  {
    id: 'git-branch-current',
    label: 'Branch',
    command: 'git branch --show-current',
    description: 'Print the current branch name.',
    group: 'git',
    tool: 'git',
    enabled: true,
    safetyLevel: 'local-read-only',
    requiresNetwork: false,
    mayExposeSecrets: false,
    requiresGitRepo: true,
  },
  {
    id: 'git-log-five',
    label: 'Recent commits',
    command: 'git log --oneline -5',
    description: 'Show the last five commits in compact form.',
    group: 'git',
    tool: 'git',
    enabled: true,
    safetyLevel: 'local-read-only',
    requiresNetwork: false,
    mayExposeSecrets: false,
    requiresGitRepo: true,
  },
  {
    id: 'git-diff-stat',
    label: 'Diff stat',
    command: 'git diff --stat',
    description: 'Summarize local diff size without printing file contents.',
    group: 'git',
    tool: 'git',
    enabled: true,
    safetyLevel: 'local-read-only',
    requiresNetwork: false,
    mayExposeSecrets: false,
    requiresGitRepo: true,
  },
  {
    id: 'git-remote-names',
    label: 'Remotes',
    command: 'git remote',
    description: 'List configured remote names without printing remote URLs.',
    group: 'git',
    tool: 'git',
    enabled: true,
    safetyLevel: 'local-read-only',
    requiresNetwork: false,
    mayExposeSecrets: false,
    requiresGitRepo: true,
  },
  {
    id: 'terraform-fmt-check',
    label: 'Terraform fmt',
    command: 'terraform fmt -check -recursive',
    description: 'Check Terraform formatting locally without rewriting files.',
    group: 'provider-tool',
    tool: 'terraform',
    enabled: true,
    safetyLevel: 'local-read-only',
    requiresNetwork: false,
    mayExposeSecrets: false,
    requiresGitRepo: true,
  },
  {
    id: 'terraform-validate',
    label: 'Terraform validate',
    command: 'terraform validate -no-color',
    description: 'Validate initialized Terraform configuration without applying changes.',
    group: 'provider-tool',
    tool: 'terraform',
    enabled: true,
    safetyLevel: 'local-read-only',
    requiresNetwork: false,
    mayExposeSecrets: false,
    requiresGitRepo: true,
  },
  {
    id: 'kubectl-current-context',
    label: 'Kube context',
    command: 'kubectl config current-context',
    description: 'Show the current local Kubernetes context without calling the cluster API.',
    group: 'provider-tool',
    tool: 'kubernetes',
    enabled: true,
    safetyLevel: 'local-read-only',
    requiresNetwork: false,
    mayExposeSecrets: false,
    requiresGitRepo: true,
  },
  {
    id: 'kubectl-list-contexts',
    label: 'Kube contexts',
    command: 'kubectl config get-contexts',
    description: 'List local Kubernetes contexts without mutating cluster state.',
    group: 'provider-tool',
    tool: 'kubernetes',
    enabled: true,
    safetyLevel: 'local-read-only',
    requiresNetwork: false,
    mayExposeSecrets: false,
    requiresGitRepo: true,
  },
  {
    id: 'docker-compose-services',
    label: 'Compose services',
    command: 'docker compose config --services',
    description: 'List services from Compose configuration without starting containers.',
    group: 'provider-tool',
    tool: 'docker-compose',
    enabled: true,
    safetyLevel: 'local-read-only',
    requiresNetwork: false,
    mayExposeSecrets: false,
    requiresGitRepo: true,
  },
  {
    id: 'docker-compose-check',
    label: 'Compose check',
    command: 'docker compose config --quiet',
    description: 'Validate Compose configuration without starting or stopping services.',
    group: 'provider-tool',
    tool: 'docker-compose',
    enabled: true,
    safetyLevel: 'local-read-only',
    requiresNetwork: false,
    mayExposeSecrets: false,
    requiresGitRepo: true,
  },
  {
    id: 'helm-lint-local',
    label: 'Helm lint',
    command: 'helm lint .',
    description: 'Lint the local Helm chart without installing or upgrading releases.',
    group: 'provider-tool',
    tool: 'helm',
    enabled: true,
    safetyLevel: 'local-read-only',
    requiresNetwork: false,
    mayExposeSecrets: false,
    requiresGitRepo: true,
  },
]

const enabledCommandRegistry = new Map(
  SAFE_TERMINAL_QUICK_COMMANDS
    .filter((command) => command.enabled && !command.requiresNetwork && !command.mayExposeSecrets)
    .map((command) => [command.command, command]),
)

export function terminalQuickCommandGroupLabel(group: TerminalQuickCommand['group']): string {
  if (group === 'provider-tool') return 'Provider / Tool context'
  return 'Git inspection'
}

export function isReadOnlyTerminalQuickCommand(command: string): boolean {
  const normalized = command.trim()
  if (!normalized) return false
  if (/[\r\n]/.test(normalized)) return false
  if (/&&|\|\||[;|`$<>]/.test(normalized)) return false
  return enabledCommandRegistry.has(normalized)
}

export function buildTerminalQuickCommandViews(
  context: NativeTerminalGitContext | null,
  toolContext?: NativeTerminalToolContext | null,
): TerminalQuickCommandView[] {
  const isGitRepo = context?.is_git_repo === true

  return SAFE_TERMINAL_QUICK_COMMANDS.map((quickCommand) => {
    const structurallySafe = isReadOnlyTerminalQuickCommand(quickCommand.command)
    const availableInWorkspace = quickCommand.group === 'provider-tool' &&
      terminalToolContextHasCommand(toolContext ?? null, quickCommand.tool, quickCommand.id)
    const disabled = !quickCommand.enabled ||
      quickCommand.requiresNetwork ||
      quickCommand.mayExposeSecrets ||
      !structurallySafe ||
      (quickCommand.requiresGitRepo && !isGitRepo)
    return {
      ...quickCommand,
      availableInWorkspace,
      disabled,
      disabledReason: !quickCommand.enabled
        ? 'Command is disabled pending safety review'
        : quickCommand.requiresNetwork
          ? 'Command requires network access and is disabled for this MVP'
          : quickCommand.mayExposeSecrets
            ? 'Command may expose secrets and is disabled'
            : !structurallySafe
              ? 'Command is not in the read-only safety registry'
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
