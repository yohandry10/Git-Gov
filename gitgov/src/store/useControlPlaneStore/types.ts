import type { CombinedEvent, ServerStats } from '@/lib/types'
import type {
  EnterpriseAdoptionProfile,
  EnterpriseOnboardingChecklistTracking,
} from '@/components/control_plane/dashboard-helpers'

export interface ServerConfig {
  url: string
  api_key?: string
}

export interface DailyActivityPoint {
  day: string
  commits: number
  pushes: number
}

export interface CommitPipelineRun {
  pipeline_event_id: string
  pipeline_id: string
  job_name: string
  status: string
  branch?: string | null
  repo_full_name?: string | null
  duration_ms?: number | null
  triggered_by?: string | null
  ingested_at: number
}

export interface CommitPipelineCorrelation {
  commit_event_id: string
  commit_sha: string
  commit_message?: string | null
  commit_created_at: number
  user_login: string
  branch?: string | null
  repo_name?: string | null
  pipeline?: CommitPipelineRun | null
}

export interface PrMergeEvidenceEntry {
  id: string
  org_id?: string | null
  org_name?: string | null
  repo_id?: string | null
  repo_full_name?: string | null
  delivery_id: string
  pr_number: number
  pr_title?: string | null
  author_login?: string | null
  merged_by_login?: string | null
  approvers: string[]
  approvals_count: number
  head_sha?: string | null
  merge_commit_sha?: string | null
  base_branch?: string | null
  created_at: number
}

export type TicketCoverageItem = Record<string, unknown>

export interface TicketCoverageStats {
  org: string
  period: string
  total_commits: number
  commits_with_ticket: number
  coverage_percentage: number
  commits_without_ticket: TicketCoverageItem[]
  tickets_without_commits: TicketCoverageItem[]
}

export interface JiraCorrelateResponse {
  scanned_commits: number
  correlations_created: number
  correlated_tickets: string[]
}

export interface JiraTicketDetail {
  id: string
  org_id?: string | null
  ticket_id: string
  ticket_url?: string | null
  title?: string | null
  status?: string | null
  assignee?: string | null
  reporter?: string | null
  priority?: string | null
  ticket_type?: string | null
  related_commits: string[]
  related_prs: string[]
  related_branches: string[]
  created_at?: number | null
  updated_at?: number | null
  ingested_at: number
}

export interface JiraTicketDetailResponse {
  found: boolean
  ticket?: JiraTicketDetail | null
}

export interface EvidencePacketCompleteness {
  ticket_found: boolean
  commits: number
  pull_requests: number
  pipelines: number
  quality_gates: number
  missing: string[]
}

export interface EvidencePacketReconstructionFilters {
  org_name?: string | null
  repo_full_name?: string | null
  branch?: string | null
  target_sha?: string | null
  ticket_id: string
  hours: number
}

export interface EvidencePacketReconstructionSources {
  commit_correlations: number
  client_events: number
  pull_request_merge_commits: number
  pull_request_merges: number
  pipeline_events: number
  quality_gate_pipeline_events: number
  legacy_pipeline_scope_fallbacks: number
}

export interface EvidencePacketReconstruction {
  filters: EvidencePacketReconstructionFilters
  sources: EvidencePacketReconstructionSources
  warnings: string[]
}

export interface EvidencePacket {
  packet_type: string
  subject: string
  generated_at: number
  org_name?: string | null
  repo_full_name?: string | null
  branch?: string | null
  target_sha?: string | null
  release_id?: string | null
  environment?: string | null
  period: string
  ticket?: JiraTicketDetail | null
  commits: Record<string, unknown>[]
  pull_requests: PrMergeEvidenceEntry[]
  pipelines: CommitPipelineRun[]
  quality_gates: CommitPipelineRun[]
  reconstruction?: EvidencePacketReconstruction
  completeness: EvidencePacketCompleteness
  content_hash: string
}

export interface EvidencePacketResponse {
  found: boolean
  packet?: EvidencePacket | null
}

