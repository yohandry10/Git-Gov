import { parseCommandError, tauriInvoke } from '@/lib/tauri'
import { useAuthStore } from '@/store/useAuthStore'
import type {
  AcceptOrgInvitationResponse,
  ApiKeyInfo,
  ControlPlaneActions,
  CreateOrgInvitationResponse,
  CreateOrgResponse,
  CreateOrgUserResponse,
  IssueOrgUserApiKeyResponse,
  MeResponse,
  OrgInvitation,
  OrgInvitationsResponse,
  OrgSummary,
  OrgUser,
  OrgUsersResponse,
  RevokeApiKeyResponse,
  TeamOverviewResponse,
  TeamReposResponse,
} from '../types'
import type { ControlPlaneGet, ControlPlaneSet } from '../store-types'
import { DEFAULT_GOVERNANCE_LOG_WINDOW } from '../constants'
import { controlPlaneStoreRuntime } from '../runtime'
import {
  isControlPlaneIdentityCompatible,
  isUnauthorizedError,
  persistSecureControlPlaneApiKey,
  persistSelectedOrgName,
  persistServerConfig,
  readStoredSelectedOrgName,
  readStoredJiraCoverageFilters,
  syncOutboxServerConfig,
} from '../helpers'

type OrganizationActionKeys =
  | 'loadMe'
  | 'loadOrgs'
  | 'validateOrgName'
  | 'activateOrgName'
  | 'createOrg'
  | 'setSelectedOrgName'
  | 'loadOrgUsers'
  | 'upsertOrgUser'
  | 'updateOrgUserStatus'
  | 'issueApiKeyForOrgUser'
  | 'loadOrgInvitations'
  | 'createOrgInvitation'
  | 'resendOrgInvitation'
  | 'revokeOrgInvitation'
  | 'previewOrgInvitation'
  | 'acceptOrgInvitation'
  | 'setTeamFilters'
  | 'loadTeamOverview'
  | 'loadTeamRepos'
  | 'refreshForCurrentRole'
  | 'loadApiKeys'
  | 'revokeApiKey'
  | 'clearError'
  | 'disconnect'

