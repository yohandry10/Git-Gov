import { describe, expect, it, vi } from 'vitest'
import { GitGovReadOnlyClient } from '../src/api'
import { loadGovernanceContext } from '../src/context'
import type { GitContext } from '../src/types'

const gitContext: GitContext = {
  isGitRepository: true,
  repositoryFullName: 'yohandry10/Git-Gov',
  branch: 'main',
  rootPath: 'C:/repo/GitGov',
  error: null,
}

describe('loadGovernanceContext', () => {
  it('loads gate/risk/executive data through read-only client methods', async () => {
    const client = {
      listDeploymentGates: vi.fn(async () => ({ items: [{ authorization_id: 'dga_123', decision: 'advisory' }] })),
      listChangeRisks: vi.fn(async () => ({ items: [{ evaluation_id: 'cra_123', risk_level: 'medium', review_status: 'accepted_risk' }] })),
      getExecutiveRepository: vi.fn(async () => ({ repository_full_name: 'yohandry10/Git-Gov', posture: 'review' })),
    } as unknown as GitGovReadOnlyClient

    const snapshot = await loadGovernanceContext({
      git: gitContext,
      config: { apiUrl: 'https://gitgov-api.example', orgName: 'yohandry10' },
      apiKey: 'readonly-secret',
      client,
    })

    expect(snapshot.error).toBeNull()
    expect(snapshot.latestGate?.authorization_id).toBe('dga_123')
    expect(snapshot.latestRisk?.review_status).toBe('accepted_risk')
    expect(snapshot.executiveRepository?.posture).toBe('review')
    expect(client.listDeploymentGates).toHaveBeenCalledWith('yohandry10/Git-Gov', 'main')
    expect(client.listChangeRisks).toHaveBeenCalledWith('yohandry10/Git-Gov', 'main')
    expect(client.getExecutiveRepository).toHaveBeenCalledWith('yohandry10/Git-Gov')
  })

  it('does not call GitGov when workspace is not a git repository', async () => {
    const client = {
      listDeploymentGates: vi.fn(),
      listChangeRisks: vi.fn(),
      getExecutiveRepository: vi.fn(),
    } as unknown as GitGovReadOnlyClient

    const snapshot = await loadGovernanceContext({
      git: {
        isGitRepository: false,
        repositoryFullName: null,
        branch: null,
        rootPath: null,
        error: 'No Git repository detected.',
      },
      config: { apiUrl: 'https://gitgov-api.example', orgName: 'yohandry10' },
      apiKey: 'readonly-secret',
      client,
    })

    expect(snapshot.error).toBe('No Git repository detected.')
    expect(client.listDeploymentGates).not.toHaveBeenCalled()
    expect(client.listChangeRisks).not.toHaveBeenCalled()
    expect(client.getExecutiveRepository).not.toHaveBeenCalled()
  })

  it('does not call GitGov until API URL, org, and SecretStorage API key are configured', async () => {
    const client = {
      listDeploymentGates: vi.fn(),
      listChangeRisks: vi.fn(),
      getExecutiveRepository: vi.fn(),
    } as unknown as GitGovReadOnlyClient

    const snapshot = await loadGovernanceContext({
      git: gitContext,
      config: { apiUrl: 'https://gitgov-api.example', orgName: 'yohandry10' },
      apiKey: undefined,
      client,
    })

    expect(snapshot.error).toContain('Configure GitGov API URL')
    expect(client.listDeploymentGates).not.toHaveBeenCalled()
    expect(client.listChangeRisks).not.toHaveBeenCalled()
    expect(client.getExecutiveRepository).not.toHaveBeenCalled()
  })
})