export interface EnterpriseAdoptionProfileRecord {
  org_id: string
  profile: EnterpriseAdoptionProfile
  updated_by: string
  created_at: number
  updated_at: number
}

export interface EnterpriseAdoptionProfileResponse {
  found: boolean
  profile?: EnterpriseAdoptionProfileRecord | null
}

export interface EnterpriseOnboardingChecklistTrackingRecord {
  org_id: string
  tracking: EnterpriseOnboardingChecklistTracking
  updated_by: string
  created_at: number
  updated_at: number
}

export interface EnterpriseOnboardingChecklistTrackingResponse {
  found: boolean
  tracking?: EnterpriseOnboardingChecklistTrackingRecord | null
}

export type EnterpriseReleaseApprovalDecision = 'approved' | 'rejected' | 'accepted-risk'
export type EnterpriseReleaseApprovalRiskSeverity = 'none' | 'low' | 'medium' | 'high' | 'critical'

export interface EnterpriseReleaseApprovalRecord {
  id: string
  org_id: string
  release_id: string
  repository_full_name: string
  branch?: string | null
  target_sha?: string | null
  environment: string
  decision: EnterpriseReleaseApprovalDecision
  approver: string
  ticket_id?: string | null
  evidence_packet_hash?: string | null
  evidence_packet_uri?: string | null
  evidence_summary: Record<string, unknown>
  risk_severity: EnterpriseReleaseApprovalRiskSeverity
  risk_acceptance_reason?: string | null
  expires_at?: number | null
  approval_hash: string
  created_by: string
  created_at: number
}

export interface EnterpriseReleaseApprovalListResponse {
  items: EnterpriseReleaseApprovalRecord[]
  total: number
  limit: number
  offset: number
}

export interface EnterpriseReleaseApprovalQuery {
  org_name?: string | null
  repository_full_name?: string | null
  branch?: string | null
  target_sha?: string | null
  release_id?: string | null
  environment?: string | null
  decision?: EnterpriseReleaseApprovalDecision | '' | null
  evidence_packet_hash?: string | null
  limit?: number | null
  offset?: number | null
}

export type EnterpriseReleaseGovernanceEvaluationStatus =
  | 'recorded'
  | 'advisory-warning'
  | 'approved'
  | 'would-block'
  | 'blocked'
  | string

export interface EnterpriseReleaseGovernanceEvaluationQuery {
  org_name?: string | null
  repository_full_name: string
  branch?: string | null
  target_sha?: string | null
  release_id: string
  environment: string
  evidence_packet_hash?: string | null
}

export interface EnterpriseReleaseGovernanceQuorumRuleSummary {
  role: string
  required: number
  observed: number
  satisfied: boolean
}

export interface EnterpriseReleaseGovernancePolicySummary {
  mode: string
  environment: string
  approval_required: boolean
  enforcement: string
  policy_applies: boolean
  quorum_enabled: boolean
  quorum_rules: EnterpriseReleaseGovernanceQuorumRuleSummary[]
}

export interface EnterpriseReleaseGovernanceApprovalSummary {
  id: string
  decision: string
  approver: string
  approver_role?: string | null
  risk_severity: string
  evidence_packet_hash?: string | null
  expires_at?: number | null
  created_at: number
  counts_toward_policy: boolean
}

export interface EnterpriseReleaseGovernanceEvaluationResponse {
  status: EnterpriseReleaseGovernanceEvaluationStatus
  policy_satisfied: boolean
  blocking: boolean
  would_block: boolean
  valid_approval_count: number
  required_approval_count: number
  policy: EnterpriseReleaseGovernancePolicySummary
  approvals: EnterpriseReleaseGovernanceApprovalSummary[]
  issues: string[]
  next_steps: string[]
}

