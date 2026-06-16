import type { RepoValidation } from '@/lib/types'
import type {
  ChangeRiskEvaluationRecord,
  DeploymentGateAuthorizationRecord,
  MultiRepoExecutiveGovernanceRepository,
} from '@/store/useControlPlaneStore/types'
import type { NativeTerminalGitContext } from '@/components/cli/terminalGitContext'
import {
  buildTerminalGovernanceTarget,
  parseRemoteRepositoryFullName,
  terminalGovernanceEmptyState,
  terminalGovernanceHasEvidence,
  type TerminalGovernanceSnapshot,
} from '@/components/cli/terminalGovernanceContext'

const gitContext: NativeTerminalGitContext = {
  cwd: 'C:/Users/PC/Desktop/GitGov',
  is_git_repo: true,
  is_detached: false,
  repo_name: 'GitGov',
  branch: 'main',
  commit_short: '1c8bf12',
  detected_at_ms: 1_700_000_000_000,
}

const validation: RepoValidation = {
  path_exists: true,
  is_git_repo: true,
  has_remote_origin: true,
  has_gitgov_toml: false,
  has_gitgov_policy: true,
  policy_path: '.gitgov/policy.yml',
  policy_format: 'yaml',
  policy_error: null,
  remote_url: 'git@github.com:yohandry10/Git-Gov.git',
}

function makeSnapshot(overrides: Partial<TerminalGovernanceSnapshot> = {}): TerminalGovernanceSnapshot {
  return {
    target: buildTerminalGovernanceTarget(gitContext, validation, 'main'),
    latestGate: null,
    latestRisk: null,
    executiveRepository: null,
    providerHealth: 'connected',
    error: null,
    ...overrides,
  }
}

describe('native terminal governance context helpers', () => {
  it('parses common GitHub remote URLs into owner/repo without exposing full URL details', () => {
    expect(parseRemoteRepositoryFullName('git@github.com:yohandry10/Git-Gov.git')).toBe('yohandry10/Git-Gov')
    expect(parseRemoteRepositoryFullName('https://github.com/yohandry10/Git-Gov.git')).toBe('yohandry10/Git-Gov')
    expect(parseRemoteRepositoryFullName('ssh://git@github.com/yohandry10/Git-Gov.git')).toBe('yohandry10/Git-Gov')
    expect(parseRemoteRepositoryFullName('https://gitlab.com/yohandry10/Git-Gov.git')).toBeNull()
    expect(parseRemoteRepositoryFullName('')).toBeNull()
  })

  it('builds a ready target from KAN-133 context and repo validation without leaking cwd', () => {
    const target = buildTerminalGovernanceTarget(gitContext, validation, 'main')

    expect(target).toEqual({
      status: 'ready',
      repositoryFullName: 'yohandry10/Git-Gov',
      branch: 'main',
      repoLabel: 'yohandry10/Git-Gov',
    })
    expect(JSON.stringify(target)).not.toContain('C:/Users/PC/Desktop')
  })

  it('reports safe empty states for pending, non-git, and missing remote context', () => {
    const pending = buildTerminalGovernanceTarget(null, null, 'main')
    expect(pending.status).toBe('pending')
    expect(terminalGovernanceEmptyState(pending)).toContain('not been detected')

    const nonGit = buildTerminalGovernanceTarget({ ...gitContext, is_git_repo: false }, validation, 'main')
    expect(nonGit.status).toBe('no-git-repo')
    expect(terminalGovernanceEmptyState(nonGit)).toContain('inside a Git repository')

    const missingRemote = buildTerminalGovernanceTarget(gitContext, { ...validation, remote_url: undefined }, 'main')
    expect(missingRemote.status).toBe('missing-remote')
    expect(terminalGovernanceEmptyState(missingRemote)).toContain('GitHub remote')
  })

  it('treats any existing gate, risk, or executive row as evidence without requiring enforcement', () => {
    expect(terminalGovernanceHasEvidence(makeSnapshot())).toBe(false)
    expect(terminalGovernanceHasEvidence(makeSnapshot({
      latestGate: { authorization_id: 'dga_1' } as DeploymentGateAuthorizationRecord,
    }))).toBe(true)
    expect(terminalGovernanceHasEvidence(makeSnapshot({
      latestRisk: { evaluation_id: 'cra_1' } as ChangeRiskEvaluationRecord,
    }))).toBe(true)
    expect(terminalGovernanceHasEvidence(makeSnapshot({
      executiveRepository: { repository_full_name: 'yohandry10/Git-Gov' } as MultiRepoExecutiveGovernanceRepository,
    }))).toBe(true)
  })
})
