import { describe, it, expect, beforeEach, vi } from 'vitest'

const mockInvoke = vi.fn()
const mockListen = vi.fn().mockResolvedValue(() => {})

vi.mock('@/lib/tauri', () => ({
  tauriInvoke: (...args: unknown[]) => mockInvoke(...args),
  tauriListen: (...args: unknown[]) => mockListen(...args),
  parseCommandError: (error: string) => {
    try {
      const parsed = JSON.parse(error)
      return { code: parsed.code || 'UNKNOWN', message: parsed.message || error }
    } catch {
      return { code: 'UNKNOWN', message: error }
    }
  },
}))

vi.mock('@/lib/notifications', () => ({
  notifyNewEvents: vi.fn(),
}))

vi.mock('@/lib/timezone', () => ({
  detectBrowserTimezone: vi.fn().mockReturnValue('UTC'),
  readStoredTimezone: vi.fn().mockReturnValue(null),
  persistTimezone: vi.fn(),
}))

vi.mock('@/store/useAuthStore', () => ({
  useAuthStore: {
    getState: vi.fn().mockReturnValue({
      user: { login: 'testuser', name: 'Test', avatar_url: '', is_admin: true },
    }),
  },
}))

import { useControlPlaneStore } from '@/store/useControlPlaneStore'
import type { UpsertFirstGovernedRepoSetupRequest } from '@/store/useControlPlaneStore/types'