export interface CreateEnterpriseReleaseApprovalRequest {
  org_name?: string | null
  release_id: string
  repository_full_name: string
  branch?: string | null
  target_sha?: string | null
  environment: string
  decision: EnterpriseReleaseApprovalDecision
  approver: string
  ticket_id?: string | null
  evidence_packet_hash?: string | null
  evidence_packet_uri?: string | null
  evidence_summary?: Record<string, unknown>
  risk_severity?: EnterpriseReleaseApprovalRiskSeverity | null
  risk_acceptance_reason?: string | null
  expires_at?: number | null
}

export interface JiraCoverageFilters {
  hours: number
  repo_full_name: string
  branch: string
}

export interface ActiveDev7dEntry {
  user_login: string
  events: number
  last_seen: number
  suspicious_test_data: boolean
  sample_repo_empty_count: number
}

export interface ApiKeyInfo {
  id: string
  client_id: string
  role: string
  org_id: string | null
  org_name?: string | null
  created_at: number
  last_used: number | null
  is_active: boolean
}

export interface MeResponse {
  client_id: string
  role: string
  org_id: string | null
  org_name?: string | null
}

export interface PendingControlPlaneSession {
  client_id: string
  role: string
  org_id: string | null
  org_name: string | null
}

export interface OrgSummary {
  id: string
  github_id?: number | null
  login: string
  name?: string | null
  avatar_url?: string | null
  created_at: number
}

export interface RevokeApiKeyResponse {
  success: boolean
  message: string
}

export interface OrgUser {
  id: string
  org_id: string
  login: string
  display_name: string | null
  email: string | null
  role: string
  status: string
  created_by: string | null
  updated_by: string | null
  created_at: number
  updated_at: number
}

export interface OrgInvitation {
  id: string
  org_id: string
  invite_email: string | null
  invite_login: string | null
  role: string
  status: string
  invited_by: string
  accepted_by: string | null
  accepted_at: number | null
  revoked_by: string | null
  revoked_at: number | null
  expires_at: number
  created_at: number
  updated_at: number
}

export interface CreateOrgResponse {
  org_id: string
  login: string
  created: boolean
}

export interface CreateOrgUserResponse {
  user: OrgUser
  created: boolean
}

export interface OrgUsersResponse {
  entries: OrgUser[]
  total: number
}

export interface CreateOrgInvitationResponse {
  invitation: OrgInvitation
  invite_token: string
}

export interface OrgInvitationsResponse {
  entries: OrgInvitation[]
  total: number
}

export interface AcceptOrgInvitationResponse {
  invitation: OrgInvitation
  client_id: string
  role: string
  org_id: string
  api_key: string
}

export interface IssueOrgUserApiKeyResponse {
  api_key: string | null
  client_id: string
  error: string | null
}

export interface TeamRepoSummary {
  repo_name: string
  events: number
  commits: number
  pushes: number
  blocked_pushes: number
  last_seen: number
}

export interface TeamDeveloperOverview {
  login: string
  display_name: string | null
  email: string | null
  role: string
  status: string
  last_seen: number | null
  total_events: number
  commits: number
  pushes: number
  blocked_pushes: number
  repos_active_count: number
  repos: TeamRepoSummary[]
}

export interface TeamRepoOverview {
  repo_name: string
  developers_active: number
  total_events: number
  commits: number
  pushes: number
  blocked_pushes: number
  last_seen: number
}

export interface TeamOverviewResponse {
  entries: TeamDeveloperOverview[]
  total: number
}

export interface TeamReposResponse {
  entries: TeamRepoOverview[]
  total: number
}

export interface ExportResponse {
  id: string
  export_type: string
  record_count: number
  content_hash: string
  data?: unknown
  created_at: number
}

export interface ExportLogEntry {
  id: string
  org_id: string | null
  exported_by: string
  export_type: string
  date_range_start: number | null
  date_range_end: number | null
  filters: unknown
  record_count: number
  content_hash: string | null
  file_path: string | null
  created_at: number
}

// ── Chat interfaces ──────────────────────────────────────────────────────────

export interface ChatAskResponse {
  status: 'ok' | 'insufficient_data' | 'feature_not_available' | 'error'
  answer: string
  missing_capability?: string | null
  can_report_feature: boolean
  data_refs: string[]
  sources?: string[]
  entities_detected?: string[]
  time_range_used?: string | null
  actions_recommended?: string[]
  confidence?: number | null
  trace_id?: string | null
}