export function createOrganizationActions(
  set: ControlPlaneSet,
  get: ControlPlaneGet,
): Pick<ControlPlaneActions, OrganizationActionKeys> {
  return {
  loadMe: async () => {
    const { serverConfig } = get()
    if (!serverConfig) return false
    try {
      const me = await tauriInvoke<MeResponse>('cmd_server_get_me', { config: serverConfig })
      const githubLogin = useAuthStore.getState().user?.login ?? null
      if (!isControlPlaneIdentityCompatible(me.client_id, githubLogin, me.role, me.principal_type)) {
        set({
          userRole: null,
          userClientId: null,
          userOrgId: null,
          selectedOrgValidated: false,
          controlPlaneAuthConfirmed: true,
          pendingControlPlaneSession: null,
          error: `La API key autenticó como '${me.client_id}', pero tu sesión GitHub es '${githubLogin ?? 'desconocida'}'.`,
        })
        return false
      }
      const meOrgName = me.org_name?.trim() || ''
      const storedOrgName = !meOrgName && me.role === 'Admin' && !me.org_id
        ? readStoredSelectedOrgName(serverConfig)
        : ''
      set({
        userRole: me.role,
        userClientId: me.client_id,
        userOrgId: me.org_id ?? null,
        selectedOrgName: meOrgName || storedOrgName || get().selectedOrgName,
        selectedOrgValidated: Boolean(meOrgName),
        error: null,
      })
      if (meOrgName) {
        persistSelectedOrgName(serverConfig, meOrgName)
      } else if (storedOrgName) {
        await get().activateOrgName(storedOrgName)
      }
      return true
    } catch (e) {
      const meError = parseCommandError(String(e)).message
      set({
        userRole: null,
        userClientId: null,
        userOrgId: null,
        selectedOrgValidated: false,
        controlPlaneAuthConfirmed: true,
        pendingControlPlaneSession: null,
        error: isUnauthorizedError(meError)
          ? 'API key inválida o expirada para Control Plane. Usa una key válida para tu rol.'
          : meError,
      })
      return false
    }
  },

  loadOrgs: async () => {
    const { serverConfig } = get()
    if (!serverConfig) return []
    set({ isLoadingOrgs: true })
    try {
      const orgs = await tauriInvoke<OrgSummary[]>('cmd_server_list_orgs', { config: serverConfig })
      set({ availableOrgs: orgs, error: null })
      return orgs
    } catch (e) {
      set({ error: parseCommandError(String(e)).message })
      return []
    } finally {
      set({ isLoadingOrgs: false })
    }
  },

  validateOrgName: async (orgName) => {
    const { serverConfig } = get()
    const login = orgName.trim()
    if (!serverConfig || !login) {
      set({ error: 'Selecciona un workspace GitGov para continuar.' })
      return null
    }
    try {
      const org = await tauriInvoke<OrgSummary>('cmd_server_get_org', {
        config: serverConfig,
        login,
      })
      set((state) => ({
        availableOrgs: state.availableOrgs.some((item) => item.login === org.login)
          ? state.availableOrgs.map((item) => item.login === org.login ? org : item)
          : [...state.availableOrgs, org],
        error: null,
      }))
      return org
    } catch (e) {
      const message = parseCommandError(String(e)).message
      const friendly = message.includes('Organization not found')
        ? `No existe un workspace GitGov llamado "${login}".`
        : message.includes('outside API key scope')
          ? `La API key actual no tiene permiso para el workspace "${login}".`
          : message
      set({ error: friendly, selectedOrgValidated: false })
      return null
    }
  },

  activateOrgName: async (orgName) => {
    const { serverConfig } = get()
    const org = await get().validateOrgName(orgName)
    if (!org) {
      set({ selectedOrgName: orgName.trim(), selectedOrgValidated: false })
      return null
    }
    persistSelectedOrgName(serverConfig, org.login)
    set({ selectedOrgName: org.login, selectedOrgValidated: true, error: null })
    return org
  },

  createOrg: async (payload) => {
    const { serverConfig } = get()
    if (!serverConfig) return null
    try {
      const response = await tauriInvoke<CreateOrgResponse>('cmd_server_create_org', {
        config: serverConfig,
        payload: {
          login: payload.login.trim(),
          name: payload.name?.trim() || null,
        },
      })
      if (response.login) {
        persistSelectedOrgName(serverConfig, response.login)
        set({ selectedOrgName: response.login, selectedOrgValidated: true })
      }
      return response
    } catch (e) {
      set({ error: parseCommandError(String(e)).message })
      return null
    }
  },

  setSelectedOrgName: (orgName) => {
    set({ selectedOrgName: orgName.trim(), selectedOrgValidated: false })
  },

  loadOrgUsers: async (params) => {
    const { serverConfig, selectedOrgName } = get()
    if (!serverConfig) return
    const orgName = params?.orgName?.trim() || selectedOrgName.trim() || undefined
    try {
      const response = await tauriInvoke<OrgUsersResponse>('cmd_server_list_org_users', {
        config: serverConfig,
        orgName,
        status: params?.status ?? null,
        limit: params?.limit ?? 50,
        offset: params?.offset ?? 0,
      })
      set({ orgUsers: response.entries, orgUsersTotal: response.total })
    } catch (e) {
      set({ error: parseCommandError(String(e)).message })
    }
  },

  upsertOrgUser: async (payload) => {
    const { serverConfig, selectedOrgName } = get()
    if (!serverConfig) return null
    const orgName = payload.orgName?.trim() || selectedOrgName.trim() || undefined
    try {
      const response = await tauriInvoke<CreateOrgUserResponse>('cmd_server_create_org_user', {
        config: serverConfig,
        payload: {
          login: payload.login.trim(),
          email: payload.email?.trim() || null,
          display_name: payload.displayName?.trim() || null,
          role: payload.role ?? null,
          status: payload.status ?? null,
          org_name: orgName ?? null,
        },
      })
      await get().loadOrgUsers({ orgName })
      return response.user
    } catch (e) {
      set({ error: parseCommandError(String(e)).message })
      return null
    }
  },

  updateOrgUserStatus: async (userId, status) => {
    const { serverConfig } = get()
    if (!serverConfig) return null
    try {
      const response = await tauriInvoke<OrgUser>('cmd_server_update_org_user_status', {
        config: serverConfig,
        userId,
        status,
      })
      await get().loadOrgUsers()
      return response
    } catch (e) {
      set({ error: parseCommandError(String(e)).message })
      return null
    }
  },

  issueApiKeyForOrgUser: async (userId) => {
    const { serverConfig } = get()
    if (!serverConfig) return null
    try {
      const response = await tauriInvoke<IssueOrgUserApiKeyResponse>('cmd_server_create_api_key_for_org_user', {
        config: serverConfig,
        userId,
      })
      return response
    } catch (e) {
      set({ error: parseCommandError(String(e)).message })
      return null
    }
  },

  loadOrgInvitations: async (params) => {
    const { serverConfig, selectedOrgName } = get()
    if (!serverConfig) return
    const orgName = params?.orgName?.trim() || selectedOrgName.trim() || undefined
    try {
      const response = await tauriInvoke<OrgInvitationsResponse>('cmd_server_list_org_invitations', {
        config: serverConfig,
        orgName,
        status: params?.status ?? null,
        limit: params?.limit ?? 50,
        offset: params?.offset ?? 0,
      })
      set({ orgInvitations: response.entries, orgInvitationsTotal: response.total })
    } catch (e) {
      set({ error: parseCommandError(String(e)).message })
    }
  },

  createOrgInvitation: async (payload) => {
    const { serverConfig, selectedOrgName } = get()
    if (!serverConfig) return null
    const orgName = payload.orgName?.trim() || selectedOrgName.trim() || undefined
    try {
      const response = await tauriInvoke<CreateOrgInvitationResponse>('cmd_server_create_org_invitation', {
        config: serverConfig,
        payload: {
          org_name: orgName ?? null,
          invite_email: payload.inviteEmail?.trim() || null,
          invite_login: payload.inviteLogin?.trim() || null,
          role: payload.role ?? null,
          expires_in_days: payload.expiresInDays ?? null,
        },
      })
      set({ lastGeneratedInviteToken: response.invite_token })
      await get().loadOrgInvitations({ orgName })
      await get().loadOrgUsers({ orgName })
      return response
    } catch (e) {
      set({ error: parseCommandError(String(e)).message })
      return null
    }
  },

  resendOrgInvitation: async (invitationId, expiresInDays) => {
    const { serverConfig } = get()
    if (!serverConfig) return null
    try {
      const response = await tauriInvoke<CreateOrgInvitationResponse>('cmd_server_resend_org_invitation', {
        config: serverConfig,
        invitationId,
        expiresInDays: expiresInDays ?? null,
      })
      set({ lastGeneratedInviteToken: response.invite_token })
      await get().loadOrgInvitations()
      return response
    } catch (e) {
      set({ error: parseCommandError(String(e)).message })
      return null
    }
  },

  revokeOrgInvitation: async (invitationId) => {
    const { serverConfig } = get()
    if (!serverConfig) return false
    try {
      await tauriInvoke<OrgInvitation>('cmd_server_revoke_org_invitation', {
        config: serverConfig,
        invitationId,
      })
      await get().loadOrgInvitations()
      return true
    } catch (e) {
      set({ error: parseCommandError(String(e)).message })
      return false
    }
  },

  previewOrgInvitation: async (token) => {
    const { serverConfig } = get()
    if (!serverConfig) return null
    try {
      const invite = await tauriInvoke<OrgInvitation>('cmd_server_preview_org_invitation', {
        config: serverConfig,
        token,
      })
      return invite
    } catch (e) {
      set({ error: parseCommandError(String(e)).message })
      return null
    }
  },

  acceptOrgInvitation: async ({ token, login }) => {
    const { serverConfig } = get()
    if (!serverConfig) return null
    try {
      return await tauriInvoke<AcceptOrgInvitationResponse>('cmd_server_accept_org_invitation', {
        config: serverConfig,
        token,
        login: login?.trim() || null,
      })
    } catch (e) {
      set({ error: parseCommandError(String(e)).message })
      return null
    }
  },

  setTeamFilters: (filters) => {
    set((state) => ({
      teamWindowDays: typeof filters.days === 'number' ? Math.max(1, Math.min(180, Math.floor(filters.days))) : state.teamWindowDays,
      teamStatusFilter: typeof filters.status === 'string' ? filters.status : state.teamStatusFilter,
    }))
  },

  loadTeamOverview: async (params) => {
    const { serverConfig, selectedOrgName, teamWindowDays, teamStatusFilter } = get()
    if (!serverConfig) return
    const orgName = params?.orgName?.trim() || selectedOrgName.trim() || undefined
    const days = typeof params?.days === 'number' ? params.days : teamWindowDays
    const status = params?.status ?? teamStatusFilter
    try {
      const response = await tauriInvoke<TeamOverviewResponse>('cmd_server_get_team_overview', {
        config: serverConfig,
        orgName,
        status: status || null,
        days,
        limit: params?.limit ?? 100,
        offset: params?.offset ?? 0,
      })
      set((state) => {
        if (!params?.append) {
          return {
            teamOverview: response.entries,
            teamOverviewTotal: response.total,
          }
        }

        const merged = [...state.teamOverview]
        const seen = new Set(merged.map((entry) => entry.login))
        for (const entry of response.entries) {
          if (seen.has(entry.login)) continue
          seen.add(entry.login)
          merged.push(entry)
        }
        return {
          teamOverview: merged,
          teamOverviewTotal: response.total,
        }
      })
    } catch (e) {
      set({ error: parseCommandError(String(e)).message })
    }
  },

  loadTeamRepos: async (params) => {
    const { serverConfig, selectedOrgName, teamWindowDays } = get()
    if (!serverConfig) return
    const orgName = params?.orgName?.trim() || selectedOrgName.trim() || undefined
    const days = typeof params?.days === 'number' ? params.days : teamWindowDays
    try {
      const response = await tauriInvoke<TeamReposResponse>('cmd_server_get_team_repos', {
        config: serverConfig,
        orgName,
        days,
        limit: params?.limit ?? 100,
        offset: params?.offset ?? 0,
      })
      set((state) => {
        if (!params?.append) {
          return {
            teamRepos: response.entries,
            teamReposTotal: response.total,
          }
        }

        const merged = [...state.teamRepos]
        const seen = new Set(merged.map((entry) => entry.repo_name))
        for (const entry of response.entries) {
          if (seen.has(entry.repo_name)) continue
          seen.add(entry.repo_name)
          merged.push(entry)
        }
        return {
          teamRepos: merged,
          teamReposTotal: response.total,
        }
      })
    } catch (e) {
      set({ error: parseCommandError(String(e)).message })
    }
  },

  refreshForCurrentRole: async (options) => {
    if (controlPlaneStoreRuntime.refreshForCurrentRoleInFlight) {
      await controlPlaneStoreRuntime.refreshForCurrentRoleInFlight
      if (options?.forceHeavy) {
        await get().refreshForCurrentRole({ forceHeavy: true })
      }
      return
    }

    const run = (async () => {
      const { userRole } = get()
      if (userRole === 'Admin') {
        await get().refreshDashboardData({
          logLimit: DEFAULT_GOVERNANCE_LOG_WINDOW,
          forceHeavy: options?.forceHeavy,
        })
        return
      }

      await get().loadLogsIncremental(DEFAULT_GOVERNANCE_LOG_WINDOW)
    })()

    controlPlaneStoreRuntime.refreshForCurrentRoleInFlight = run
    try {
      await run
    } finally {
      if (controlPlaneStoreRuntime.refreshForCurrentRoleInFlight === run) controlPlaneStoreRuntime.refreshForCurrentRoleInFlight = null
    }
  },

  loadApiKeys: async (params) => {
    const { serverConfig, selectedOrgName, selectedOrgValidated, userRole, userOrgId } = get()
    if (!serverConfig) return
    const isExplicitGlobal = params?.global === true
    if (!isExplicitGlobal && userRole === 'Admin' && !userOrgId && !selectedOrgValidated) {
      set({ error: 'Valida un workspace GitGov antes de consultar API keys de organización.' })
      return
    }
    const orgName = isExplicitGlobal ? undefined : (params?.orgName?.trim() || selectedOrgName.trim() || undefined)
    set({ isLoadingApiKeys: true })
    try {
      const keys = await tauriInvoke<ApiKeyInfo[]>('cmd_server_list_api_keys', {
        config: serverConfig,
        orgName: orgName ?? null,
      })
      set({ apiKeys: keys })
    } catch (e) {
      set({ error: parseCommandError(String(e)).message })
    } finally {
      set({ isLoadingApiKeys: false })
    }
  },

  revokeApiKey: async (keyId, params) => {
    const { serverConfig, selectedOrgName, selectedOrgValidated, userRole, userOrgId } = get()
    if (!serverConfig) return false
    const isExplicitGlobal = params?.global === true
    if (!isExplicitGlobal && userRole === 'Admin' && !userOrgId && !selectedOrgValidated) {
      set({ error: 'Valida un workspace GitGov antes de revocar API keys de organización.' })
      return false
    }
    const orgName = isExplicitGlobal ? undefined : (params?.orgName?.trim() || selectedOrgName.trim() || undefined)
    try {
      const resp = await tauriInvoke<RevokeApiKeyResponse>('cmd_server_revoke_api_key', {
        config: serverConfig,
        keyId,
        orgName: orgName ?? null,
      })
      if (resp.success) {
        await get().loadApiKeys(params)
      } else {
        set({ error: resp.message || 'No se pudo revocar la API key.' })
      }
      return resp.success
    } catch (e) {
      set({ error: parseCommandError(String(e)).message })
      return false
    }
  },

  clearError: () => set({ error: null }),

  disconnect: () => {
    // Teardown SSE connection
    get().disconnectSse()
    persistServerConfig(null)
    void persistSecureControlPlaneApiKey(undefined)
    void syncOutboxServerConfig(null)
    set({
      serverConfig: null,
      isConnected: false,
      sseConnected: false,
      connectionStatus: 'disconnected',
      maintenanceDetectedAt: null,
      serverStats: null,
      serverLogs: [],
      activeDevs7d: [],
      activeDevs7dUpdatedAt: null,
      logsPage: 0,
      jenkinsCorrelations: [],
      prMergeEvidence: [],
      dailyActivity: [],
      ticketCoverage: null,
      jiraCoverageFilters: readStoredJiraCoverageFilters(),
      jiraTicketDetails: {},
      jiraTicketDetailFetchedAt: {},
      jiraTicketDetailLoading: {},
      evidencePacket: null,
      evidencePacketTicketId: '',
      isEvidencePacketLoading: false,
      enterpriseAdoptionProfile: null,
      enterpriseAdoptionProfileUpdatedAt: null,
      isEnterpriseAdoptionProfileLoading: false,
      isEnterpriseAdoptionProfileSaving: false,
      enterpriseAdoptionProfileError: null,
      enterpriseOnboardingChecklistTracking: null,
      enterpriseOnboardingChecklistTrackingUpdatedAt: null,
      isEnterpriseOnboardingChecklistTrackingLoading: false,
      isEnterpriseOnboardingChecklistTrackingSaving: false,
      enterpriseOnboardingChecklistTrackingError: null,
      releaseApprovals: [],
      releaseApprovalsTotal: 0,
      releaseApprovalsFilters: { limit: 10, offset: 0 },
      deploymentGateAuthorizations: [],
      deploymentGateAuthorizationsTotal: 0,
      deploymentGateAuthorizationsFilters: { limit: 10, offset: 0 },
      deploymentGateAuthorizationsUpdatedAt: null,
      deploymentGateRiskContexts: {},
      changeRiskEvaluations: [],
      changeRiskEvaluationsTotal: 0,
      changeRiskEvaluationsFilters: { limit: 10, offset: 0 },
      changeRiskSelectedEvaluation: null,
      changeRiskRuleCatalog: null,
      changeRiskEvaluationTrace: null,
      changeRiskEvaluationReview: null,
      changeRiskCabPackets: [],
      changeRiskCabPacketsTotal: 0,
      changeRiskCabPacketsFilters: { limit: 10, offset: 0 },
      changeRiskCabPacket: null,
      changeRiskCabPacketArtifact: null,
      changeRiskCabPacketReview: null,
      changeRiskCabDecisionManifests: [],
      changeRiskCabDecisionManifestsTotal: 0,
      changeRiskCabDecisionManifest: null,
      changeRiskCabDecisionManifestArtifact: null,
      isChangeRiskEvaluationsLoading: false,
      isChangeRiskRulesLoading: false,
      isChangeRiskTraceLoading: false,
      isChangeRiskEvaluationCreating: false,
      isChangeRiskReviewLoading: false,
      isChangeRiskReviewUpdating: false,
      isChangeRiskCabPacketsLoading: false,
      isChangeRiskCabPacketCreating: false,
      isChangeRiskCabPacketDownloading: false,
      isChangeRiskCabPacketArchiving: false,
      isChangeRiskCabPacketReviewLoading: false,
      isChangeRiskCabPacketReviewUpdating: false,
      isChangeRiskCabDecisionManifestCreating: false,
      isChangeRiskCabDecisionManifestsLoading: false,
      isChangeRiskCabDecisionManifestDownloading: false,
      isChangeRiskCabDecisionManifestRevoking: false,
      changeRiskError: null,
      releaseGovernanceEvaluation: null,
      isReleaseGovernanceEvaluating: false,
      isReleaseApprovalsLoading: false,
      isDeploymentGateRiskContextLoading: false,
      isReleaseApprovalSubmitting: false,
      releaseApprovalError: null,
      deploymentGateRiskContextError: null,
      userRole: null,
      userClientId: null,
      userOrgId: null,
      controlPlaneAuthConfirmed: true,
      pendingControlPlaneSession: null,
      selectedOrgName: '',
      selectedOrgValidated: false,
      availableOrgs: [],
      isLoadingOrgs: false,
      orgUsers: [],
      orgUsersTotal: 0,
      orgInvitations: [],
      orgInvitationsTotal: 0,
      lastGeneratedInviteToken: null,
      teamOverview: [],
      teamOverviewTotal: 0,
      teamRepos: [],
      teamReposTotal: 0,
      teamWindowDays: 30,
      teamStatusFilter: '',
      apiKeys: [],
      isLoadingApiKeys: false,
      exportLogs: [],
      isRefreshingDashboard: false,
      error: null,
      isChatLoading: false,
      governanceCopilotResponse: null,
      isGovernanceCopilotLoading: false,
      governanceCopilotError: null,
    })
  },

  // ── Chat actions ─────────────────────────────────────────────────────────
  }
}