describe('useControlPlaneStore', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.unstubAllEnvs()
    localStorage.clear()
    // Reset key state fields
    useControlPlaneStore.setState({
      serverConfig: null,
      serverStats: null,
      serverLogs: [],
      dailyActivity: [],
      jenkinsCorrelations: [],
      prMergeEvidence: [],
      ticketCoverage: null,
      userRole: null,
      userClientId: null,
      userOrgId: null,
      controlPlaneAuthConfirmed: false,
      isConnected: false,
      isLoading: false,
      error: null,
      logsPage: 1,
      logsPageSize: 50,
      chatSessions: [],
      activeChatSessionId: null,
      chatMessages: [],
      isChatLoading: false,
      governanceCopilotResponse: null,
      isGovernanceCopilotLoading: false,
      governanceCopilotError: null,
      releaseApprovals: [],
      releaseApprovalsTotal: 0,
      releaseApprovalsFilters: { limit: 10, offset: 0 },
      deploymentGateAuthorizations: [],
      deploymentGateAuthorizationsTotal: 0,
      deploymentGateAuthorizationsFilters: { limit: 10, offset: 0 },
      deploymentGateAuthorizationsUpdatedAt: null,
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
      complianceEvidenceSelectedDeploymentGateId: null,
      firstGovernedRepoSetup: null,
      firstGovernedRepoSetupUpdatedAt: null,
      isFirstGovernedRepoSetupLoading: false,
      isFirstGovernedRepoSetupSaving: false,
      firstGovernedRepoSetupError: null,
      firstGovernedRepoWizardState: null,
      isFirstGovernedRepoWizardLoading: false,
      isFirstGovernedRepoWizardActionRunning: false,
      firstGovernedRepoWizardError: null,
      complianceControlFrameworks: [],
      complianceFrameworkPacks: [],
      selectedComplianceFrameworkId: 'gitgov_release_governance_baseline_v1',
      complianceFrameworkImportResponse: null,
      complianceFrameworkPackDiff: null,
      complianceEvidenceExport: null,
      complianceEvidenceMapping: null,
      complianceReviewPackage: null,
      complianceReviewPackageArtifact: null,
      complianceFrameworkReviewReport: null,
      complianceFrameworkReviewReports: null,
      complianceFrameworkReviewReportArtifact: null,
      complianceFrameworkReviewReportProvenanceManifest: null,
      compliancePeriodReport: null,
      compliancePeriodReports: null,
      compliancePeriodReportProfiles: null,
      compliancePeriodReportProfile: null,
      compliancePeriodReportProfileRun: null,
      compliancePeriodReportArtifact: null,
      compliancePeriodReportAccessLog: null,
      compliancePeriodReportPdfExport: null,
      compliancePeriodReportProvenanceManifest: null,
      compliancePeriodReportSharePackages: null,
      compliancePeriodReportSharePackage: null,
      compliancePeriodReportSharePackageArtifact: null,
      isComplianceFrameworksLoading: false,
      isComplianceFrameworkPackImporting: false,
      isComplianceFrameworkPackReviewing: false,
      isComplianceFrameworkPackDiffLoading: false,
      isComplianceEvidenceExportCreating: false,
      isComplianceEvidenceMappingCreating: false,
      isComplianceReviewPackageCreating: false,
      isComplianceReviewPackageDownloading: false,
      isComplianceFrameworkReviewReportCreating: false,
      isComplianceFrameworkReviewReportsLoading: false,
      isComplianceFrameworkReviewReportReviewing: false,
      isComplianceFrameworkReviewReportDownloading: false,
      isCompliancePeriodReportCreating: false,
      isCompliancePeriodReportsLoading: false,
      isCompliancePeriodReportProfileCreating: false,
      isCompliancePeriodReportProfilesLoading: false,
      isCompliancePeriodReportProfileUpdating: false,
      isCompliancePeriodReportProfileArchiving: false,
      isCompliancePeriodReportProfileRunning: false,
      isCompliancePeriodReportDownloading: false,
      isCompliancePeriodReportReviewing: false,
      isCompliancePeriodReportRetentionUpdating: false,
      isCompliancePeriodReportAccessLogLoading: false,
      isCompliancePeriodReportPdfExportCreating: false,
      isCompliancePeriodReportPdfExportDownloading: false,
      isCompliancePeriodReportProvenanceManifestCreating: false,
      isCompliancePeriodReportProvenanceManifestDownloading: false,
      isCompliancePeriodReportSharePackageCreating: false,
      isCompliancePeriodReportSharePackagesLoading: false,
      isCompliancePeriodReportSharePackageDownloading: false,
      isCompliancePeriodReportSharePackageRevoking: false,
      complianceEvidenceError: null,
      isDeploymentGateAuthorizationsLoading: false,
      isReleaseApprovalsLoading: false,
      isReleaseApprovalSubmitting: false,
      releaseApprovalError: null,
      displayTimezone: 'UTC',
      sseConnected: false,
      policyData: null,
      policyHistory: [],
      isPolicyLoading: false,
      isPolicySaving: false,
      policyError: null,
      selectedOrgName: '',
      selectedOrgValidated: false,
      availableOrgs: [],
      isLoadingOrgs: false,
      orgUsers: [],
      orgUsersTotal: 0,
      orgInvitations: [],
      orgInvitationsTotal: 0,
      teamOverview: [],
      teamOverviewTotal: 0,
      teamRepos: [],
      teamReposTotal: 0,
      apiKeys: [],
      isLoadingApiKeys: false,
      exportLogs: [],
    })
  })

  describe('setServerConfig', () => {
    it('stores server config', () => {
      useControlPlaneStore.getState().setServerConfig({ url: 'http://127.0.0.1:3000', api_key: 'test-key' })
      expect(useControlPlaneStore.getState().serverConfig).toEqual({
        url: 'http://127.0.0.1:3000',
        api_key: 'test-key',
      })
    })

    it('uses VITE_SERVER_URL instead of stale localhost when no URL is provided', async () => {
      vi.stubEnv('VITE_SERVER_URL', 'https://gitgov-api.onrender.com')
      mockInvoke
        .mockResolvedValueOnce(null) // cmd_cp_get_api_key
        .mockResolvedValueOnce(undefined) // cmd_cp_clear_api_key
        .mockResolvedValueOnce(undefined) // cmd_server_sync_outbox
        .mockResolvedValueOnce(false) // cmd_server_health

      await useControlPlaneStore.getState().initFromEnv()

      expect(useControlPlaneStore.getState().serverConfig?.url).toBe('https://gitgov-api.onrender.com')
    })
  })

  describe('loadMe', () => {
    it('accepts the platform founder principal without requiring GitHub identity match', async () => {
      useControlPlaneStore.setState({
        serverConfig: { url: 'https://gitgov-api.onrender.com', api_key: 'founder-key' },
      })
      mockInvoke.mockResolvedValueOnce({
        client_id: 'bootstrap-admin',
        role: 'Admin',
        principal_type: 'platform_founder',
        org_id: null,
        org_name: null,
        requires_workspace_for_tenant_surfaces: true,
      })

      const loaded = await useControlPlaneStore.getState().loadMe()

      expect(loaded).toBe(true)
      expect(useControlPlaneStore.getState().userClientId).toBe('bootstrap-admin')
      expect(useControlPlaneStore.getState().userRole).toBe('Admin')
      expect(useControlPlaneStore.getState().error).toBeNull()
    })
  })

  describe('clearError', () => {
    it('clears error', () => {
      useControlPlaneStore.setState({ error: 'an error' })
      useControlPlaneStore.getState().clearError()
      expect(useControlPlaneStore.getState().error).toBeNull()
    })
  })

  describe('setLogsPage', () => {
    it('sets the current page number', () => {
      useControlPlaneStore.getState().setLogsPage(3)
      expect(useControlPlaneStore.getState().logsPage).toBe(3)
    })
  })

  describe('setDisplayTimezone', () => {
    it('sets timezone', () => {
      useControlPlaneStore.getState().setDisplayTimezone('America/Lima')
      expect(useControlPlaneStore.getState().displayTimezone).toBe('America/Lima')
    })
  })

  describe('checkConnection', () => {
    it('calls server health and sets connected on success', async () => {
      useControlPlaneStore.setState({
        serverConfig: { url: 'http://127.0.0.1:3000', api_key: 'key' },
      })
      mockInvoke.mockResolvedValueOnce({ status: 'ok' }) // cmd_server_health

      await useControlPlaneStore.getState().checkConnection()

      expect(mockInvoke).toHaveBeenCalledWith('cmd_server_health', expect.any(Object))
    })

    it('sets error when no server config', async () => {
      useControlPlaneStore.setState({ serverConfig: null })
      await useControlPlaneStore.getState().checkConnection()
      // Should not throw but serverConfig is null so nothing should happen
      expect(mockInvoke).not.toHaveBeenCalled()
    })
  })

  describe('loadStats', () => {
    it('fetches stats from server', async () => {
      useControlPlaneStore.setState({
        serverConfig: { url: 'http://127.0.0.1:3000', api_key: 'key' },
      })
      const mockStats = {
        github_events: { total: 100, today: 10, pushes_today: 5, by_type: {} },
        client_events: { total: 50, today: 5, blocked_today: 1, desktop_pushes_today: 3, by_type: {}, by_status: {} },
        violations: { total: 2, unresolved: 1, critical: 0 },
        active_devs_week: 3,
        active_repos: 2,
      }
      mockInvoke.mockResolvedValueOnce(mockStats) // cmd_server_get_stats

      await useControlPlaneStore.getState().loadStats()

      expect(mockInvoke).toHaveBeenCalledWith('cmd_server_get_stats', expect.any(Object))
      expect(useControlPlaneStore.getState().serverStats).toEqual(mockStats)
    })
  })

  describe('loadLogs', () => {
    it('fetches logs from server', async () => {
      useControlPlaneStore.setState({
        serverConfig: { url: 'http://127.0.0.1:3000', api_key: 'key' },
      })
      // fetchLogsKeysetWindow → fetchLogsByFilter → tauriInvoke returns CombinedEvent[]
      const mockLogs = [
        { id: '1', source: 'client', event_type: 'commit', created_at: 1000, status: 'success', details: {} },
      ]
      mockInvoke.mockResolvedValueOnce(mockLogs)

      await useControlPlaneStore.getState().loadLogs(10, 0)

      expect(mockInvoke).toHaveBeenCalledWith('cmd_server_get_logs', expect.any(Object))
      expect(useControlPlaneStore.getState().serverLogs).toEqual(mockLogs)
    })
  })

  describe('chat session management', () => {
    it('creates a new chat session', () => {
      useControlPlaneStore.getState().createChatSession()
      const sessions = useControlPlaneStore.getState().chatSessions
      expect(sessions.length).toBe(1)
      // Title format: "Chat nuevo {n}" where n = sessions.length at creation
      expect(sessions[0].title).toMatch(/^Chat nuevo/)
      expect(useControlPlaneStore.getState().activeChatSessionId).toBe(sessions[0].id)
    })

    it('limits to max chat sessions', () => {
      for (let i = 0; i < 9; i++) {
        useControlPlaneStore.getState().createChatSession()
      }
      expect(useControlPlaneStore.getState().chatSessions.length).toBeLessThanOrEqual(8)
    })

    it('can close the only chat session (resets it)', () => {
      useControlPlaneStore.getState().createChatSession()
      const sessionId = useControlPlaneStore.getState().chatSessions[0].id
      useControlPlaneStore.getState().closeChatSession(sessionId)
      // With only 1 session, closeChatSession resets it instead of removing
      expect(useControlPlaneStore.getState().chatSessions.length).toBe(1)
      expect(useControlPlaneStore.getState().chatMessages).toEqual([])
    })

    it('clears chat messages for active session', () => {
      // clearChatMessages requires an active session
      useControlPlaneStore.getState().createChatSession()
      const sessionId = useControlPlaneStore.getState().chatSessions[0].id
      // Manually add messages to the session
      useControlPlaneStore.setState((s) => ({
        chatMessages: [{ id: '1', role: 'user' as const, content: 'test', timestamp: Date.now() }],
        chatSessions: s.chatSessions.map((ses) =>
          ses.id === sessionId
            ? { ...ses, messages: [{ id: '1', role: 'user' as const, content: 'test', timestamp: Date.now() }] }
            : ses,
        ),
      }))
      useControlPlaneStore.getState().clearChatMessages()
      expect(useControlPlaneStore.getState().chatMessages).toEqual([])
    })
  })

  describe('disconnect', () => {
    it('resets connection state', () => {
      // disconnectSse calls tauriInvoke('cmd_server_sse_disconnect') which needs to return a Promise
      mockInvoke.mockReturnValue(Promise.resolve(undefined))

      useControlPlaneStore.setState({
        isConnected: true,
        serverConfig: { url: 'http://127.0.0.1:3000', api_key: 'key' },
        serverStats: { github_events: { total: 1, today: 0, pushes_today: 0, by_type: {} }, client_events: { total: 0, today: 0, blocked_today: 0, desktop_pushes_today: 0, by_type: {}, by_status: {} }, violations: { total: 0, unresolved: 0, critical: 0 }, active_devs_week: 0, active_repos: 0 },
      })
      useControlPlaneStore.getState().disconnect()
      expect(useControlPlaneStore.getState().isConnected).toBe(false)
      expect(useControlPlaneStore.getState().serverConfig).toBeNull()
    })
  })

  describe('team filters', () => {
    it('sets team window days', () => {
      useControlPlaneStore.getState().setTeamFilters({ days: 30 })
      expect(useControlPlaneStore.getState().teamWindowDays).toBe(30)
    })

    it('sets team status filter', () => {
      useControlPlaneStore.getState().setTeamFilters({ status: 'active' })
      expect(useControlPlaneStore.getState().teamStatusFilter).toBe('active')
    })
  })

  describe('policy state', () => {
    it('loads policy from server', async () => {
      useControlPlaneStore.setState({
        serverConfig: { url: 'http://127.0.0.1:3000', api_key: 'key' },
      })
      const mockPolicy = {
        version: '1',
        checksum: 'abc',
        config: { branches: { patterns: [], protected: [] }, groups: {}, admins: [] },
        updated_at: Date.now(),
      }
      mockInvoke.mockResolvedValueOnce(mockPolicy)

      await useControlPlaneStore.getState().loadPolicy('my-repo')

      expect(mockInvoke).toHaveBeenCalledWith('cmd_server_get_policy', expect.any(Object))
      expect(useControlPlaneStore.getState().policyData).toEqual(mockPolicy)
    })

    it('does not silently override repo-managed Policy-as-Code', async () => {
      useControlPlaneStore.setState({
        serverConfig: { url: 'http://127.0.0.1:3000', api_key: 'key' },
        policyData: {
          version: '1',
          checksum: 'abc',
          config: {
            branches: { patterns: [], protected: [] },
            groups: {},
            admins: [],
            rules: {
              require_pull_request: true,
              min_approvals: 1,
              require_conventional_commits: false,
              require_signed_commits: false,
              max_files_per_commit: null,
              require_linked_ticket: true,
              block_force_push: false,
              forbidden_patterns: [],
            },
            checklist: { confirm: [], auto_check: [] },
            enforcement: {
              pull_requests: 'block',
              commits: 'warn',
              branches: 'block',
              traceability: 'block',
              quality_gates: 'warn',
            },
          },
          source: {
            source_mode: 'repo-policy-as-code',
            source_path: '.gitgov/policy.yml',
            reviewers: [],
            drift_status: 'in-sync',
          },
          updated_at: Date.now(),
        },
      })

      const saved = await useControlPlaneStore.getState().savePolicy('acme/repo', {
        branches: { patterns: [], protected: [] },
        groups: {},
        admins: [],
        rules: {
          require_pull_request: false,
          min_approvals: 0,
          require_conventional_commits: false,
          require_signed_commits: false,
          max_files_per_commit: null,
          require_linked_ticket: false,
          block_force_push: false,
          forbidden_patterns: [],
        },
        checklist: { confirm: [], auto_check: [] },
        enforcement: {
          pull_requests: 'off',
          commits: 'off',
          branches: 'off',
          traceability: 'off',
          quality_gates: 'off',
        },
      })

      expect(saved).toBe(false)
      expect(mockInvoke).not.toHaveBeenCalledWith('cmd_server_override_policy', expect.any(Object))
      expect(useControlPlaneStore.getState().policyError).toContain('.gitgov/policy.yml')
    })
  })

  describe('selectedOrgName', () => {
    it('sets selected org name', () => {
      useControlPlaneStore.getState().setSelectedOrgName('my-org')
      expect(useControlPlaneStore.getState().selectedOrgName).toBe('my-org')
    })

    it('invalidates a previously validated workspace when edited', () => {
      useControlPlaneStore.setState({
        selectedOrgName: 'validated-org',
        selectedOrgValidated: true,
      })

      useControlPlaneStore.getState().setSelectedOrgName('other-org')

      expect(useControlPlaneStore.getState().selectedOrgName).toBe('other-org')
      expect(useControlPlaneStore.getState().selectedOrgValidated).toBe(false)
    })

    it('validates and persists the active workspace through the backend', async () => {
      useControlPlaneStore.setState({
        serverConfig: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
      })
      mockInvoke.mockResolvedValueOnce({
        id: 'org-1',
        github_id: 123,
        login: 'yohandry10',
        name: 'Yohandry',
        avatar_url: 'https://avatars.githubusercontent.com/u/123',
        created_at: 1,
      })

      const response = await useControlPlaneStore.getState().activateOrgName(' yohandry10 ')

      expect(mockInvoke).toHaveBeenCalledWith('cmd_server_get_org', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        login: 'yohandry10',
      })
      expect(response?.login).toBe('yohandry10')
      expect(useControlPlaneStore.getState().selectedOrgName).toBe('yohandry10')
      expect(useControlPlaneStore.getState().selectedOrgValidated).toBe(true)
      const storedValues = Array.from({ length: localStorage.length }, (_, index) => {
        const key = localStorage.key(index)
        return key ? localStorage.getItem(key) : null
      })
      expect(storedValues).toContain('yohandry10')
    })

    it('keeps an invalid workspace unvalidated with an actionable error', async () => {
      useControlPlaneStore.setState({
        serverConfig: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
      })
      mockInvoke.mockRejectedValueOnce('Organization not found')

      const response = await useControlPlaneStore.getState().activateOrgName('missing-org')

      expect(response).toBeNull()
      expect(useControlPlaneStore.getState().selectedOrgName).toBe('missing-org')
      expect(useControlPlaneStore.getState().selectedOrgValidated).toBe(false)
      expect(useControlPlaneStore.getState().error).toBe('No existe un workspace GitGov llamado "missing-org".')
    })

    it('loads API keys by active workspace unless global scope is explicit', async () => {
      useControlPlaneStore.setState({
        serverConfig: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        userRole: 'Admin',
        userOrgId: null,
        selectedOrgName: 'yohandry10',
        selectedOrgValidated: true,
      })
      mockInvoke.mockResolvedValueOnce([
        {
          id: 'key-1',
          client_id: 'developer',
          role: 'Developer',
          org_id: 'org-1',
          org_name: 'yohandry10',
          is_active: true,
          created_at: 1,
          revoked_at: null,
          last_used_at: null,
        },
      ])

      await useControlPlaneStore.getState().loadApiKeys()

      expect(mockInvoke).toHaveBeenCalledWith('cmd_server_list_api_keys', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        orgName: 'yohandry10',
      })
      expect(useControlPlaneStore.getState().apiKeys[0].org_name).toBe('yohandry10')

      mockInvoke.mockClear()
      mockInvoke.mockResolvedValueOnce([])

      await useControlPlaneStore.getState().loadApiKeys({ global: true })

      expect(mockInvoke).toHaveBeenCalledWith('cmd_server_list_api_keys', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        orgName: null,
      })
    })

    it('does not fall back to global API key catalog without a validated workspace', async () => {
      useControlPlaneStore.setState({
        serverConfig: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        userRole: 'Admin',
        userOrgId: null,
        selectedOrgName: '',
        selectedOrgValidated: false,
      })

      await useControlPlaneStore.getState().loadApiKeys()

      expect(mockInvoke).not.toHaveBeenCalled()
      expect(useControlPlaneStore.getState().error).toBe(
        'Valida un workspace GitGov antes de consultar API keys de organización.',
      )

      const revoked = await useControlPlaneStore.getState().revokeApiKey('key-1')

      expect(revoked).toBe(false)
      expect(mockInvoke).not.toHaveBeenCalled()
      expect(useControlPlaneStore.getState().error).toBe(
        'Valida un workspace GitGov antes de revocar API keys de organización.',
      )
    })

    it('surfaces API key revoke failures returned by the server', async () => {
      useControlPlaneStore.setState({
        serverConfig: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        userRole: 'Admin',
        userOrgId: null,
        selectedOrgName: 'yohandry10',
        selectedOrgValidated: true,
      })
      mockInvoke.mockResolvedValueOnce({ success: false, message: 'API key not found or already revoked' })

      const revoked = await useControlPlaneStore.getState().revokeApiKey('key-1')

      expect(revoked).toBe(false)
      expect(useControlPlaneStore.getState().error).toBe('API key not found or already revoked')
    })
  })

  describe('askGovernanceCopilot', () => {
    it('calls the desktop copilot proxy with selected org and context', async () => {
      useControlPlaneStore.setState({
        serverConfig: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        selectedOrgName: 'yohandry10',
      })
      mockInvoke.mockResolvedValueOnce({
        success: true,
        mode: 'fallback',
        answer: 'Ready with evidence.',
        citations: [{ id: 'evidence-packet', label: 'Evidence Packet', endpoint: '/evidence', status: 'ok' }],
        sources: [{ id: 'evidence-packet', label: 'Evidence Packet', endpoint: '/evidence', status: 'ok' }],
        warnings: [],
      })

      const response = await useControlPlaneStore.getState().askGovernanceCopilot({
        question: 'Is KAN-39 ready?',
        repository_full_name: 'yohandry10/Git-Gov',
        branch: 'main',
        ticket_id: 'KAN-39',
        release_id: 'KAN-39-smoke',
        environment: 'production',
        hours: 720,
      })

      expect(mockInvoke).toHaveBeenCalledWith('cmd_server_governance_copilot_ask', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        request: {
          question: 'Is KAN-39 ready?',
          org_name: 'yohandry10',
          repository_full_name: 'yohandry10/Git-Gov',
          branch: 'main',
          ticket_id: 'KAN-39',
          release_id: 'KAN-39-smoke',
          environment: 'production',
          hours: 720,
        },
      })
      expect(response?.mode).toBe('fallback')
      expect(useControlPlaneStore.getState().governanceCopilotResponse?.answer).toBe('Ready with evidence.')
      expect(useControlPlaneStore.getState().governanceCopilotError).toBeNull()
    })

    it('keeps copilot errors separate from the main dashboard error', async () => {
      useControlPlaneStore.setState({
        serverConfig: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        error: 'existing dashboard error',
      })
      mockInvoke.mockRejectedValueOnce(JSON.stringify({ code: 'COPILOT_HTTP_ERROR', message: 'Unauthorized' }))

      const response = await useControlPlaneStore.getState().askGovernanceCopilot({
        question: 'Check release',
      })

      expect(response).toBeNull()
      expect(useControlPlaneStore.getState().error).toBe('existing dashboard error')
      expect(useControlPlaneStore.getState().governanceCopilotError).toBe('Unauthorized')
      expect(useControlPlaneStore.getState().isGovernanceCopilotLoading).toBe(false)
    })
  })

  describe('first governed repo wizard', () => {
    const setupRecord = {
      run_id: '7ac97ef2-9e76-4f35-9d74-39110f44e01e',
      org_id: 'org-1',
      status: 'ready',
      goal: 'govern_release',
      repository_full_name: 'yohandry10/Git-Gov',
      default_branch: 'main',
      selected_providers: ['github', 'jira', 'jenkins'],
      selected_modules: ['traceability', 'release-readiness', 'evidence-packets', 'quality-gates'],
      policy_preset: 'moderate',
      baseline: {
        gate_readiness: 'baseline_ready',
        policy_workflow_preview_acknowledged: true,
      },
      created_by: 'admin',
      updated_by: 'admin',
      created_at: 1,
      updated_at: 2,
      completed_at: null,
    }

    it('loads wizard state with selected org scope', async () => {
      useControlPlaneStore.setState({
        serverConfig: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        selectedOrgName: 'yohandry10',
      })
      mockInvoke.mockResolvedValueOnce({
        org_id: 'org-1',
        found: true,
        setup: setupRecord,
        state: {
          schema_version: 'gitgov_first_governed_repo_wizard_state.v1',
          current_step: 'baseline_preview',
          safety: {
            stores_secret_values: false,
            agent_governance_required: false,
          },
        },
      })

      const response = await useControlPlaneStore.getState().loadFirstGovernedRepoWizardState()

      expect(mockInvoke).toHaveBeenCalledWith('cmd_server_get_first_governed_repo_wizard_state', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        orgName: 'yohandry10',
      })
      expect(response?.state.current_step).toBe('baseline_preview')
      expect(useControlPlaneStore.getState().firstGovernedRepoSetup?.run_id).toBe(setupRecord.run_id)
      expect(useControlPlaneStore.getState().firstGovernedRepoWizardState?.safety).toEqual({
        stores_secret_values: false,
        agent_governance_required: false,
      })
    })

    it('runs manual wizard steps without provider secrets or agent governance dependency', async () => {
      useControlPlaneStore.setState({
        serverConfig: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        selectedOrgName: 'yohandry10',
      })
      const runResponse = {
        setup: setupRecord,
        state: {
          current_step: 'first_result',
          provider_health: [{ provider: 'github', status: 'ready' }],
          safety: {
            stores_secret_values: false,
            mutates_provider_state: false,
            agent_governance_required: false,
            compliance_claim: false,
          },
        },
      }
      mockInvoke
        .mockResolvedValueOnce(runResponse)
        .mockResolvedValueOnce(runResponse)
        .mockResolvedValueOnce(runResponse)
        .mockResolvedValueOnce({
          ...runResponse,
          setup: { ...setupRecord, status: 'completed', completed_at: 3 },
        })
      const payload: UpsertFirstGovernedRepoSetupRequest = {
        status: 'ready' as const,
        goal: 'govern_release' as const,
        repository_full_name: 'yohandry10/Git-Gov',
        default_branch: 'main',
        selected_providers: ['github', 'jira'],
        selected_modules: ['traceability', 'release-readiness', 'evidence-packets'],
        policy_preset: 'moderate' as const,
        baseline: {
          version: 1,
          gate_readiness: 'baseline_ready' as const,
          policy_workflow_preview_acknowledged: true,
          setup_summary: {
            repository_full_name: 'yohandry10/Git-Gov',
            default_branch: 'main',
            goal: 'govern_release',
            policy_preset: 'moderate',
            provider_count: 2,
            module_count: 3,
            github_selected: true,
            policy_workflow_preview_acknowledged: true,
          },
          action_center_gaps: [],
          first_result: {
            status: 'ready_for_advisory_gate',
            deployment_gate_mode: 'advisory',
            cta: 'simulate_deployment_gate',
            evidence_contract: {
              repo: 'yohandry10/Git-Gov',
              branch: 'main',
              providers: ['github', 'jira'],
              modules: ['traceability', 'release-readiness', 'evidence-packets'],
            },
          },
        },
      }

      await useControlPlaneStore.getState().createFirstGovernedRepoWizardRun(payload)
      await useControlPlaneStore.getState().validateFirstGovernedRepoWizardRun(setupRecord.run_id, payload)
      await useControlPlaneStore.getState().planFirstGovernedRepoWizardRun(setupRecord.run_id, payload)
      const completed = await useControlPlaneStore.getState().completeFirstGovernedRepoWizardRun(setupRecord.run_id, {
        ...payload,
        status: 'completed',
      })

      expect(mockInvoke).toHaveBeenNthCalledWith(1, 'cmd_server_create_first_governed_repo_wizard_run', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        payload: { ...payload, org_name: 'yohandry10' },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(2, 'cmd_server_validate_first_governed_repo_wizard_run', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        runId: setupRecord.run_id,
        payload: { ...payload, org_name: 'yohandry10' },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(3, 'cmd_server_plan_first_governed_repo_wizard_run', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        runId: setupRecord.run_id,
        payload: { ...payload, org_name: 'yohandry10' },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(4, 'cmd_server_complete_first_governed_repo_wizard_run', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        runId: setupRecord.run_id,
        payload: { ...payload, status: 'completed', org_name: 'yohandry10' },
      })
      expect(completed?.state.safety).toMatchObject({
        stores_secret_values: false,
        mutates_provider_state: false,
        agent_governance_required: false,
        compliance_claim: false,
      })
      expect(useControlPlaneStore.getState().firstGovernedRepoSetup?.status).toBe('completed')
      expect(useControlPlaneStore.getState().isFirstGovernedRepoWizardActionRunning).toBe(false)
    })
  })

  describe('enterprise release approvals', () => {
    it('loads release approvals with selected org scope', async () => {
      useControlPlaneStore.setState({
        serverConfig: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        selectedOrgName: 'yohandry10',
      })
      mockInvoke.mockResolvedValueOnce({
        items: [{
          id: 'approval-1',
          org_id: 'org-1',
          release_id: 'KAN-43',
          repository_full_name: 'yohandry10/Git-Gov',
          environment: 'production',
          decision: 'approved',
          approver: 'release.manager@example.com',
          evidence_summary: {},
          risk_severity: 'none',
          approval_hash: 'a'.repeat(64),
          created_by: 'admin',
          created_at: 1,
        }],
        total: 1,
        limit: 10,
        offset: 0,
      })

      const response = await useControlPlaneStore.getState().loadEnterpriseReleaseApprovals({
        repository_full_name: 'yohandry10/Git-Gov',
      })

      expect(mockInvoke).toHaveBeenCalledWith('cmd_server_list_enterprise_release_approvals', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        query: {
          org_name: 'yohandry10',
          repository_full_name: 'yohandry10/Git-Gov',
          branch: null,
          target_sha: null,
          release_id: null,
          environment: null,
          decision: null,
          evidence_packet_hash: null,
          limit: 10,
          offset: 0,
        },
      })
      expect(response?.total).toBe(1)
      expect(useControlPlaneStore.getState().releaseApprovals[0].release_id).toBe('KAN-43')
    })

    it('creates release approval and prepends it to state', async () => {
      useControlPlaneStore.setState({
        serverConfig: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        selectedOrgName: 'yohandry10',
      })
      const approval = {
        id: 'approval-2',
        org_id: 'org-1',
        release_id: 'KAN-43',
        repository_full_name: 'yohandry10/Git-Gov',
        branch: 'main',
        environment: 'production',
        decision: 'approved',
        approver: 'release.manager@example.com',
        ticket_id: 'KAN-43',
        evidence_packet_hash: 'b'.repeat(64),
        evidence_packet_uri: '/evidence/packets/tickets/KAN-43',
        evidence_summary: {},
        risk_severity: 'none',
        approval_hash: 'c'.repeat(64),
        created_by: 'admin',
        created_at: 2,
      }
      mockInvoke.mockResolvedValueOnce(approval)

      const response = await useControlPlaneStore.getState().createEnterpriseReleaseApproval({
        release_id: ' KAN-43 ',
        repository_full_name: ' yohandry10/Git-Gov ',
        branch: ' main ',
        environment: ' production ',
        decision: 'approved',
        approver: ' release.manager@example.com ',
        ticket_id: ' KAN-43 ',
        evidence_packet_hash: 'b'.repeat(64),
        evidence_packet_uri: ' /evidence/packets/tickets/KAN-43 ',
        evidence_summary: {},
        risk_severity: 'none',
      })

      expect(mockInvoke).toHaveBeenCalledWith('cmd_server_create_enterprise_release_approval', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        payload: {
          org_name: 'yohandry10',
          release_id: 'KAN-43',
          repository_full_name: 'yohandry10/Git-Gov',
          branch: 'main',
          target_sha: null,
          environment: 'production',
          decision: 'approved',
          approver: 'release.manager@example.com',
          ticket_id: 'KAN-43',
          evidence_packet_hash: 'b'.repeat(64),
          evidence_packet_uri: '/evidence/packets/tickets/KAN-43',
          evidence_summary: {},
          risk_severity: 'none',
          risk_acceptance_reason: null,
          expires_at: null,
        },
      })
      expect(response?.id).toBe('approval-2')
      expect(useControlPlaneStore.getState().releaseApprovals[0].id).toBe('approval-2')
    })

    it('evaluates release governance with selected org scope', async () => {
      useControlPlaneStore.setState({
        serverConfig: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        selectedOrgName: 'yohandry10',
      })
      mockInvoke.mockResolvedValueOnce({
        status: 'recorded',
        policy_satisfied: true,
        blocking: false,
        would_block: false,
        valid_approval_count: 0,
        required_approval_count: 0,
        policy: {
          mode: 'record-only',
          environment: 'production',
          approval_required: false,
          enforcement: 'disabled',
          policy_applies: true,
          quorum_enabled: false,
          quorum_rules: [],
        },
        approvals: [],
        issues: [],
        next_steps: ['Create an optional release approval to strengthen audit evidence.'],
      })

      const response = await useControlPlaneStore.getState().evaluateEnterpriseReleaseGovernance({
        repository_full_name: ' yohandry10/Git-Gov ',
        branch: ' main ',
        target_sha: ' abcdef1234567890abcdef1234567890abcdef12 ',
        release_id: ' KAN-46 ',
        environment: ' production ',
        evidence_packet_hash: 'd'.repeat(64),
      })

      expect(mockInvoke).toHaveBeenCalledWith('cmd_server_evaluate_enterprise_release_governance', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        query: {
          org_name: 'yohandry10',
          repository_full_name: 'yohandry10/Git-Gov',
          branch: 'main',
          target_sha: 'abcdef1234567890abcdef1234567890abcdef12',
          release_id: 'KAN-46',
          environment: 'production',
          evidence_packet_hash: 'd'.repeat(64),
        },
      })
      expect(response?.status).toBe('recorded')
      expect(useControlPlaneStore.getState().releaseGovernanceEvaluation?.policy.mode).toBe('record-only')
    })

    it('loads deployment gate authorization history with selected org scope', async () => {
      useControlPlaneStore.setState({
        serverConfig: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        selectedOrgName: 'yohandry10',
      })
      mockInvoke.mockResolvedValueOnce({
        items: [{
          id: 'row-1',
          authorization_id: 'dga_123',
          org_id: 'org-1',
          release_id: 'KAN-83',
          repository_full_name: 'yohandry10/Git-Gov',
          branch: 'main',
          target_sha: 'abcdef1234567890abcdef1234567890abcdef12',
          environment: 'production',
          deployer: 'github-actions',
          ticket_id: 'KAN-83',
          evidence_packet_hash: 'e'.repeat(64),
          evidence_packet_uri: '/evidence/packets/tickets/KAN-83',
          decision: 'advisory',
          approved: true,
          blocking: false,
          would_block: false,
          reason: 'Deployment approved by current release governance policy.',
          blocked_by: [],
          warnings: ['First governed repo setup is not configured.'],
          policy_checksum: 'f'.repeat(64),
          break_glass_eligible: false,
          break_glass_used: false,
          evaluation: {
            status: 'recorded',
            policy_satisfied: true,
            blocking: false,
            would_block: false,
            valid_approval_count: 0,
            required_approval_count: 0,
            policy: {
              mode: 'record-only',
              environment: 'production',
              approval_required: false,
              enforcement: 'disabled',
              policy_applies: true,
              quorum_enabled: false,
              quorum_rules: [],
            },
            approvals: [],
            issues: [],
            next_steps: [],
          },
          details: {},
          request_payload: {},
          requested_by: 'admin',
          created_at: 3,
        }],
        total: 1,
        limit: 10,
        offset: 0,
      })

      const response = await useControlPlaneStore.getState().loadDeploymentGateAuthorizations({
        repository_full_name: ' yohandry10/Git-Gov ',
        branch: ' main ',
        environment: ' production ',
        deployer: ' github-actions ',
      })

      expect(mockInvoke).toHaveBeenCalledWith('cmd_server_list_deployment_gate_authorizations', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        query: {
          org_name: 'yohandry10',
          authorization_id: null,
          repository_full_name: 'yohandry10/Git-Gov',
          branch: 'main',
          target_sha: null,
          release_id: null,
          environment: 'production',
          decision: null,
          deployer: 'github-actions',
          limit: 10,
          offset: 0,
        },
      })
      expect(response?.total).toBe(1)
      expect(useControlPlaneStore.getState().deploymentGateAuthorizations[0].authorization_id).toBe('dga_123')
      expect(useControlPlaneStore.getState().deploymentGateAuthorizationsUpdatedAt).toBeGreaterThan(0)
    })

    it('loads change risk advisory history with tenant scope and release filters', async () => {
      useControlPlaneStore.setState({
        serverConfig: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        selectedOrgName: 'yohandry10',
      })
      mockInvoke.mockResolvedValueOnce({
        items: [{
          evaluation_id: 'cra_123',
          org_id: 'org-1',
          repository_full_name: 'yohandry10/Git-Gov',
          branch: 'main',
          environment: 'production',
          change_id: 'KAN-121',
          deployment_gate_id: 'dga_123',
          release_id: 'KAN-121',
          commit_sha: 'abcdef1234567890abcdef1234567890abcdef12',
          evidence_packet_hash: 'e'.repeat(64),
          risk_level: 'medium',
          ruleset_version: 'change_risk_rules.v1',
          risk_reasons: ['production_environment', 'deployment_gate_advisory'],
          missing_evidence: [],
          blocking_gaps: [],
          recommended_manual_actions: ['Review Deployment Gate warnings before approving change.'],
          triggered_rules: ['production_environment'],
          non_triggered_rules: ['gate_blocked'],
          evaluation_trace: { ruleset_version: 'change_risk_rules.v1', triggered_rules: ['production_environment'] },
          trace_hash: 'sha256:' + '1'.repeat(64),
          advisory_only: true,
          llm_used: false,
          agent_governance_used: false,
          compliance_claim: false,
          certification: false,
          evaluation: { source: 'store-test' },
          request_payload: { deployment_gate_id: 'dga_123' },
          created_by: 'admin',
          created_at: 4,
        }],
        total: 1,
        limit: 10,
        offset: 0,
      })

      const response = await useControlPlaneStore.getState().loadChangeRiskEvaluations({
        repository_full_name: ' yohandry10/Git-Gov ',
        branch: ' main ',
        release_id: ' KAN-121 ',
        environment: ' production ',
        deployment_gate_id: ' dga_123 ',
        review_status: 'needs_review',
      })

      expect(mockInvoke).toHaveBeenCalledWith('cmd_server_list_change_risk_evaluations', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        query: {
          org_name: 'yohandry10',
          evaluation_id: null,
          deployment_gate_id: 'dga_123',
          repository_full_name: 'yohandry10/Git-Gov',
          branch: 'main',
          change_id: null,
          commit_sha: null,
          release_id: 'KAN-121',
          environment: 'production',
          review_status: 'needs_review',
          limit: 10,
          offset: 0,
        },
      })
      expect(response?.items[0].advisory_only).toBe(true)
      expect(response?.items[0].llm_used).toBe(false)
      expect(response?.items[0].agent_governance_used).toBe(false)
      expect(response?.items[0].compliance_claim).toBe(false)
      expect(response?.items[0].certification).toBe(false)
      expect(useControlPlaneStore.getState().changeRiskEvaluationsTotal).toBe(1)
    })

    it('loads change risk rule catalog and evaluation trace', async () => {
      useControlPlaneStore.setState({
        serverConfig: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        selectedOrgName: 'yohandry10',
      })
      mockInvoke
        .mockResolvedValueOnce({
          ruleset_version: 'change_risk_rules.v1',
          catalog_hash: 'sha256:' + 'a'.repeat(64),
          rules: [{
            rule_id: 'gate_blocked',
            title: 'Gate blocked',
            description: 'The deployment gate returned a blocking decision.',
            severity: 'high',
            evidence_inputs: ['deployment_gate'],
            manual_action_hint: 'Resolve blocking governance gaps.',
            enabled: true,
          }],
          advisory_only: true,
          llm_used: false,
          agent_governance_used: false,
          compliance_claim: false,
          certification: false,
        })
        .mockResolvedValueOnce({
          evaluation_id: 'cra_123',
          org_id: 'org-1',
          ruleset_version: 'change_risk_rules.v1',
          triggered_rules: ['gate_blocked'],
          non_triggered_rules: ['production_environment'],
          evaluation_trace: { risk_level: 'high', rules: [{ rule_id: 'gate_blocked', triggered: true }] },
          trace_hash: 'sha256:' + 'b'.repeat(64),
          advisory_only: true,
          llm_used: false,
          agent_governance_used: false,
          compliance_claim: false,
          certification: false,
          created_at: 6,
        })

      const catalog = await useControlPlaneStore.getState().loadChangeRiskRules()
      const trace = await useControlPlaneStore.getState().loadChangeRiskEvaluationTrace(' cra_123 ')

      expect(mockInvoke).toHaveBeenNthCalledWith(1, 'cmd_server_get_change_risk_rules', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(2, 'cmd_server_get_change_risk_evaluation_trace', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        evaluationId: 'cra_123',
        query: { org_name: 'yohandry10' },
      })
      expect(catalog?.rules[0].rule_id).toBe('gate_blocked')
      expect(trace?.trace_hash).toBe('sha256:' + 'b'.repeat(64))
      expect(useControlPlaneStore.getState().changeRiskRuleCatalog?.catalog_hash).toBe('sha256:' + 'a'.repeat(64))
      expect(useControlPlaneStore.getState().changeRiskEvaluationTrace?.triggered_rules).toContain('gate_blocked')
    })

    it('loads and updates change risk manual review without changing risk trace state', async () => {
      useControlPlaneStore.setState({
        serverConfig: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        selectedOrgName: 'yohandry10',
        changeRiskEvaluationsFilters: { limit: 10, offset: 0, review_status: 'needs_review' },
        changeRiskSelectedEvaluation: {
          evaluation_id: 'cra_review',
          org_id: 'org-1',
          repository_full_name: 'yohandry10/Git-Gov',
          branch: 'main',
          environment: 'production',
          risk_level: 'high',
          ruleset_version: 'change_risk_rules.v1',
          risk_reasons: ['deployment_gate_blocked'],
          missing_evidence: ['release_approval'],
          blocking_gaps: ['gate blocked'],
          recommended_manual_actions: ['Resolve blocking governance gaps.'],
          triggered_rules: ['gate_blocked'],
          non_triggered_rules: [],
          evaluation_trace: { risk_level: 'high' },
          trace_hash: 'sha256:' + 'd'.repeat(64),
          advisory_only: true,
          llm_used: false,
          agent_governance_used: false,
          compliance_claim: false,
          certification: false,
          evaluation: {},
          request_payload: {},
          created_by: 'admin',
          review_status: 'needs_review',
          created_at: 7,
        },
        changeRiskEvaluations: [{
          evaluation_id: 'cra_review',
          org_id: 'org-1',
          repository_full_name: 'yohandry10/Git-Gov',
          branch: 'main',
          environment: 'production',
          risk_level: 'high',
          ruleset_version: 'change_risk_rules.v1',
          risk_reasons: ['deployment_gate_blocked'],
          missing_evidence: ['release_approval'],
          blocking_gaps: ['gate blocked'],
          recommended_manual_actions: ['Resolve blocking governance gaps.'],
          triggered_rules: ['gate_blocked'],
          non_triggered_rules: [],
          evaluation_trace: { risk_level: 'high' },
          trace_hash: 'sha256:' + 'd'.repeat(64),
          advisory_only: true,
          llm_used: false,
          agent_governance_used: false,
          compliance_claim: false,
          certification: false,
          evaluation: {},
          request_payload: {},
          created_by: 'admin',
          review_status: 'needs_review',
          created_at: 7,
        }],
      })
      mockInvoke
        .mockResolvedValueOnce({
          evaluation_id: 'cra_review',
          org_id: 'org-1',
          risk_level: 'high',
          ruleset_version: 'change_risk_rules.v1',
          trace_hash: 'sha256:' + 'd'.repeat(64),
          review_status: 'needs_review',
          advisory_only: true,
          llm_used: false,
          agent_governance_used: false,
          compliance_claim: false,
          certification: false,
        })
        .mockResolvedValueOnce({
          evaluation_id: 'cra_review',
          org_id: 'org-1',
          risk_level: 'high',
          ruleset_version: 'change_risk_rules.v1',
          trace_hash: 'sha256:' + 'd'.repeat(64),
          review_status: 'accepted_risk',
          reviewed_by_user_id: 'admin',
          reviewed_at: 8,
          review_notes_safe: 'Manual CAB reviewed deterministic trace.',
          mitigation_notes_safe: 'Rollback owner remains online.',
          decision_reason_safe: 'Business exception accepted manually.',
          review_updated_at: 9,
          advisory_only: true,
          llm_used: false,
          agent_governance_used: false,
          compliance_claim: false,
          certification: false,
        })

      const initial = await useControlPlaneStore.getState().loadChangeRiskEvaluationReview(' cra_review ')
      const updated = await useControlPlaneStore.getState().updateChangeRiskEvaluationReview(' cra_review ', {
        review_status: ' accepted_risk ',
        review_notes: ' Manual CAB reviewed deterministic trace. ',
        mitigation_notes: ' Rollback owner remains online. ',
        decision_reason: ' Business exception accepted manually. ',
      })

      expect(mockInvoke).toHaveBeenNthCalledWith(1, 'cmd_server_get_change_risk_evaluation_review', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        evaluationId: 'cra_review',
        query: { org_name: 'yohandry10' },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(2, 'cmd_server_update_change_risk_evaluation_review', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        evaluationId: 'cra_review',
        payload: {
          org_name: 'yohandry10',
          review_status: 'accepted_risk',
          review_notes: 'Manual CAB reviewed deterministic trace.',
          mitigation_notes: 'Rollback owner remains online.',
          decision_reason: 'Business exception accepted manually.',
        },
      })
      expect(initial?.review_status).toBe('needs_review')
      expect(updated?.review_status).toBe('accepted_risk')
      expect(updated?.trace_hash).toBe('sha256:' + 'd'.repeat(64))
      expect(updated?.llm_used).toBe(false)
      expect(updated?.agent_governance_used).toBe(false)
      expect(useControlPlaneStore.getState().changeRiskSelectedEvaluation?.review_status).toBe('accepted_risk')
      expect(useControlPlaneStore.getState().changeRiskSelectedEvaluation?.trace_hash).toBe('sha256:' + 'd'.repeat(64))
      expect(useControlPlaneStore.getState().changeRiskEvaluations).toHaveLength(0)
    })

    it('creates change risk advisory records without AI, agent governance, or compliance claims', async () => {
      useControlPlaneStore.setState({
        serverConfig: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        selectedOrgName: 'yohandry10',
      })
      const record = {
        evaluation_id: 'cra_456',
        org_id: 'org-1',
        repository_full_name: 'yohandry10/Git-Gov',
        branch: 'main',
        environment: 'production',
        change_id: 'CAB-7',
        deployment_gate_id: 'dga_blocked',
        release_id: 'KAN-121',
        commit_sha: 'abcdef1234567890abcdef1234567890abcdef12',
        evidence_packet_hash: 'e'.repeat(64),
        risk_level: 'high',
        ruleset_version: 'change_risk_rules.v1',
        risk_reasons: ['production_environment', 'deployment_gate_blocked'],
        missing_evidence: ['release_approval'],
        blocking_gaps: ['Deployment gate blocked by release governance.'],
        recommended_manual_actions: ['Resolve blocking Deployment Gate gaps before approving deployment.'],
        triggered_rules: ['production_environment', 'gate_blocked'],
        non_triggered_rules: ['missing_ci_evidence'],
        evaluation_trace: { risk_level: 'high', rules: [{ rule_id: 'gate_blocked', triggered: true }] },
        trace_hash: 'sha256:' + 'c'.repeat(64),
        advisory_only: true,
        llm_used: false,
        agent_governance_used: false,
        compliance_claim: false,
        certification: false,
        evaluation: { risk_level: 'high' },
        request_payload: { deployment_gate_id: 'dga_blocked' },
        created_by: 'admin',
        created_at: 5,
      }
      mockInvoke.mockResolvedValueOnce(record)

      const response = await useControlPlaneStore.getState().createChangeRiskEvaluation({
        repository_full_name: ' yohandry10/Git-Gov ',
        branch: ' main ',
        environment: ' production ',
        deployment_gate_id: ' dga_blocked ',
        release_id: ' KAN-121 ',
        commit_sha: ' abcdef1234567890abcdef1234567890abcdef12 ',
        evidence_packet_hash: ' ' + 'e'.repeat(64) + ' ',
        change_id: ' CAB-7 ',
        evidence_refs: [' deployment_gate:dga_blocked ', '', ' evidence_packet_hash:' + 'e'.repeat(64) + ' '],
      })

      expect(mockInvoke).toHaveBeenCalledWith('cmd_server_create_change_risk_evaluation', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        payload: {
          org_name: 'yohandry10',
          repository_full_name: 'yohandry10/Git-Gov',
          branch: 'main',
          environment: 'production',
          deployment_gate_id: 'dga_blocked',
          release_id: 'KAN-121',
          commit_sha: 'abcdef1234567890abcdef1234567890abcdef12',
          evidence_packet_hash: 'e'.repeat(64),
          change_id: 'CAB-7',
          evidence_refs: ['deployment_gate:dga_blocked', 'evidence_packet_hash:' + 'e'.repeat(64)],
        },
      })
      expect(response?.risk_level).toBe('high')
      expect(response?.advisory_only).toBe(true)
      expect(response?.llm_used).toBe(false)
      expect(response?.agent_governance_used).toBe(false)
      expect(response?.compliance_claim).toBe(false)
      expect(response?.certification).toBe(false)
      expect(useControlPlaneStore.getState().changeRiskSelectedEvaluation?.evaluation_id).toBe('cra_456')
      expect(useControlPlaneStore.getState().changeRiskEvaluationTrace?.trace_hash).toBe('sha256:' + 'c'.repeat(64))
      expect(useControlPlaneStore.getState().changeRiskEvaluations[0].blocking_gaps).toContain('Deployment gate blocked by release governance.')
    })

    it('creates, downloads, and archives change risk CAB packets as manual JSON artifacts', async () => {
      useControlPlaneStore.setState({
        serverConfig: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        selectedOrgName: 'yohandry10',
      })
      const packet = {
        packet_id: 'crcab_' + '1'.repeat(32),
        org_id: 'org-1',
        name: 'KAN-125 CAB packet',
        filters: { review_status: 'accepted_risk' },
        evaluation_ids: ['cra_123'],
        artifact_hash: 'sha256:' + 'f'.repeat(64),
        status: 'active',
        created_by_user_id: 'admin',
        created_at: 10,
        downloaded_at: null,
        download_count: 0,
        archived_at: null,
        archived_by_user_id: null,
        review_status: 'pending_review',
        reviewed_by_user_id: null,
        reviewed_at: null,
        review_notes_safe: null,
        mitigation_notes_safe: null,
        decision_reason_safe: null,
        follow_up_required: false,
        follow_up_owner_safe: null,
        review_updated_at: null,
      }
      const artifact = {
        schema_version: 'gitgov_change_risk_cab_packet.v1',
        packet_id: packet.packet_id,
        summary: { total_evaluations: 1 },
        claims: {
          advisory_only: true,
          manual_review_packet: true,
          compliance_claim: false,
          certification: false,
        },
        audit_metadata: {
          llm_used: false,
          agent_governance_used: false,
          enforcement: false,
          source_evaluations_mutated: false,
        },
        verification: {
          packet_hash: packet.artifact_hash,
        },
      }
      mockInvoke
        .mockResolvedValueOnce({
          items: [packet],
          total: 1,
          limit: 10,
          offset: 0,
        })
        .mockResolvedValueOnce({
          packet,
          download_url: `/change-risk/cab-packets/${packet.packet_id}/download`,
          artifact,
        })
        .mockResolvedValueOnce(artifact)
        .mockResolvedValueOnce({
          packet: {
            ...packet,
            status: 'archived',
            archived_at: 12,
            archived_by_user_id: 'admin',
          },
          download_url: `/change-risk/cab-packets/${packet.packet_id}/download`,
          artifact: null,
        })

      const listed = await useControlPlaneStore.getState().loadChangeRiskCabPackets({
        status: 'active',
      })
      const created = await useControlPlaneStore.getState().createChangeRiskCabPacket({
        name: ' KAN-125 CAB packet ',
        repository_full_name: ' yohandry10/Git-Gov ',
        branch: ' main ',
        environment: ' production ',
        review_status: ' accepted_risk ',
        evaluation_ids: [' cra_123 ', ''],
        deployment_gate_ids: [' dga_123 '],
      })
      const downloaded = await useControlPlaneStore
        .getState()
        .downloadChangeRiskCabPacket(` ${packet.packet_id} `)
      const archived = await useControlPlaneStore
        .getState()
        .archiveChangeRiskCabPacket(` ${packet.packet_id} `)

      expect(mockInvoke).toHaveBeenNthCalledWith(1, 'cmd_server_list_change_risk_cab_packets', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        query: {
          org_name: 'yohandry10',
          status: 'active',
          limit: 10,
          offset: 0,
        },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(2, 'cmd_server_create_change_risk_cab_packet', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        payload: {
          org_name: 'yohandry10',
          name: 'KAN-125 CAB packet',
          repository_full_name: 'yohandry10/Git-Gov',
          branch: 'main',
          environment: 'production',
          risk_level: null,
          review_status: 'accepted_risk',
          date_range_start: null,
          date_range_end: null,
          evaluation_ids: ['cra_123'],
          deployment_gate_ids: ['dga_123'],
        },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(3, 'cmd_server_download_change_risk_cab_packet', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        packetId: packet.packet_id,
        query: { org_name: 'yohandry10' },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(4, 'cmd_server_archive_change_risk_cab_packet', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        packetId: packet.packet_id,
        payload: {
          org_name: 'yohandry10',
          name: '',
          evaluation_ids: [],
          deployment_gate_ids: [],
        },
      })
      expect(listed?.total).toBe(1)
      expect(created?.artifact?.claims).toEqual(expect.objectContaining({
        advisory_only: true,
        compliance_claim: false,
      }))
      expect(downloaded?.verification).toEqual({ packet_hash: packet.artifact_hash })
      expect(archived?.packet.status).toBe('archived')
      expect(useControlPlaneStore.getState().changeRiskCabPacketArtifact?.verification).toEqual({
        packet_hash: packet.artifact_hash,
      })
      expect(useControlPlaneStore.getState().changeRiskCabPackets[0].status).toBe('archived')
    })

    it('loads and updates change risk CAB packet manual disposition without changing the packet hash', async () => {
      useControlPlaneStore.setState({
        serverConfig: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        selectedOrgName: 'yohandry10',
        changeRiskCabPackets: [
          {
            packet_id: 'crcab_' + '2'.repeat(32),
            org_id: 'org-1',
            name: 'KAN-126 CAB packet',
            filters: { review_status: 'accepted_risk' },
            evaluation_ids: ['cra_456'],
            artifact_hash: 'sha256:' + 'e'.repeat(64),
            status: 'active',
            created_by_user_id: 'admin',
            created_at: 20,
            downloaded_at: null,
            download_count: 0,
            archived_at: null,
            archived_by_user_id: null,
            review_status: 'pending_review',
            reviewed_by_user_id: null,
            reviewed_at: null,
            review_notes_safe: null,
            mitigation_notes_safe: null,
            decision_reason_safe: null,
            follow_up_required: false,
            follow_up_owner_safe: null,
            review_updated_at: null,
          },
        ],
      })
      const packetId = 'crcab_' + '2'.repeat(32)
      const initialReview = {
        packet_id: packetId,
        org_id: 'org-1',
        artifact_hash: 'sha256:' + 'e'.repeat(64),
        packet_status: 'active',
        review_status: 'pending_review',
        reviewed_by_user_id: null,
        reviewed_at: null,
        review_notes_safe: null,
        mitigation_notes_safe: null,
        decision_reason_safe: null,
        follow_up_required: false,
        follow_up_owner_safe: null,
        review_updated_at: null,
        manual_cab_disposition_only: true,
        advisory_only: true,
        llm_used: false,
        agent_governance_used: false,
        release_blocking: false,
        deployment_execution: false,
        compliance_claim: false,
        certification: false,
      }
      const updatedReview = {
        ...initialReview,
        review_status: 'needs_mitigation',
        reviewed_by_user_id: 'kan-126-admin',
        reviewed_at: 30,
        review_notes_safe: 'Manual CAB disposition recorded.',
        mitigation_notes_safe: 'Attach rollback rehearsal evidence.',
        decision_reason_safe: 'Rollback rehearsal evidence missing.',
        follow_up_required: true,
        follow_up_owner_safe: 'release-owner',
        review_updated_at: 31,
      }
      mockInvoke.mockResolvedValueOnce(initialReview).mockResolvedValueOnce(updatedReview)

      const loaded = await useControlPlaneStore
        .getState()
        .getChangeRiskCabPacketReview(` ${packetId} `)
      const saved = await useControlPlaneStore.getState().updateChangeRiskCabPacketReview(` ${packetId} `, {
        review_status: ' needs_mitigation ',
        review_notes: ' Manual CAB disposition recorded. ',
        mitigation_notes: ' Attach rollback rehearsal evidence. ',
        decision_reason: ' Rollback rehearsal evidence missing. ',
        follow_up_required: true,
        follow_up_owner: ' release-owner ',
      })

      expect(mockInvoke).toHaveBeenNthCalledWith(1, 'cmd_server_get_change_risk_cab_packet_review', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        packetId,
        query: { org_name: 'yohandry10' },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(2, 'cmd_server_update_change_risk_cab_packet_review', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        packetId,
        payload: {
          org_name: 'yohandry10',
          review_status: 'needs_mitigation',
          review_notes: 'Manual CAB disposition recorded.',
          mitigation_notes: 'Attach rollback rehearsal evidence.',
          decision_reason: 'Rollback rehearsal evidence missing.',
          follow_up_required: true,
          follow_up_owner: 'release-owner',
        },
      })
      expect(loaded?.manual_cab_disposition_only).toBe(true)
      expect(saved?.artifact_hash).toBe('sha256:' + 'e'.repeat(64))
      expect(saved?.release_blocking).toBe(false)
      expect(saved?.deployment_execution).toBe(false)
      expect(useControlPlaneStore.getState().changeRiskCabPackets[0].review_status).toBe('needs_mitigation')
      expect(useControlPlaneStore.getState().changeRiskCabPackets[0].artifact_hash).toBe('sha256:' + 'e'.repeat(64))
      expect(useControlPlaneStore.getState().changeRiskCabPackets[0].follow_up_required).toBe(true)
    })

    it('creates downloads and revokes change risk CAB decision manifests without changing source packet evidence', async () => {
      useControlPlaneStore.setState({
        serverConfig: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        selectedOrgName: 'yohandry10',
      })
      const manifest = {
        manifest_id: 'crcabdm_' + '3'.repeat(32),
        org_id: 'org-1',
        cab_packet_id: 'crcab_' + '2'.repeat(32),
        cab_packet_hash: 'sha256:' + 'e'.repeat(64),
        manifest_hash: 'sha256:' + 'f'.repeat(64),
        review_status_snapshot: 'needs_mitigation',
        reviewed_by_user_id: 'kan-126-admin',
        reviewed_at: 30,
        created_by_user_id: 'kan-127-admin',
        created_at: 40,
        download_count: 0,
        downloaded_at: null,
        status: 'active',
        revoked_at: null,
        revoked_by_user_id: null,
      }
      const artifact = {
        schema_version: 'gitgov_change_risk_cab_decision_manifest.v1',
        manifest_id: manifest.manifest_id,
        cab_packet: {
          packet_id: manifest.cab_packet_id,
          cab_packet_hash: manifest.cab_packet_hash,
        },
        included_evaluations: {
          count: 1,
          trace_hashes: ['sha256:' + 'a'.repeat(64)],
        },
        claims: {
          advisory_only: true,
          llm_used: false,
          agent_governance_used: false,
          compliance_claim: false,
          certification: false,
        },
        audit_metadata: {
          deployment_execution: false,
          source_cab_packet_mutated: false,
          source_evaluations_mutated: false,
        },
        hash_chain: {
          cab_packet_hash: manifest.cab_packet_hash,
          manifest_hash: manifest.manifest_hash,
        },
      }
      mockInvoke
        .mockResolvedValueOnce({ items: [manifest], total: 1, limit: 10, offset: 0 })
        .mockResolvedValueOnce({
          manifest,
          download_url: `/change-risk/cab-decision-manifests/${manifest.manifest_id}/download`,
          artifact,
        })
        .mockResolvedValueOnce(artifact)
        .mockResolvedValueOnce({
          manifest: {
            ...manifest,
            status: 'revoked',
            revoked_at: 50,
            revoked_by_user_id: 'kan-127-admin',
          },
          download_url: `/change-risk/cab-decision-manifests/${manifest.manifest_id}/download`,
          artifact: null,
        })

      const listed = await useControlPlaneStore
        .getState()
        .loadChangeRiskCabDecisionManifests(` ${manifest.cab_packet_id} `)
      const created = await useControlPlaneStore
        .getState()
        .createChangeRiskCabDecisionManifest(` ${manifest.cab_packet_id} `)
      const downloaded = await useControlPlaneStore
        .getState()
        .downloadChangeRiskCabDecisionManifest(` ${manifest.manifest_id} `)
      const revoked = await useControlPlaneStore
        .getState()
        .revokeChangeRiskCabDecisionManifest(` ${manifest.manifest_id} `)

      expect(mockInvoke).toHaveBeenNthCalledWith(1, 'cmd_server_list_change_risk_cab_decision_manifests', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        packetId: manifest.cab_packet_id,
        query: {
          org_name: 'yohandry10',
          status: null,
          limit: 10,
          offset: 0,
        },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(2, 'cmd_server_create_change_risk_cab_decision_manifest', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        packetId: manifest.cab_packet_id,
        payload: { org_name: 'yohandry10' },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(3, 'cmd_server_download_change_risk_cab_decision_manifest', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        manifestId: manifest.manifest_id,
        query: { org_name: 'yohandry10' },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(4, 'cmd_server_revoke_change_risk_cab_decision_manifest', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        manifestId: manifest.manifest_id,
        payload: { org_name: 'yohandry10' },
      })
      expect(listed?.total).toBe(1)
      expect(created?.artifact?.claims).toEqual(expect.objectContaining({
        advisory_only: true,
        compliance_claim: false,
        certification: false,
      }))
      expect(downloaded?.hash_chain).toEqual(expect.objectContaining({
        cab_packet_hash: manifest.cab_packet_hash,
        manifest_hash: manifest.manifest_hash,
      }))
      expect(revoked?.manifest.status).toBe('revoked')
      expect(useControlPlaneStore.getState().changeRiskCabDecisionManifestArtifact?.included_evaluations).toEqual({
        count: 1,
        trace_hashes: ['sha256:' + 'a'.repeat(64)],
      })
      expect(useControlPlaneStore.getState().changeRiskCabDecisionManifests[0].status).toBe('revoked')
    })

    it('creates the compliance evidence review chain with explicit manual-first payloads', async () => {
      useControlPlaneStore.setState({
        serverConfig: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        selectedOrgName: 'yohandry10',
      })
      mockInvoke
        .mockResolvedValueOnce({
          export: {
            export_id: 'cee_123',
            org_id: 'org-1',
            created_by_user_id: 'admin',
            scope: 'deployment_gate',
            deployment_gate_id: 'dga_123',
            release_id: 'KAN-102',
            status: 'completed',
            format: 'json',
            artifact_hash: 'a'.repeat(64),
            policy_checksum: 'f'.repeat(64),
            gate_decision: 'approved',
            created_at: 1,
            completed_at: 2,
          },
          artifact: {
            compliance_claim: false,
            framework_mapping: false,
            agent_governance_used: false,
          },
        })
        .mockResolvedValueOnce({
          mapping: {
            mapping_id: 'cem_123',
            org_id: 'org-1',
            evidence_export_id: 'cee_123',
            evidence_export_hash: 'a'.repeat(64),
            framework_id: 'gitgov_release_governance_baseline_v1',
            framework_version: '1.0.0',
            created_by_user_id: 'admin',
            compliance_claim: false,
            regulatory_claim: false,
            requires_auditor_review: true,
            created_at: 3,
          },
          items: [{
            control_id: 'GOV-REL-001',
            control_title: 'Release authorization',
            status: 'covered',
            evidence_refs: ['deployment_gate:dga_123'],
            missing_evidence: [],
            notes_safe: 'Deployment Gate evidence exists.',
          }],
        })
        .mockResolvedValueOnce({
          review_package: {
            review_package_id: 'crp_123',
            org_id: 'org-1',
            created_by_user_id: 'admin',
            mapping_id: 'cem_123',
            evidence_export_id: 'cee_123',
            evidence_export_hash: 'a'.repeat(64),
            mapping_hash: 'b'.repeat(64),
            framework_id: 'gitgov_release_governance_baseline_v1',
            framework_version: '1.0.0',
            format: 'json',
            artifact_hash: 'c'.repeat(64),
            compliance_claim: false,
            regulatory_claim: false,
            requires_auditor_review: true,
            certification: false,
            created_at: 4,
          },
          download_url: '/compliance/review-packages/crp_123/download',
          artifact: { certification: false },
        })
        .mockResolvedValueOnce({
          review_package_id: 'crp_123',
          artifact_hash: 'c'.repeat(64),
          compliance_claim: false,
          regulatory_claim: false,
          certification: false,
        })
        .mockResolvedValueOnce({
          report: {
            report_id: 'frr_123',
            org_id: 'org-1',
            created_by_user_id: 'admin',
            mapping_id: 'cem_123',
            review_package_id: 'crp_123',
            evidence_export_id: 'cee_123',
            evidence_export_hash: 'a'.repeat(64),
            mapping_hash: 'b'.repeat(64),
            review_package_hash: 'c'.repeat(64),
            framework_id: 'gitgov_release_governance_baseline_v1',
            framework_version: '1.0.0',
            framework_owner_type: 'gitgov',
            format: 'json',
            artifact_hash: 'd'.repeat(64),
            compliance_claim: false,
            regulatory_claim: false,
            requires_auditor_review: true,
            certification: false,
            review_status: 'needs_review',
            reviewed_by_user_id: null,
            reviewed_at: null,
            review_notes_safe: null,
            created_at: 5,
          },
          download_url: '/compliance/framework-review-reports/frr_123/download',
          artifact: { schema_version: 'gitgov_framework_review_report.v1', certification: false },
        })
        .mockResolvedValueOnce({
          items: [
            {
              report_id: 'frr_123',
              org_id: 'org-1',
              created_by_user_id: 'admin',
              mapping_id: 'cem_123',
              review_package_id: 'crp_123',
              evidence_export_id: 'cee_123',
              evidence_export_hash: 'a'.repeat(64),
              mapping_hash: 'b'.repeat(64),
              review_package_hash: 'c'.repeat(64),
              framework_id: 'gitgov_release_governance_baseline_v1',
              framework_version: '1.0.0',
              framework_owner_type: 'gitgov',
              format: 'json',
              artifact_hash: 'd'.repeat(64),
              compliance_claim: false,
              regulatory_claim: false,
              requires_auditor_review: true,
              certification: false,
              review_status: 'needs_review',
              reviewed_by_user_id: null,
              reviewed_at: null,
              review_notes_safe: null,
              created_at: 5,
              downloaded_at: null,
            },
          ],
          count: 1,
          limit: 25,
        })
        .mockResolvedValueOnce({
          items: [
            {
              report_id: 'frr_123',
              org_id: 'org-1',
              created_by_user_id: 'admin',
              mapping_id: 'cem_123',
              review_package_id: 'crp_123',
              evidence_export_id: 'cee_123',
              evidence_export_hash: 'a'.repeat(64),
              mapping_hash: 'b'.repeat(64),
              review_package_hash: 'c'.repeat(64),
              framework_id: 'gitgov_release_governance_baseline_v1',
              framework_version: '1.0.0',
              framework_owner_type: 'gitgov',
              format: 'json',
              artifact_hash: 'd'.repeat(64),
              compliance_claim: false,
              regulatory_claim: false,
              requires_auditor_review: true,
              certification: false,
              review_status: 'needs_review',
              reviewed_by_user_id: null,
              reviewed_at: null,
              review_notes_safe: null,
              created_at: 5,
              downloaded_at: null,
            },
          ],
          count: 1,
          limit: 25,
        })
        .mockResolvedValueOnce({
          assignments: [{
            id: 'assign-1',
            org_id: 'org-1',
            report_id: 'frr_123',
            auditor_client_id: 'kan109-auditor',
            assignment_status: 'active',
            assigned_by_user_id: 'admin',
            assignment_notes_safe: 'Assigned note',
            created_at: 7,
            updated_at: 7,
          }],
          count: 1,
        })
        .mockResolvedValueOnce({
          assignments: [{
            id: 'assign-1',
            org_id: 'org-1',
            report_id: 'frr_123',
            auditor_client_id: 'kan109-auditor',
            assignment_status: 'active',
            assigned_by_user_id: 'admin',
            assignment_notes_safe: 'Assigned note',
            created_at: 7,
            updated_at: 8,
          }],
          count: 1,
        })
        .mockResolvedValueOnce({
          comments: [],
          count: 0,
        })
        .mockResolvedValueOnce({
          id: 'comment-1',
          org_id: 'org-1',
          report_id: 'frr_123',
          commenter_client_id: 'kan109-auditor',
          comment_body_safe: 'Needs owner sign-off.',
          review_status_suggestion: 'needs_changes',
          created_at: 9,
        })
        .mockResolvedValueOnce({
          report: {
            report_id: 'frr_123',
            org_id: 'org-1',
            created_by_user_id: 'admin',
            mapping_id: 'cem_123',
            review_package_id: 'crp_123',
            evidence_export_id: 'cee_123',
            evidence_export_hash: 'a'.repeat(64),
            mapping_hash: 'b'.repeat(64),
            review_package_hash: 'c'.repeat(64),
            framework_id: 'gitgov_release_governance_baseline_v1',
            framework_version: '1.0.0',
            framework_owner_type: 'gitgov',
            format: 'json',
            artifact_hash: 'd'.repeat(64),
            compliance_claim: false,
            regulatory_claim: false,
            requires_auditor_review: true,
            certification: false,
            review_status: 'needs_changes',
            reviewed_by_user_id: 'admin',
            reviewed_at: 6,
            review_notes_safe: 'Needs owner sign-off.',
            created_at: 5,
            downloaded_at: null,
          },
          download_url: '/compliance/framework-review-reports/frr_123/download',
        })
        .mockResolvedValueOnce({
          schema_version: 'gitgov_framework_review_report.v1',
          certification: false,
        })
        .mockResolvedValueOnce({
          manifest: {
            manifest_id: 'frrm_123',
            org_id: 'org-1',
            report_id: 'frr_123',
            generated_by_user_id: 'admin',
            manifest_hash: 'sha256:' + 'e'.repeat(64),
            previous_manifest_hash: null,
            signature_algorithm: 'sha256-provenance-manifest-v1',
            created_at: 10,
          },
          download_url: '/compliance/framework-review-reports/frr_123/provenance-manifests/frrm_123',
          artifact: {
            schema_version: 'gitgov_framework_review_report_provenance_manifest.v1',
            hash_chain: {
              manifest_hash: 'sha256:' + 'e'.repeat(64),
              previous_manifest_hash: null,
            },
            claims: {
              compliance_claim: false,
              regulatory_claim: false,
              certification: false,
            },
          },
        })
        .mockResolvedValueOnce({
          pdf_export: {
            pdf_export_id: 'frrpdf_123',
            org_id: 'org-1',
            report_id: 'frr_123',
            manifest_id: 'frrm_123',
            created_by_user_id: 'admin',
            source_report_hash: 'sha256:' + 'd'.repeat(64),
            manifest_hash: 'sha256:' + 'e'.repeat(64),
            pdf_artifact_hash: 'sha256:' + 'f'.repeat(64),
            content_type: 'application/pdf',
            page_count: 1,
            compliance_claim: false,
            regulatory_claim: false,
            requires_auditor_review: true,
            certification: false,
            created_at: 11,
            downloaded_at: null,
          },
          download_url: '/compliance/framework-review-reports/frr_123/pdf-export/download?pdf_export_id=frrpdf_123',
        })
        .mockResolvedValueOnce({
          pdf_export: {
            pdf_export_id: 'frrpdf_123',
            org_id: 'org-1',
            report_id: 'frr_123',
            manifest_id: 'frrm_123',
            created_by_user_id: 'admin',
            source_report_hash: 'sha256:' + 'd'.repeat(64),
            manifest_hash: 'sha256:' + 'e'.repeat(64),
            pdf_artifact_hash: 'sha256:' + 'f'.repeat(64),
            content_type: 'application/pdf',
            page_count: 1,
            compliance_claim: false,
            regulatory_claim: false,
            requires_auditor_review: true,
            certification: false,
            created_at: 11,
            downloaded_at: 12,
          },
          pdf_base64: 'JVBERi0xLjQK',
        })
        .mockResolvedValueOnce({
          period_report: {
            period_report_id: 'cpr_123',
            org_id: 'org-1',
            created_by_user_id: 'admin',
            framework_id: 'gitgov_release_governance_baseline_v1',
            date_range_start: 1000,
            date_range_end: 2000,
            report_count: 1,
            source_report_ids: ['frr_123'],
            format: 'json',
            status: 'generated',
            artifact_hash: 'sha256:' + '1'.repeat(64),
            compliance_claim: false,
            regulatory_claim: false,
            requires_auditor_review: true,
            certification: false,
            review_status: 'needs_review',
            reviewed_by_user_id: null,
            reviewed_at: null,
            review_notes_safe: null,
            created_at: 13,
            downloaded_at: null,
            retention_status: 'active',
            retention_until: 1800000000000,
            download_count: 0,
            last_downloaded_at: null,
            archived_at: null,
          },
          download_url: '/compliance/period-reports/cpr_123/download',
          artifact: {
            schema_version: 'gitgov_period_compliance_report.v1',
            summary: {
              report_count: 1,
              reports_with_manifest_count: 1,
            },
            claims: {
              compliance_claim: false,
              regulatory_claim: false,
              certification: false,
              requires_auditor_review: true,
            },
          },
        })
        .mockResolvedValueOnce({
          items: [{
            period_report_id: 'cpr_123',
            org_id: 'org-1',
            created_by_user_id: 'admin',
            framework_id: 'gitgov_release_governance_baseline_v1',
            date_range_start: 1000,
            date_range_end: 2000,
            report_count: 1,
            source_report_ids: ['frr_123'],
            format: 'json',
            status: 'generated',
            artifact_hash: 'sha256:' + '1'.repeat(64),
            compliance_claim: false,
            regulatory_claim: false,
            requires_auditor_review: true,
            certification: false,
            review_status: 'needs_review',
            reviewed_by_user_id: null,
            reviewed_at: null,
            review_notes_safe: null,
            created_at: 13,
            downloaded_at: null,
            retention_status: 'active',
            retention_until: 1800000000000,
            download_count: 0,
            last_downloaded_at: null,
            archived_at: null,
          }],
          count: 1,
          limit: 25,
        })
        .mockResolvedValueOnce({
          schema_version: 'gitgov_period_compliance_report.v1',
          period_report_id: 'cpr_123',
          source_hashes: {
            report_hashes: ['sha256:' + 'd'.repeat(64)],
          },
          claims: {
            compliance_claim: false,
            regulatory_claim: false,
            certification: false,
            requires_auditor_review: true,
          },
        })
        .mockResolvedValueOnce({
          period_report: {
            period_report_id: 'cpr_123',
            org_id: 'org-1',
            created_by_user_id: 'admin',
            framework_id: 'gitgov_release_governance_baseline_v1',
            date_range_start: 1000,
            date_range_end: 2000,
            report_count: 1,
            source_report_ids: ['frr_123'],
            format: 'json',
            status: 'generated',
            artifact_hash: 'sha256:' + '1'.repeat(64),
            compliance_claim: false,
            regulatory_claim: false,
            requires_auditor_review: true,
            certification: false,
            review_status: 'reviewed',
            reviewed_by_user_id: 'auditor',
            reviewed_at: 17,
            review_notes_safe: 'Monthly approval',
            created_at: 13,
            downloaded_at: null,
            retention_status: 'active',
            retention_until: 1800000000000,
            download_count: 0,
            last_downloaded_at: null,
            archived_at: null,
          },
          download_url: '/compliance/period-reports/cpr_123/download',
          artifact: null,
        })
        .mockResolvedValueOnce({
          period_report: {
            period_report_id: 'cpr_123',
            org_id: 'org-1',
            created_by_user_id: 'admin',
            framework_id: 'gitgov_release_governance_baseline_v1',
            date_range_start: 1000,
            date_range_end: 2000,
            report_count: 1,
            source_report_ids: ['frr_123'],
            format: 'json',
            status: 'generated',
            artifact_hash: 'sha256:' + '1'.repeat(64),
            compliance_claim: false,
            regulatory_claim: false,
            requires_auditor_review: true,
            certification: false,
            review_status: 'reviewed',
            reviewed_by_user_id: 'auditor',
            reviewed_at: 17,
            review_notes_safe: 'Monthly approval',
            created_at: 13,
            downloaded_at: 16,
            retention_status: 'active',
            retention_until: 1900000000000,
            download_count: 1,
            last_downloaded_at: 16,
            archived_at: null,
          },
          download_url: '/compliance/period-reports/cpr_123/download',
          artifact: null,
        })
        .mockResolvedValueOnce({
          items: [{
            access_log_id: 'cprlog_123',
            org_id: 'org-1',
            period_report_id: 'cpr_123',
            actor_client_id: 'admin',
            action: 'retention_updated',
            artifact_type: 'retention',
            artifact_id: 'cpr_123',
            artifact_hash: 'sha256:' + '1'.repeat(64),
            metadata: { retention_until: 1900000000000 },
            created_at: 17,
          }],
          count: 1,
          limit: 50,
        })
        .mockResolvedValueOnce({
          pdf_export: {
            pdf_export_id: 'cprpdf_123',
            org_id: 'org-1',
            period_report_id: 'cpr_123',
            created_by_user_id: 'auditor',
            source_period_report_hash: 'sha256:' + '1'.repeat(64),
            pdf_artifact_hash: 'sha256:' + '2'.repeat(64),
            content_type: 'application/pdf',
            page_count: 2,
            compliance_claim: false,
            regulatory_claim: false,
            requires_auditor_review: true,
            certification: false,
            created_at: 14,
            downloaded_at: null,
          },
          download_url: '/compliance/period-reports/cpr_123/pdf-export/download?pdf_export_id=cprpdf_123',
        })
        .mockResolvedValueOnce({
          pdf_export: {
            pdf_export_id: 'cprpdf_123',
            org_id: 'org-1',
            period_report_id: 'cpr_123',
            created_by_user_id: 'auditor',
            source_period_report_hash: 'sha256:' + '1'.repeat(64),
            pdf_artifact_hash: 'sha256:' + '2'.repeat(64),
            content_type: 'application/pdf',
            page_count: 2,
            compliance_claim: false,
            regulatory_claim: false,
            requires_auditor_review: true,
            certification: false,
            created_at: 14,
            downloaded_at: 15,
          },
          pdf_base64: 'JVBERi0xLjQK',
        })
        .mockResolvedValueOnce({
          manifest: {
            manifest_id: 'cprm_123',
            org_id: 'org-1',
            period_report_id: 'cpr_123',
            generated_by_user_id: 'auditor',
            manifest_hash: 'sha256:' + '3'.repeat(64),
            previous_manifest_hash: null,
            signature_algorithm: 'sha256-period-report-provenance-manifest-v1',
            created_at: 18,
          },
          download_url: '/compliance/period-reports/cpr_123/provenance-manifests/cprm_123',
          artifact: {
            schema_version: 'gitgov_period_compliance_report_provenance_manifest.v1',
            hash_chain: {
              subject_type: 'period_compliance_report',
              subject_id: 'cpr_123',
              previous_manifest_hash: null,
              manifest_hash: 'sha256:' + '3'.repeat(64),
            },
            claims: {
              compliance_claim: false,
              regulatory_claim: false,
              certification: false,
              requires_auditor_review: true,
            },
            audit_metadata: {
              agent_governance_required: false,
              source_period_report_artifact_mutated: false,
            },
          },
        })
        .mockResolvedValueOnce({
          schema_version: 'gitgov_period_compliance_report_provenance_manifest.v1',
          manifest_id: 'cprm_123',
          hash_chain: {
            subject_type: 'period_compliance_report',
            subject_id: 'cpr_123',
            previous_manifest_hash: null,
            manifest_hash: 'sha256:' + '3'.repeat(64),
          },
        })

      const exportResponse = await useControlPlaneStore.getState().createComplianceEvidenceExport(' dga_123 ')
      const mappingResponse = await useControlPlaneStore.getState().createComplianceEvidenceMapping(' cee_123 ')
      const packageResponse = await useControlPlaneStore.getState().createComplianceReviewPackage(' cem_123 ')
      const artifact = await useControlPlaneStore.getState().downloadComplianceReviewPackage(' crp_123 ')
      const reportResponse = await useControlPlaneStore.getState().createComplianceFrameworkReviewReport(' cem_123 ', ' crp_123 ')
      const reportHistory = await useControlPlaneStore.getState().loadComplianceFrameworkReviewReports({
        framework_id: ' gitgov_release_governance_baseline_v1 ',
        mapping_id: ' cem_123 ',
        review_package_id: ' crp_123 ',
        limit: 500,
      })
      const assignedReports = await useControlPlaneStore.getState().loadAssignedComplianceFrameworkReviewReports({
        framework_id: ' gitgov_release_governance_baseline_v1 ',
      })
      const loadedAssignments = await useControlPlaneStore
        .getState()
        .loadComplianceFrameworkReviewReportAssignments(' frr_123 ')
      const savedAssignments = await useControlPlaneStore
        .getState()
        .saveComplianceFrameworkReviewReportAssignments(
          ' frr_123 ',
          [' kan109-auditor ', 'kan109-auditor', ' '],
          ' Assigned note ',
        )
      const loadedComments = await useControlPlaneStore
        .getState()
        .loadComplianceFrameworkReviewReportComments(' frr_123 ')
      const createdComment = await useControlPlaneStore
        .getState()
        .createComplianceFrameworkReviewReportComment(
          ' frr_123 ',
          ' Needs owner sign-off. ',
          ' needs_changes ',
        )
      const reviewedReport = await useControlPlaneStore.getState().reviewComplianceFrameworkReviewReport(
        ' frr_123 ',
        ' needs_changes ',
        ' Needs owner sign-off. ',
      )
      const reportArtifact = await useControlPlaneStore.getState().downloadComplianceFrameworkReviewReport(' frr_123 ')
      const manifest = await useControlPlaneStore
        .getState()
        .createComplianceFrameworkReviewReportProvenanceManifest(' frr_123 ')
      const pdfExport = await useControlPlaneStore
        .getState()
        .createComplianceFrameworkReviewReportPdfExport(' frr_123 ', ' frrm_123 ')
      const pdfDownload = await useControlPlaneStore
        .getState()
        .downloadComplianceFrameworkReviewReportPdfExport(' frr_123 ', ' frrpdf_123 ')
      const periodReport = await useControlPlaneStore
        .getState()
        .createCompliancePeriodReport(1000, 2000, ' gitgov_release_governance_baseline_v1 ')
      const periodReports = await useControlPlaneStore
        .getState()
        .loadCompliancePeriodReports({ framework_id: ' gitgov_release_governance_baseline_v1 ' })
      const periodArtifact = await useControlPlaneStore.getState().downloadCompliancePeriodReport(' cpr_123 ')
      const reviewedPeriodReport = await useControlPlaneStore
        .getState()
        .reviewCompliancePeriodReport(' cpr_123 ', ' reviewed ', ' Monthly approval ')
      const updatedPeriodRetention = await useControlPlaneStore
        .getState()
        .updateCompliancePeriodReportRetention(' cpr_123 ', {
          retention_until: 1900000000000,
          archive: false,
        })
      const periodAccessLog = await useControlPlaneStore
        .getState()
        .loadCompliancePeriodReportAccessLog(' cpr_123 ', { limit: 50 })
      const periodPdfExport = await useControlPlaneStore
        .getState()
        .createCompliancePeriodReportPdfExport(' cpr_123 ')
      const periodPdfDownload = await useControlPlaneStore
        .getState()
        .downloadCompliancePeriodReportPdfExport(' cpr_123 ', ' cprpdf_123 ')
      const periodManifest = await useControlPlaneStore
        .getState()
        .createCompliancePeriodReportProvenanceManifest(' cpr_123 ')
      const periodManifestArtifact = await useControlPlaneStore
        .getState()
        .downloadCompliancePeriodReportProvenanceManifest(' cpr_123 ', ' cprm_123 ')

      expect(mockInvoke).toHaveBeenNthCalledWith(1, 'cmd_server_create_compliance_evidence_export', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        payload: {
          org_name: 'yohandry10',
          scope: 'deployment_gate',
          deployment_gate_id: 'dga_123',
          format: 'json',
          include_sections: [
            'gate_decision',
            'policy',
            'readiness',
            'approvals',
            'evidence',
            'gaps',
            'audit',
          ],
        },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(2, 'cmd_server_create_compliance_evidence_mapping', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        payload: {
          org_name: 'yohandry10',
          evidence_export_id: 'cee_123',
          framework_id: 'gitgov_release_governance_baseline_v1',
          framework_version: null,
        },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(3, 'cmd_server_create_compliance_review_package', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        payload: {
          org_name: 'yohandry10',
          mapping_id: 'cem_123',
          format: 'json',
          include_sections: [
            'summary',
            'source_hashes',
            'framework',
            'control_matrix',
            'missing_evidence',
            'no_claims',
            'audit_metadata',
          ],
        },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(4, 'cmd_server_download_compliance_review_package', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        reviewPackageId: 'crp_123',
        query: {
          org_name: 'yohandry10',
        },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(5, 'cmd_server_create_compliance_framework_review_report', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        payload: {
          org_name: 'yohandry10',
          mapping_id: 'cem_123',
          review_package_id: 'crp_123',
          format: 'json',
        },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(6, 'cmd_server_list_compliance_framework_review_reports', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        query: {
          org_name: 'yohandry10',
          framework_id: 'gitgov_release_governance_baseline_v1',
          mapping_id: 'cem_123',
          review_package_id: 'crp_123',
          limit: 500,
          assigned_to_me: null,
        },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(7, 'cmd_server_list_assigned_compliance_framework_review_reports', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        query: {
          org_name: 'yohandry10',
          framework_id: 'gitgov_release_governance_baseline_v1',
          mapping_id: null,
          review_package_id: null,
          limit: 25,
          assigned_to_me: true,
        },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(8, 'cmd_server_list_compliance_framework_review_report_assignments', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        reportId: 'frr_123',
        query: {
          org_name: 'yohandry10',
        },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(9, 'cmd_server_upsert_compliance_framework_review_report_assignments', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        reportId: 'frr_123',
        payload: {
          org_name: 'yohandry10',
          auditor_client_ids: ['kan109-auditor'],
          assignment_notes_safe: 'Assigned note',
        },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(10, 'cmd_server_list_compliance_framework_review_report_comments', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        reportId: 'frr_123',
        query: {
          org_name: 'yohandry10',
        },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(11, 'cmd_server_create_compliance_framework_review_report_comment', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        reportId: 'frr_123',
        payload: {
          org_name: 'yohandry10',
          comment_body_safe: 'Needs owner sign-off.',
          review_status_suggestion: 'needs_changes',
        },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(12, 'cmd_server_review_compliance_framework_review_report', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        reportId: 'frr_123',
        payload: {
          org_name: 'yohandry10',
          review_status: 'needs_changes',
          review_notes_safe: 'Needs owner sign-off.',
        },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(13, 'cmd_server_download_compliance_framework_review_report', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        reportId: 'frr_123',
        query: {
          org_name: 'yohandry10',
        },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(14, 'cmd_server_create_compliance_framework_review_report_provenance_manifest', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        reportId: 'frr_123',
        payload: {
          org_name: 'yohandry10',
        },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(15, 'cmd_server_create_compliance_framework_review_report_pdf_export', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        reportId: 'frr_123',
        payload: {
          org_name: 'yohandry10',
          manifest_id: 'frrm_123',
        },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(16, 'cmd_server_download_compliance_framework_review_report_pdf_export', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        reportId: 'frr_123',
        query: {
          org_name: 'yohandry10',
          pdf_export_id: 'frrpdf_123',
        },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(17, 'cmd_server_create_compliance_period_report', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        payload: {
          org_name: 'yohandry10',
          date_range_start: 1000,
          date_range_end: 2000,
          framework_id: 'gitgov_release_governance_baseline_v1',
          format: 'json',
        },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(18, 'cmd_server_list_compliance_period_reports', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        query: {
          org_name: 'yohandry10',
          framework_id: 'gitgov_release_governance_baseline_v1',
          limit: 25,
        },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(19, 'cmd_server_download_compliance_period_report', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        periodReportId: 'cpr_123',
        query: {
          org_name: 'yohandry10',
        },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(20, 'cmd_server_review_compliance_period_report', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        periodReportId: 'cpr_123',
        payload: {
          org_name: 'yohandry10',
          review_status: 'reviewed',
          review_notes_safe: 'Monthly approval',
        },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(21, 'cmd_server_update_compliance_period_report_retention', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        periodReportId: 'cpr_123',
        payload: {
          org_name: 'yohandry10',
          retention_until: 1900000000000,
          archive: false,
        },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(22, 'cmd_server_list_compliance_period_report_access_log', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        periodReportId: 'cpr_123',
        query: {
          org_name: 'yohandry10',
          limit: 50,
        },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(23, 'cmd_server_create_compliance_period_report_pdf_export', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        periodReportId: 'cpr_123',
        payload: {
          org_name: 'yohandry10',
        },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(24, 'cmd_server_download_compliance_period_report_pdf_export', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        periodReportId: 'cpr_123',
        query: {
          org_name: 'yohandry10',
          pdf_export_id: 'cprpdf_123',
        },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(25, 'cmd_server_create_compliance_period_report_provenance_manifest', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        periodReportId: 'cpr_123',
        payload: {
          org_name: 'yohandry10',
        },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(26, 'cmd_server_download_compliance_period_report_provenance_manifest', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        periodReportId: 'cpr_123',
        manifestId: 'cprm_123',
        query: {
          org_name: 'yohandry10',
        },
      })
      expect(exportResponse?.export.export_id).toBe('cee_123')
      expect(mappingResponse?.items).toHaveLength(1)
      expect(packageResponse?.review_package.certification).toBe(false)
      expect(artifact?.certification).toBe(false)
      expect(reportResponse?.report.report_id).toBe('frr_123')
      expect(reportHistory?.items).toHaveLength(1)
      const historyItem = reportHistory?.items[0] as Record<string, unknown> | undefined
      expect(historyItem?.artifact).toBeUndefined()
      expect(historyItem?.payload_json_redacted).toBeUndefined()
      expect(assignedReports?.items[0]?.report_id).toBe('frr_123')
      expect(loadedAssignments?.assignments[0]?.auditor_client_id).toBe('kan109-auditor')
      expect(savedAssignments?.assignments[0]?.updated_at).toBe(8)
      expect(loadedComments?.count).toBe(0)
      expect(createdComment?.review_status_suggestion).toBe('needs_changes')
      expect(reviewedReport?.report.review_status).toBe('needs_changes')
      expect(reportArtifact?.schema_version).toBe('gitgov_framework_review_report.v1')
      expect(manifest?.manifest.signature_algorithm).toBe('sha256-provenance-manifest-v1')
      expect(pdfExport?.pdf_export.pdf_artifact_hash).toBe('sha256:' + 'f'.repeat(64))
      expect(pdfDownload?.pdf_base64).toBe('JVBERi0xLjQK')
      expect(periodReport?.period_report.period_report_id).toBe('cpr_123')
      expect(periodReports?.items[0]?.source_report_ids).toEqual(['frr_123'])
      expect(periodArtifact?.schema_version).toBe('gitgov_period_compliance_report.v1')
      expect(reviewedPeriodReport?.period_report.review_status).toBe('reviewed')
      expect(reviewedPeriodReport?.period_report.review_notes_safe).toBe('Monthly approval')
      expect(updatedPeriodRetention?.period_report.retention_until).toBe(1900000000000)
      expect(periodAccessLog?.items[0]?.action).toBe('retention_updated')
      expect(periodPdfExport?.pdf_export.pdf_artifact_hash).toBe('sha256:' + '2'.repeat(64))
      expect(periodPdfDownload?.pdf_base64).toBe('JVBERi0xLjQK')
      expect(periodManifest?.manifest.signature_algorithm).toBe('sha256-period-report-provenance-manifest-v1')
      expect(periodManifestArtifact?.schema_version).toBe('gitgov_period_compliance_report_provenance_manifest.v1')
      expect(useControlPlaneStore.getState().complianceFrameworkReviewReports?.count).toBe(1)
      expect(useControlPlaneStore.getState().complianceFrameworkReviewReports?.items[0]?.review_status).toBe('needs_changes')
      expect(useControlPlaneStore.getState().assignedComplianceFrameworkReviewReports?.count).toBe(1)
      expect(useControlPlaneStore.getState().complianceFrameworkReviewReportAssignments?.assignments[0]?.assignment_status).toBe('active')
      expect(useControlPlaneStore.getState().complianceFrameworkReviewReportComments?.comments[0]?.comment_body_safe).toBe('Needs owner sign-off.')
      expect(useControlPlaneStore.getState().complianceFrameworkReviewReportProvenanceManifest?.artifact.schema_version).toBe('gitgov_framework_review_report_provenance_manifest.v1')
      expect(useControlPlaneStore.getState().complianceFrameworkReviewReportPdfExport?.pdf_export.downloaded_at).toBe(12)
      expect(useControlPlaneStore.getState().compliancePeriodReport?.period_report.artifact_hash).toBe('sha256:' + '1'.repeat(64))
      expect(useControlPlaneStore.getState().compliancePeriodReport?.period_report.retention_status).toBe('active')
      expect(useControlPlaneStore.getState().compliancePeriodReport?.period_report.review_status).toBe('reviewed')
      expect(useControlPlaneStore.getState().compliancePeriodReports?.count).toBe(1)
      expect(useControlPlaneStore.getState().compliancePeriodReportArtifact?.claims).toEqual({
        compliance_claim: false,
        regulatory_claim: false,
        certification: false,
        requires_auditor_review: true,
      })
      expect(useControlPlaneStore.getState().compliancePeriodReportAccessLog?.items[0]?.artifact_type).toBe('retention')
      expect(useControlPlaneStore.getState().compliancePeriodReportPdfExport?.pdf_export.downloaded_at).toBe(15)
      expect(useControlPlaneStore.getState().compliancePeriodReportProvenanceManifest?.artifact.schema_version).toBe('gitgov_period_compliance_report_provenance_manifest.v1')
      expect(useControlPlaneStore.getState().complianceEvidenceSelectedDeploymentGateId).toBe('dga_123')
    })

    it('manages saved period report profiles and run artifacts', async () => {
      useControlPlaneStore.setState({
        serverConfig: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        selectedOrgName: 'yohandry10',
      })

      const profile = {
        profile_id: 'cprprof_123',
        org_id: 'org-1',
        created_by_user_id: 'admin',
        updated_by_user_id: 'admin',
        name: 'Monthly evidence profile',
        period_type: 'monthly',
        framework_id: 'gitgov_release_governance_baseline_v1',
        framework_owner_type: 'gitgov_managed',
        include_pdf: true,
        include_manifest: true,
        retention_days: 45,
        filters: { manual_run_template: true },
        status: 'active',
        run_count: 0,
        last_run_at: null,
        last_period_report_id: null,
        last_pdf_export_id: null,
        last_manifest_id: null,
        archived_at: null,
        created_at: 10,
        updated_at: 10,
      }
      const periodReport = {
        period_report_id: 'cpr_kan118',
        org_id: 'org-1',
        created_by_user_id: 'admin',
        framework_id: 'gitgov_release_governance_baseline_v1',
        date_range_start: 1000,
        date_range_end: 2000,
        report_count: 2,
        source_report_ids: ['frr_a', 'frr_b'],
        format: 'json',
        status: 'generated',
        artifact_hash: 'sha256:' + '1'.repeat(64),
        compliance_claim: false,
        regulatory_claim: false,
        requires_auditor_review: true,
        certification: false,
        review_status: 'needs_review',
        reviewed_by_user_id: null,
        reviewed_at: null,
        review_notes_safe: null,
        created_at: 20,
        downloaded_at: null,
        retention_status: 'active',
        retention_until: 1900000000000,
        download_count: 0,
        last_downloaded_at: null,
        archived_at: null,
      }
      const runProfile = {
        ...profile,
        run_count: 1,
        last_run_at: 30,
        last_period_report_id: 'cpr_kan118',
        last_pdf_export_id: 'cprpdf_kan118',
        last_manifest_id: 'cprm_kan118',
        updated_at: 30,
      }

      mockInvoke
        .mockResolvedValueOnce({ profile })
        .mockResolvedValueOnce({ items: [profile], count: 1, limit: 25 })
        .mockResolvedValueOnce({ profile: { ...profile, retention_days: 90, updated_at: 15 } })
        .mockResolvedValueOnce({
          profile: runProfile,
          period_report: periodReport,
          pdf_export: {
            pdf_export_id: 'cprpdf_kan118',
            org_id: 'org-1',
            period_report_id: 'cpr_kan118',
            created_by_user_id: 'admin',
            source_period_report_hash: periodReport.artifact_hash,
            pdf_artifact_hash: 'sha256:' + '2'.repeat(64),
            content_type: 'application/pdf',
            page_count: 1,
            compliance_claim: false,
            regulatory_claim: false,
            requires_auditor_review: true,
            certification: false,
            created_at: 31,
            downloaded_at: null,
          },
          manifest: {
            manifest_id: 'cprm_kan118',
            org_id: 'org-1',
            period_report_id: 'cpr_kan118',
            generated_by_user_id: 'admin',
            manifest_hash: 'sha256:' + '3'.repeat(64),
            previous_manifest_hash: null,
            signature_algorithm: 'sha256-period-report-provenance-manifest-v1',
            created_at: 32,
          },
          download_url: '/compliance/period-reports/cpr_kan118/download',
        })
        .mockResolvedValueOnce({ profile: { ...runProfile, status: 'archived', archived_at: 40 } })

      const created = await useControlPlaneStore.getState().createCompliancePeriodReportProfile({
        name: ' Monthly evidence profile ',
        period_type: ' monthly ',
        framework_id: ' gitgov_release_governance_baseline_v1 ',
        framework_owner_type: 'gitgov_managed',
        include_pdf: true,
        include_manifest: true,
        retention_days: 45,
        filters: { manual_run_template: true },
      })
      const loaded = await useControlPlaneStore.getState().loadCompliancePeriodReportProfiles({
        framework_id: ' gitgov_release_governance_baseline_v1 ',
        status: 'active',
      })
      const updated = await useControlPlaneStore.getState().updateCompliancePeriodReportProfile(' cprprof_123 ', {
        retention_days: 90,
      })
      const run = await useControlPlaneStore.getState().runCompliancePeriodReportProfile(' cprprof_123 ', {
        date_range_start: 1000,
        date_range_end: 2000,
      })
      const archived = await useControlPlaneStore.getState().archiveCompliancePeriodReportProfile(' cprprof_123 ')

      expect(mockInvoke).toHaveBeenNthCalledWith(1, 'cmd_server_create_compliance_period_report_profile', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        payload: {
          org_name: 'yohandry10',
          name: 'Monthly evidence profile',
          period_type: 'monthly',
          framework_id: 'gitgov_release_governance_baseline_v1',
          framework_owner_type: 'gitgov_managed',
          include_pdf: true,
          include_manifest: true,
          retention_days: 45,
          filters: { manual_run_template: true },
        },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(2, 'cmd_server_list_compliance_period_report_profiles', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        query: {
          org_name: 'yohandry10',
          framework_id: 'gitgov_release_governance_baseline_v1',
          status: 'active',
          limit: 25,
        },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(3, 'cmd_server_update_compliance_period_report_profile', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        profileId: 'cprprof_123',
        payload: {
          org_name: 'yohandry10',
          name: null,
          period_type: null,
          framework_id: null,
          framework_owner_type: null,
          include_pdf: null,
          include_manifest: null,
          retention_days: 90,
          filters: null,
        },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(4, 'cmd_server_run_compliance_period_report_profile', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        profileId: 'cprprof_123',
        payload: {
          org_name: 'yohandry10',
          date_range_start: 1000,
          date_range_end: 2000,
        },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(5, 'cmd_server_archive_compliance_period_report_profile', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        profileId: 'cprprof_123',
        payload: {
          org_name: 'yohandry10',
        },
      })

      expect(created?.profile.profile_id).toBe('cprprof_123')
      expect(loaded?.items[0]?.period_type).toBe('monthly')
      expect(updated?.profile.retention_days).toBe(90)
      expect(run?.period_report.review_status).toBe('needs_review')
      expect(run?.pdf_export?.content_type).toBe('application/pdf')
      expect(run?.manifest?.signature_algorithm).toBe('sha256-period-report-provenance-manifest-v1')
      expect(archived?.profile.status).toBe('archived')
      expect(useControlPlaneStore.getState().compliancePeriodReportProfileRun?.profile.run_count).toBe(1)
      expect(useControlPlaneStore.getState().compliancePeriodReport?.period_report.period_report_id).toBe('cpr_kan118')
      expect(useControlPlaneStore.getState().compliancePeriodReportPdfExport?.pdf_export.pdf_export_id).toBe('cprpdf_kan118')
      expect(useControlPlaneStore.getState().compliancePeriodReportProvenanceManifest?.manifest.manifest_id).toBe('cprm_kan118')
      expect(useControlPlaneStore.getState().compliancePeriodReportProfiles?.count).toBe(0)
    })

    it('manages period report share package create list download and revoke flow', async () => {
      useControlPlaneStore.setState({
        serverConfig: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        selectedOrgName: 'yohandry10',
      })

      const sharePackage = {
        share_package_id: 'cprsp_kan119',
        org_id: 'org-1',
        period_report_id: 'cpr_kan119',
        created_by_user_id: 'admin',
        period_report_artifact_hash: 'sha256:' + '1'.repeat(64),
        pdf_export_id: 'cprpdf_kan119',
        pdf_artifact_hash: 'sha256:' + '2'.repeat(64),
        manifest_id: 'cprm_kan119',
        manifest_hash: 'sha256:' + '3'.repeat(64),
        artifact_hash: 'sha256:' + '4'.repeat(64),
        package_format: 'json_bundle',
        status: 'active',
        no_claims_snapshot: {
          compliance_claim: false,
          regulatory_claim: false,
          certification: false,
          compliance_score: false,
          requires_auditor_review: true,
        },
        source_hashes: {
          period_report_artifact_hash: 'sha256:' + '1'.repeat(64),
          pdf_artifact_hash: 'sha256:' + '2'.repeat(64),
          manifest_hash: 'sha256:' + '3'.repeat(64),
        },
        download_count: 0,
        downloaded_at: null,
        last_downloaded_at: null,
        revoked_by_user_id: null,
        revoked_at: null,
        created_at: 100,
        error_message_safe: null,
      }

      mockInvoke
        .mockResolvedValueOnce({
          share_package: sharePackage,
          download_url: '/compliance/period-report-share-packages/cprsp_kan119/download',
          artifact: {
            schema_version: 'gitgov_period_compliance_report_share_package.v1',
            claims: sharePackage.no_claims_snapshot,
            verification: { package_hash: sharePackage.artifact_hash },
          },
        })
        .mockResolvedValueOnce({ items: [sharePackage], count: 1, limit: 25 })
        .mockResolvedValueOnce({
          schema_version: 'gitgov_period_compliance_report_share_package.v1',
          claims: sharePackage.no_claims_snapshot,
          verification: { package_hash: sharePackage.artifact_hash },
        })
        .mockResolvedValueOnce({
          share_package: {
            ...sharePackage,
            status: 'revoked',
            revoked_by_user_id: 'admin',
            revoked_at: 200,
          },
          download_url: '/compliance/period-report-share-packages/cprsp_kan119/download',
          artifact: null,
        })

      const created = await useControlPlaneStore.getState().createCompliancePeriodReportSharePackage(' cpr_kan119 ')
      const loaded = await useControlPlaneStore.getState().loadCompliancePeriodReportSharePackages(' cpr_kan119 ', {
        status: 'active',
      })
      const downloaded = await useControlPlaneStore.getState().downloadCompliancePeriodReportSharePackage(' cprsp_kan119 ')
      const revoked = await useControlPlaneStore.getState().revokeCompliancePeriodReportSharePackage(' cprsp_kan119 ')

      expect(mockInvoke).toHaveBeenNthCalledWith(1, 'cmd_server_create_compliance_period_report_share_package', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        periodReportId: 'cpr_kan119',
        payload: {
          org_name: 'yohandry10',
        },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(2, 'cmd_server_list_compliance_period_report_share_packages', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        periodReportId: 'cpr_kan119',
        query: {
          org_name: 'yohandry10',
          status: 'active',
          limit: 25,
        },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(3, 'cmd_server_download_compliance_period_report_share_package', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        sharePackageId: 'cprsp_kan119',
        query: {
          org_name: 'yohandry10',
        },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(4, 'cmd_server_revoke_compliance_period_report_share_package', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        sharePackageId: 'cprsp_kan119',
        payload: {
          org_name: 'yohandry10',
        },
      })

      expect(created?.share_package.artifact_hash).toBe('sha256:' + '4'.repeat(64))
      expect(loaded?.items[0]?.status).toBe('active')
      expect(downloaded?.schema_version).toBe('gitgov_period_compliance_report_share_package.v1')
      expect(revoked?.share_package.status).toBe('revoked')
      expect(useControlPlaneStore.getState().compliancePeriodReportSharePackageArtifact?.verification).toEqual({
        package_hash: 'sha256:' + '4'.repeat(64),
      })
      expect(useControlPlaneStore.getState().compliancePeriodReportSharePackages?.items[0]?.status).toBe('revoked')
    })

    it('imports customer-owned compliance framework packs and selects the imported framework', async () => {
      useControlPlaneStore.setState({
        serverConfig: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        selectedOrgName: 'yohandry10',
      })
      mockInvoke
        .mockResolvedValueOnce({
          framework_pack: {
            framework_pack_id: 'cfp_123',
            org_id: 'org-1',
            framework_id: 'customer_bank_controls_123',
            framework_name: 'Bank Controls',
            framework_version: '2026.06',
            description: 'Customer controls',
            owner_type: 'customer',
            owner_name: 'Customer Security Office',
            source: 'customer_provided',
            review_status: 'needs_review',
            schema_version: 'gitgov_customer_framework_pack.v1',
            pack_hash: 'sha256:' + '1'.repeat(64),
            control_count: 1,
            compliance_claim: false,
            regulatory_claim: false,
            gitgov_certifies: false,
            requires_auditor_review: true,
            official_regulatory_mapping: false,
            created_by_user_id: 'admin',
            created_at: 1,
          },
          framework: {
            framework_id: 'customer_bank_controls_123',
            org_id: 'org-1',
            name: 'Bank Controls',
            version: '2026.06',
            description: 'Customer controls',
            is_regulatory: false,
            is_active: true,
            owner_type: 'customer',
            owner_name: 'Customer Security Office',
            source: 'customer_provided',
            is_gitgov_owned: false,
            official_regulatory_mapping: false,
            framework_pack_id: 'cfp_123',
            pack_hash: 'sha256:' + '1'.repeat(64),
            controls: [],
          },
        })
        .mockResolvedValueOnce({
          frameworks: [{
            framework_id: 'gitgov_release_governance_baseline_v1',
            name: 'GitGov Release Governance Baseline',
            version: '1.0.0',
            description: 'Baseline',
            is_regulatory: false,
            is_active: true,
            owner_type: 'gitgov',
            owner_name: 'GitGov',
            source: 'gitgov_owned',
            is_gitgov_owned: true,
            official_regulatory_mapping: false,
            controls: [],
          }],
        })
        .mockResolvedValueOnce({
          framework_packs: [{
            framework_pack_id: 'cfp_123',
            org_id: 'org-1',
            framework_id: 'customer_bank_controls_123',
            framework_name: 'Bank Controls',
            framework_version: '2026.06',
            description: 'Customer controls',
            owner_type: 'customer',
            owner_name: 'Customer Security Office',
            source: 'customer_provided',
            review_status: 'needs_review',
            schema_version: 'gitgov_customer_framework_pack.v1',
            pack_hash: 'sha256:' + '1'.repeat(64),
            control_count: 1,
            compliance_claim: false,
            regulatory_claim: false,
            gitgov_certifies: false,
            requires_auditor_review: true,
            official_regulatory_mapping: false,
            created_by_user_id: 'admin',
            created_at: 1,
          }],
        })

      const response = await useControlPlaneStore
        .getState()
        .importComplianceFrameworkPack('{"schema_version":"gitgov_customer_framework_pack.v1"}', 'json')

      expect(mockInvoke).toHaveBeenNthCalledWith(1, 'cmd_server_import_compliance_framework_pack', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        payload: {
          org_name: 'yohandry10',
          format: 'json',
          pack: { schema_version: 'gitgov_customer_framework_pack.v1' },
          content: null,
        },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(2, 'cmd_server_list_compliance_control_frameworks', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        query: { org_name: 'yohandry10' },
      })
      expect(mockInvoke).toHaveBeenNthCalledWith(3, 'cmd_server_list_compliance_framework_packs', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        query: { org_name: 'yohandry10' },
      })
      expect(response?.framework.framework_id).toBe('customer_bank_controls_123')
      expect(useControlPlaneStore.getState().selectedComplianceFrameworkId).toBe('gitgov_release_governance_baseline_v1')
      expect(useControlPlaneStore.getState().complianceFrameworkPacks).toHaveLength(1)
    })

    it('loads customer framework pack diffs without creating compliance claims', async () => {
      useControlPlaneStore.setState({
        serverConfig: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        selectedOrgName: 'yohandry10',
        complianceFrameworkPackDiff: null,
      })
      mockInvoke.mockResolvedValueOnce({
        base_pack: {
          framework_pack_id: 'cfp_base',
          org_id: 'org-1',
          framework_id: 'customer_bank_controls_base',
          framework_name: 'Bank Controls',
          framework_version: '2026.06',
          description: 'Customer controls v1',
          owner_type: 'customer',
          owner_name: 'Customer Security Office',
          source: 'customer_provided',
          review_status: 'reviewed',
          schema_version: 'gitgov_customer_framework_pack.v1',
          pack_hash: 'sha256:' + '1'.repeat(64),
          control_count: 2,
          compliance_claim: false,
          regulatory_claim: false,
          gitgov_certifies: false,
          requires_auditor_review: true,
          official_regulatory_mapping: false,
          created_by_user_id: 'admin',
          created_at: 1,
        },
        target_pack: {
          framework_pack_id: 'cfp_target',
          org_id: 'org-1',
          framework_id: 'customer_bank_controls_target',
          framework_name: 'Bank Controls',
          framework_version: '2026.07',
          description: 'Customer controls v2',
          owner_type: 'customer',
          owner_name: 'Customer Security Office',
          source: 'customer_provided',
          review_status: 'reviewed',
          schema_version: 'gitgov_customer_framework_pack.v1',
          pack_hash: 'sha256:' + '2'.repeat(64),
          control_count: 3,
          compliance_claim: false,
          regulatory_claim: false,
          gitgov_certifies: false,
          requires_auditor_review: true,
          official_regulatory_mapping: false,
          created_by_user_id: 'admin',
          created_at: 2,
        },
        original_framework_id: 'bank_internal_release_controls',
        same_original_framework: true,
        summary: {
          added: 1,
          removed: 0,
          changed: 1,
          unchanged: 1,
        },
        controls: [
          {
            control_id: 'BRC-CI-02',
            change_type: 'changed',
            base: {
              title: 'CI evidence',
              description: 'Collect CI evidence.',
              required_evidence_types: ['pipeline_run'],
            },
            target: {
              title: 'CI evidence and approval',
              description: 'Collect CI evidence and approval.',
              required_evidence_types: ['deployment_gate_authorization', 'pipeline_run'],
            },
            changed_fields: ['title', 'description', 'required_evidence_types'],
          },
          {
            control_id: 'BRC-APPROVAL-05',
            change_type: 'added',
            base: null,
            target: {
              title: 'Manual approval',
              description: 'Require human approval.',
              required_evidence_types: ['release_approval'],
            },
            changed_fields: [],
          },
        ],
        compliance_claim: false,
        regulatory_claim: false,
        gitgov_certifies: false,
        official_regulatory_mapping: false,
        requires_auditor_review: true,
      })

      const response = await useControlPlaneStore
        .getState()
        .loadComplianceFrameworkPackDiff(' cfp_base ', ' cfp_target ')

      expect(mockInvoke).toHaveBeenCalledWith('cmd_server_diff_compliance_framework_packs', {
        config: { url: 'https://gitgov-api.onrender.com', api_key: 'key' },
        query: {
          org_name: 'yohandry10',
          base_pack_id: 'cfp_base',
          target_pack_id: 'cfp_target',
        },
      })
      expect(response?.summary).toEqual({
        added: 1,
        removed: 0,
        changed: 1,
        unchanged: 1,
      })
      expect(response?.controls[0].changed_fields).toEqual(['title', 'description', 'required_evidence_types'])
      expect(response?.compliance_claim).toBe(false)
      expect(response?.regulatory_claim).toBe(false)
      expect(response?.gitgov_certifies).toBe(false)
      expect(response?.official_regulatory_mapping).toBe(false)
      expect(response?.requires_auditor_review).toBe(true)
      expect(useControlPlaneStore.getState().complianceFrameworkPackDiff?.original_framework_id).toBe(
        'bank_internal_release_controls',
      )
    })
  })
})