export interface ChatMessage {
  id: string
  role: 'user' | 'assistant'
  content: string
  response?: ChatAskResponse
  timestamp: number
}

export interface GovernanceCopilotAskRequest {
  question: string
  org_name?: string | null
  repository_full_name?: string | null
  branch?: string | null
  ticket_id?: string | null
  release_id?: string | null
  environment?: string | null
  hours?: number | null
}

export interface GovernanceCopilotCitation {
  id: string
  label: string
  endpoint: string
  status: 'ok' | 'missing' | 'error' | 'skipped' | string
  httpStatus?: number | null
}

export interface GovernanceCopilotSource extends GovernanceCopilotCitation {
  summary?: unknown
}

export interface GovernanceCopilotResponse {
  success: boolean
  mode?: 'ai' | 'fallback' | string | null
  model?: string | null
  answer: string
  citations: GovernanceCopilotCitation[]
  sources: GovernanceCopilotSource[]
  warnings: string[]
}

export interface ChatSession {
  id: string
  title: string
  created_at: number
  updated_at: number
  messages: ChatMessage[]
}

export interface PolicyResponseData {
  version: string
  checksum: string
  config: import('@/lib/types').GitGovConfig
  source: import('@/lib/types').PolicySourceMetadata
  updated_at: number
}

export interface PolicyHistoryEntry {
  id: string
  repo_id: string
  config: import('@/lib/types').GitGovConfig
  checksum: string
  source?: import('@/lib/types').PolicySourceMetadata
  changed_by: string
  change_type: string
  previous_checksum: string | null
  created_at: number
}

export interface ControlPlaneState {
  serverConfig: ServerConfig | null
  serverStats: ServerStats | null
  serverLogs: CombinedEvent[]
  activeDevs7d: ActiveDev7dEntry[]
  activeDevs7dUpdatedAt: number | null
  logsPage: number
  logsPageSize: number
  jenkinsCorrelations: CommitPipelineCorrelation[]
  prMergeEvidence: PrMergeEvidenceEntry[]
  dailyActivity: DailyActivityPoint[]
  ticketCoverage: TicketCoverageStats | null
  jiraCoverageFilters: JiraCoverageFilters
  jiraTicketDetails: Record<string, JiraTicketDetail | null>
  jiraTicketDetailFetchedAt: Record<string, number>
  jiraTicketDetailLoading: Record<string, boolean>
  evidencePacket: EvidencePacket | null
  evidencePacketTicketId: string
  isEvidencePacketLoading: boolean
  enterpriseAdoptionProfile: EnterpriseAdoptionProfile | null
  enterpriseAdoptionProfileUpdatedAt: number | null
  isEnterpriseAdoptionProfileLoading: boolean
  isEnterpriseAdoptionProfileSaving: boolean
  enterpriseAdoptionProfileError: string | null
  enterpriseOnboardingChecklistTracking: EnterpriseOnboardingChecklistTracking | null
  enterpriseOnboardingChecklistTrackingUpdatedAt: number | null
  isEnterpriseOnboardingChecklistTrackingLoading: boolean
  isEnterpriseOnboardingChecklistTrackingSaving: boolean
  enterpriseOnboardingChecklistTrackingError: string | null
  releaseApprovals: EnterpriseReleaseApprovalRecord[]
  releaseApprovalsTotal: number
  releaseApprovalsFilters: EnterpriseReleaseApprovalQuery
  releaseGovernanceEvaluation: EnterpriseReleaseGovernanceEvaluationResponse | null
  isReleaseGovernanceEvaluating: boolean
  isReleaseApprovalsLoading: boolean
  isReleaseApprovalSubmitting: boolean
  releaseApprovalError: string | null
  userRole: string | null
  userClientId: string | null
  userOrgId: string | null
  controlPlaneAuthConfirmed: boolean
  pendingControlPlaneSession: PendingControlPlaneSession | null
  selectedOrgName: string
  selectedOrgValidated: boolean
  availableOrgs: OrgSummary[]
  isLoadingOrgs: boolean
  orgUsers: OrgUser[]
  orgUsersTotal: number
  orgInvitations: OrgInvitation[]
  orgInvitationsTotal: number
  lastGeneratedInviteToken: string | null
  teamOverview: TeamDeveloperOverview[]
  teamOverviewTotal: number
  teamRepos: TeamRepoOverview[]
  teamReposTotal: number
  teamWindowDays: number
  teamStatusFilter: '' | 'active' | 'disabled'
  apiKeys: ApiKeyInfo[]
  isLoadingApiKeys: boolean
  exportLogs: ExportLogEntry[]
  connectionStatus: 'connected' | 'disconnected' | 'maintenance' | 'checking'
  maintenanceDetectedAt: number | null
  isConnected: boolean
  isLoading: boolean
  isRefreshingDashboard: boolean
  error: string | null
  chatSessions: ChatSession[]
  activeChatSessionId: string | null
  chatMessages: ChatMessage[]
  isChatLoading: boolean
  governanceCopilotResponse: GovernanceCopilotResponse | null
  isGovernanceCopilotLoading: boolean
  governanceCopilotError: string | null
  displayTimezone: string
  policyData: PolicyResponseData | null
  policyHistory: PolicyHistoryEntry[]
  isPolicyLoading: boolean
  isPolicySaving: boolean
  policyError: string | null
  sseConnected: boolean
}

