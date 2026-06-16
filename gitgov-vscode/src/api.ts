import type {
  ChangeRiskEvaluationListResponse,
  DeploymentGateAuthorizationListResponse,
  ExecutiveRepository,
  GitGovConnectionConfig,
  MultiRepoExecutiveGovernanceResponse,
} from './types'

export const READ_ONLY_ENDPOINTS = [
  '/deployment-gates/authorizations',
  '/change-risk/evaluations',
  '/executive/repositories',
] as const

export interface GitGovReadOnlyClientOptions {
  config: GitGovConnectionConfig
  apiKey: string
  fetchImpl?: typeof fetch
}

export class GitGovReadOnlyError extends Error {
  constructor(
    message: string,
    readonly status?: number,
  ) {
    super(message)
    this.name = 'GitGovReadOnlyError'
  }
}

function buildUrl(apiUrl: string, path: (typeof READ_ONLY_ENDPOINTS)[number], params: Record<string, string | number | null | undefined>): string {
  const url = new URL(`${apiUrl.replace(/\/+$/, '')}${path}`)
  for (const [key, value] of Object.entries(params)) {
    if (value !== null && value !== undefined && String(value).trim()) {
      url.searchParams.set(key, String(value))
    }
  }
  return url.toString()
}

async function readJson<T>(response: Response): Promise<T> {
  try {
    return (await response.json()) as T
  } catch {
    throw new GitGovReadOnlyError('GitGov returned an unreadable response.', response.status)
  }
}

export class GitGovReadOnlyClient {
  private readonly fetchImpl: typeof fetch
  private readonly apiUrl: string
  private readonly orgName: string
  private readonly apiKey: string

  constructor(options: GitGovReadOnlyClientOptions) {
    this.fetchImpl = options.fetchImpl ?? fetch
    this.apiUrl = options.config.apiUrl.replace(/\/+$/, '')
    this.orgName = options.config.orgName.trim()
    this.apiKey = options.apiKey.trim()
  }

  async listDeploymentGates(repositoryFullName: string, branch: string | null): Promise<DeploymentGateAuthorizationListResponse> {
    return this.get('/deployment-gates/authorizations', {
      org_name: this.orgName,
      repository_full_name: repositoryFullName,
      branch,
      limit: 1,
      offset: 0,
    })
  }

  async listChangeRisks(repositoryFullName: string, branch: string | null): Promise<ChangeRiskEvaluationListResponse> {
    return this.get('/change-risk/evaluations', {
      org_name: this.orgName,
      repository_full_name: repositoryFullName,
      branch,
      limit: 1,
      offset: 0,
    })
  }

  async getExecutiveRepository(repositoryFullName: string): Promise<ExecutiveRepository | null> {
    const response = await this.get<MultiRepoExecutiveGovernanceResponse>('/executive/repositories', {
      org_name: this.orgName,
      repository: repositoryFullName,
      limit: 1,
      offset: 0,
    })
    return response.repositories[0] ?? null
  }

  private async get<T>(
    path: (typeof READ_ONLY_ENDPOINTS)[number],
    params: Record<string, string | number | null | undefined>,
  ): Promise<T> {
    const response = await this.fetchImpl(buildUrl(this.apiUrl, path, params), {
      method: 'GET',
      headers: {
        Authorization: `Bearer ${this.apiKey}`,
        Accept: 'application/json',
      },
    })

    if (!response.ok) {
      if (response.status === 401 || response.status === 403) {
        throw new GitGovReadOnlyError('GitGov denied this read-only request. Check org scope and key permissions.', response.status)
      }
      throw new GitGovReadOnlyError(`GitGov read-only request failed with HTTP ${response.status}.`, response.status)
    }

    return readJson<T>(response)
  }
}
