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
      complianceEvidenceSelectedDeploymentGateId: null,
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
      expect(useControlPlaneStore.getState().complianceFrameworkReviewReports?.count).toBe(1)
      expect(useControlPlaneStore.getState().complianceFrameworkReviewReports?.items[0]?.review_status).toBe('needs_changes')
      expect(useControlPlaneStore.getState().assignedComplianceFrameworkReviewReports?.count).toBe(1)
      expect(useControlPlaneStore.getState().complianceFrameworkReviewReportAssignments?.assignments[0]?.assignment_status).toBe('active')
      expect(useControlPlaneStore.getState().complianceFrameworkReviewReportComments?.comments[0]?.comment_body_safe).toBe('Needs owner sign-off.')
      expect(useControlPlaneStore.getState().complianceFrameworkReviewReportProvenanceManifest?.artifact.schema_version).toBe('gitgov_framework_review_report_provenance_manifest.v1')
      expect(useControlPlaneStore.getState().complianceFrameworkReviewReportPdfExport?.pdf_export.downloaded_at).toBe(12)
      expect(useControlPlaneStore.getState().complianceEvidenceSelectedDeploymentGateId).toBe('dga_123')
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
