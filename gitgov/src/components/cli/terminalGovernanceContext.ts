import type { RepoValidation } from '@/lib/types'
import type {
  ChangeRiskEvaluationRecord,
  DeploymentGateAuthorizationRecord,
  MultiRepoExecutiveGovernanceRepository,
} from '@/store/useControlPlaneStore/types'
import type { NativeTerminalGitContext } from './terminalGitContext'

export type TerminalGovernanceTargetStatus =
  | 'pending'
  | 'no-git-repo'
  | 'missing-remote'
  | 'ready'

export interface TerminalGovernanceTarget {
  status: TerminalGovernanceTargetStatus
  repositoryFullName: string | null
  branch: string | null
  repoLabel: string
}

export interface TerminalGovernanceSnapshot {
  target: TerminalGovernanceTarget
  latestGate: DeploymentGateAuthorizationRecord | null
  latestRisk: ChangeRiskEvaluationRecord | null
  executiveRepository: MultiRepoExecutiveGovernanceRepository | null
  providerHealth: 'connected' | 'disconnected' | 'checking' | 'maintenance'
  error: string | null
}

export function parseRemoteRepositoryFullName(remoteUrl?: string | null): string | null {
  const trimmed = remoteUrl?.trim()
  if (!trimmed) return null

  const githubPathMatch = trimmed.match(/github\.com[:/](?<owner>[^/\s:]+)\/(?<repo>[^/\s]+?)(?:\.git)?(?:[#?].*)?$/i)
  if (!githubPathMatch?.groups) return null

  const owner = githubPathMatch.groups.owner.trim()
  const repo = githubPathMatch.groups.repo.replace(/\.git$/i, '').trim()
  if (!owner || !repo) return null

  return `${owner}/${repo}`
}

export function buildTerminalGovernanceTarget(
  context: NativeTerminalGitContext | null,
  validation: RepoValidation | null | undefined,
  currentBranch: string | null | undefined,
): TerminalGovernanceTarget {
  if (!context) {
    return {
      status: 'pending',
      repositoryFullName: null,
      branch: currentBranch?.trim() || null,
      repoLabel: 'Context pending',
    }
  }

  if (!context.is_git_repo) {
    return {
      status: 'no-git-repo',
      repositoryFullName: null,
      branch: null,
      repoLabel: 'No Git repository',
    }
  }

  const repositoryFullName = parseRemoteRepositoryFullName(validation?.remote_url)
  if (!repositoryFullName) {
    return {
      status: 'missing-remote',
      repositoryFullName: null,
      branch: context.branch?.trim() || currentBranch?.trim() || null,
      repoLabel: context.repo_name?.trim() || 'Git repository',
    }
  }

  return {
    status: 'ready',
    repositoryFullName,
    branch: context.branch?.trim() || currentBranch?.trim() || null,
    repoLabel: repositoryFullName,
  }
}

export function terminalGovernanceEmptyState(target: TerminalGovernanceTarget): string | null {
  if (target.status === 'pending') return 'Git context has not been detected yet.'
  if (target.status === 'no-git-repo') return 'Open the terminal inside a Git repository to load governance context.'
  if (target.status === 'missing-remote') return 'GitGov needs a GitHub remote to map this local repo to Control Plane evidence.'
  return null
}

export function terminalGovernanceHasEvidence(snapshot: TerminalGovernanceSnapshot): boolean {
  return Boolean(snapshot.latestGate || snapshot.latestRisk || snapshot.executiveRepository)
}
