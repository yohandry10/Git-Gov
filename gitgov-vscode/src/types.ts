export interface GitGovConnectionConfig {
  apiUrl: string
  orgName: string
}

export interface StoredGitGovConnection extends GitGovConnectionConfig {
  hasApiKey: boolean
}

export interface GitGovSecretStore {
  get(key: string): Thenable<string | undefined> | Promise<string | undefined>
  store(key: string, value: string): Thenable<void> | Promise<void>
  delete(key: string): Thenable<void> | Promise<void>
}

export interface GitContext {
  isGitRepository: boolean
  repositoryFullName: string | null
  branch: string | null
  rootPath: string | null
  error: string | null
}

export interface DeploymentGateAuthorization {
  authorization_id?: string
  decision?: string
  repository_full_name?: string
  branch?: string
  environment?: string
  created_at?: string
}

export interface DeploymentGateAuthorizationListResponse {
  items: DeploymentGateAuthorization[]
}

export interface ChangeRiskEvaluation {
  evaluation_id?: string
  risk_level?: string
  review_status?: string
  repository_full_name?: string
  branch?: string
  created_at?: string
}

export interface ChangeRiskEvaluationListResponse {
  items: ChangeRiskEvaluation[]
}

export interface ExecutiveRepository {
  repository_full_name?: string
  posture?: string
  gate_count?: number
  change_risk_count?: number
  latest_gate_decision?: string | null
  latest_risk_level?: string | null
  latest_review_status?: string | null
}

export interface MultiRepoExecutiveGovernanceResponse {
  repositories: ExecutiveRepository[]
}

export interface GovernanceSnapshot {
  git: GitContext
  configured: boolean
  latestGate: DeploymentGateAuthorization | null
  latestRisk: ChangeRiskEvaluation | null
  executiveRepository: ExecutiveRepository | null
  error: string | null
}