export interface ControlPlaneActions {
  initFromEnv: () => Promise<void>
  setServerConfig: (config: ServerConfig) => void
  applyEnvApiKey: () => Promise<boolean>
  applyApiKey: (apiKey: string, url?: string) => Promise<boolean>
  markControlPlaneSessionValidated: (session: PendingControlPlaneSession) => void
  confirmControlPlaneSession: () => void
  resetControlPlaneAuthGate: () => void
  checkConnection: (options?: { background?: boolean }) => Promise<void>
  refreshDashboardData: (params?: { logLimit?: number; forceHeavy?: boolean }) => Promise<void>
  loadStats: () => Promise<void>
  loadDailyActivity: (days?: number) => Promise<void>
  loadLogs: (limit?: number, offset?: number) => Promise<void>
  loadLogsIncremental: (limit?: number) => Promise<void>
  loadActiveDevs7d: () => Promise<void>
  setLogsPage: (page: number) => void
  loadJenkinsCorrelations: (limit?: number) => Promise<void>
  loadPrMergeEvidence: (limit?: number) => Promise<void>
  loadTicketCoverage: (params?: { hours?: number; repo_full_name?: string; branch?: string; org_name?: string }) => Promise<void>
  applyTicketCoverageFilters: (filters: Partial<JiraCoverageFilters>) => Promise<void>
  correlateJiraTickets: (params?: { hours?: number; limit?: number; repo_full_name?: string; org_name?: string }) => Promise<JiraCorrelateResponse | null>
  loadJiraTicketDetail: (ticketId: string) => Promise<JiraTicketDetail | null>
  loadTicketEvidencePacket: (ticketId: string, params?: { hours?: number; repo_full_name?: string; branch?: string; target_sha?: string; release_id?: string; environment?: string; org_name?: string }) => Promise<EvidencePacket | null>
  loadEnterpriseAdoptionProfile: (orgName?: string) => Promise<EnterpriseAdoptionProfile | null>
  saveEnterpriseAdoptionProfile: (profile: EnterpriseAdoptionProfile, orgName?: string) => Promise<boolean>
  loadEnterpriseOnboardingChecklistTracking: (orgName?: string) => Promise<EnterpriseOnboardingChecklistTracking | null>
  saveEnterpriseOnboardingChecklistTracking: (tracking: EnterpriseOnboardingChecklistTracking, orgName?: string) => Promise<boolean>
  loadEnterpriseReleaseApprovals: (query?: EnterpriseReleaseApprovalQuery) => Promise<EnterpriseReleaseApprovalListResponse | null>
  evaluateEnterpriseReleaseGovernance: (query: EnterpriseReleaseGovernanceEvaluationQuery) => Promise<EnterpriseReleaseGovernanceEvaluationResponse | null>
  createEnterpriseReleaseApproval: (payload: CreateEnterpriseReleaseApprovalRequest) => Promise<EnterpriseReleaseApprovalRecord | null>
  loadMe: () => Promise<boolean>
  loadOrgs: () => Promise<OrgSummary[]>
  validateOrgName: (orgName: string) => Promise<OrgSummary | null>
  activateOrgName: (orgName: string) => Promise<OrgSummary | null>
  createOrg: (payload: { login: string; name?: string }) => Promise<CreateOrgResponse | null>
  setSelectedOrgName: (orgName: string) => void
  loadOrgUsers: (params?: { orgName?: string; status?: string; limit?: number; offset?: number }) => Promise<void>
  upsertOrgUser: (payload: {
    orgName?: string
    login: string
    email?: string
    displayName?: string
    role?: string
    status?: string
  }) => Promise<OrgUser | null>
  updateOrgUserStatus: (userId: string, status: 'active' | 'disabled') => Promise<OrgUser | null>
  issueApiKeyForOrgUser: (userId: string) => Promise<IssueOrgUserApiKeyResponse | null>
  loadOrgInvitations: (params?: { orgName?: string; status?: string; limit?: number; offset?: number }) => Promise<void>
  createOrgInvitation: (payload: {
    orgName?: string
    inviteEmail?: string
    inviteLogin?: string
    role?: string
    expiresInDays?: number
  }) => Promise<CreateOrgInvitationResponse | null>
  resendOrgInvitation: (invitationId: string, expiresInDays?: number) => Promise<CreateOrgInvitationResponse | null>
  revokeOrgInvitation: (invitationId: string) => Promise<boolean>
  previewOrgInvitation: (token: string) => Promise<OrgInvitation | null>
  acceptOrgInvitation: (payload: { token: string; login?: string }) => Promise<AcceptOrgInvitationResponse | null>
  setTeamFilters: (filters: { days?: number; status?: '' | 'active' | 'disabled' }) => void
  loadTeamOverview: (params?: { orgName?: string; days?: number; status?: '' | 'active' | 'disabled'; limit?: number; offset?: number; append?: boolean }) => Promise<void>
  loadTeamRepos: (params?: { orgName?: string; days?: number; limit?: number; offset?: number; append?: boolean }) => Promise<void>
  refreshForCurrentRole: (options?: { forceHeavy?: boolean }) => Promise<void>
  loadApiKeys: (params?: { orgName?: string | null; global?: boolean }) => Promise<void>
  revokeApiKey: (keyId: string, params?: { orgName?: string | null; global?: boolean }) => Promise<boolean>
  exportAuditData: (params: { exportType?: string; startDate?: number; endDate?: number; orgName?: string }) => Promise<ExportResponse | null>
  loadExportLogs: () => Promise<void>
  clearError: () => void
  disconnect: () => void
  chatAsk: (question: string, orgName?: string) => Promise<ChatAskResponse | null>
  askGovernanceCopilot: (request: GovernanceCopilotAskRequest) => Promise<GovernanceCopilotResponse | null>
  reportFeature: (question: string, missingCapability?: string) => Promise<boolean>
  clearChatMessages: () => void
  createChatSession: () => void
  setActiveChatSession: (sessionId: string) => void
  closeChatSession: (sessionId: string) => void
  refreshChatMessagesForActiveUser: () => void
  setDisplayTimezone: (tz: string) => void
  loadPolicy: (repoName: string) => Promise<void>
  savePolicy: (repoName: string, config: import('@/lib/types').GitGovConfig) => Promise<boolean>
  loadPolicyHistory: (repoName: string) => Promise<void>
  connectSse: () => Promise<void>
  disconnectSse: () => void
}
