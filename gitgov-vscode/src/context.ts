import { GitGovReadOnlyClient } from './api'
import type { GitContext, GitGovConnectionConfig, GovernanceSnapshot } from './types'

export interface LoadGovernanceContextOptions {
  git: GitContext
  config: GitGovConnectionConfig
  apiKey: string | undefined
  client?: GitGovReadOnlyClient
}

export async function loadGovernanceContext(options: LoadGovernanceContextOptions): Promise<GovernanceSnapshot> {
  const configured = Boolean(options.config.apiUrl.trim() && options.config.orgName.trim() && options.apiKey?.trim())

  if (!options.git.isGitRepository) {
    return {
      git: options.git,
      configured,
      latestGate: null,
      latestRisk: null,
      executiveRepository: null,
      error: options.git.error ?? 'No Git repository detected.',
    }
  }

  if (!options.git.repositoryFullName) {
    return {
      git: options.git,
      configured,
      latestGate: null,
      latestRisk: null,
      executiveRepository: null,
      error: 'GitGov needs a GitHub origin remote to map this workspace to governance evidence.',
    }
  }

  if (!configured) {
    return {
      git: options.git,
      configured,
      latestGate: null,
      latestRisk: null,
      executiveRepository: null,
      error: 'Configure GitGov API URL, org, and read-only API key before refreshing governance context.',
    }
  }

  const client = options.client ?? new GitGovReadOnlyClient({
    config: options.config,
    apiKey: options.apiKey ?? '',
  })

  try {
    const [gates, risks, executiveRepository] = await Promise.all([
      client.listDeploymentGates(options.git.repositoryFullName, options.git.branch),
      client.listChangeRisks(options.git.repositoryFullName, options.git.branch),
      client.getExecutiveRepository(options.git.repositoryFullName),
    ])

    return {
      git: options.git,
      configured,
      latestGate: gates.items[0] ?? null,
      latestRisk: risks.items[0] ?? null,
      executiveRepository,
      error: null,
    }
  } catch (error) {
    return {
      git: options.git,
      configured,
      latestGate: null,
      latestRisk: null,
      executiveRepository: null,
      error: error instanceof Error ? error.message : 'GitGov governance context is unavailable.',
    }
  }
}
