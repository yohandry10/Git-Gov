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
  })
})
