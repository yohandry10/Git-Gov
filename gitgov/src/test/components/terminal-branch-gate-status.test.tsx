import { act, fireEvent, render, screen, waitFor } from '@testing-library/react'
import type { RepoValidation } from '@/lib/types'
import type {
  DeploymentGateAuthorizationListResponse,
  DeploymentGateAuthorizationRecord,
  ServerConfig,
} from '@/store/useControlPlaneStore/types'
import type { NativeTerminalGitContext } from '@/components/cli/terminalGitContext'
import { buildTerminalGovernanceTarget } from '@/components/cli/terminalGovernanceContext'
import { TerminalBranchGateStatusBadge } from '@/components/cli/TerminalBranchGateStatusBadge'
import { TerminalGovernanceContextPanel } from '@/components/cli/TerminalGovernanceContextPanel'
import {
  summarizeTerminalBranchGateStatus,
  terminalBranchGateInitialStatus,
} from '@/components/cli/terminalBranchGateStatus'

const mockInvoke = vi.hoisted(() => vi.fn())

vi.mock('@/lib/tauri', () => ({
  tauriInvoke: (...args: unknown[]) => mockInvoke(...args),
  parseCommandError: (error: string) => {
    try {
      const parsed = JSON.parse(error)
      return {
        code: parsed.code || 'UNKNOWN',
        message: parsed.message || error,
      }
    } catch {
      return {
        code: 'UNKNOWN',
        message: error,
      }
    }
  },
}))

const serverConfig: ServerConfig = {
  url: 'https://gitgov-api.example.test',
  api_key: 'test-key',
}

const gitContext: NativeTerminalGitContext = {
  cwd: 'C:/work/customer-secret-path/GitGov',
  is_git_repo: true,
  is_detached: false,
  repo_name: 'GitGov',
  branch: 'main',
  commit_short: 'abc1234',
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

function authorization(overrides: Partial<DeploymentGateAuthorizationRecord> = {}): DeploymentGateAuthorizationRecord {
  return {
    id: 'row-1',
    authorization_id: 'dga_1',
    org_id: 'org-1',
    release_id: 'KAN-140',
    repository_full_name: 'yohandry10/Git-Gov',
    branch: 'main',
    target_sha: 'abcdef1234567890abcdef1234567890abcdef12',
    environment: 'production',
    deployer: 'github-actions',
    ticket_id: 'KAN-140',
    evidence_packet_hash: 'e'.repeat(64),
    evidence_packet_uri: '/evidence/packets/tickets/KAN-140',
    decision: 'approved',
    approved: true,
    blocking: false,
    would_block: false,
    reason: 'Latest deployment authorization is advisory and approved.',
    blocked_by: [],
    warnings: [],
    policy_checksum: 'f'.repeat(64),
    break_glass_eligible: false,
    break_glass_used: false,
    evaluation: {
      status: 'approved',
      policy_satisfied: true,
      blocking: false,
      would_block: false,
      valid_approval_count: 1,
      required_approval_count: 1,
      policy: {
        mode: 'approval-required',
        environment: 'production',
        approval_required: true,
        enforcement: 'advisory',
        policy_applies: true,
        quorum_enabled: false,
        quorum_rules: [],
      },
      approvals: [],
      issues: [],
      next_steps: [],
    },
    governance_decision: {
      consumer_type: 'deployment_gate',
      decision: 'approved',
      agent_governance_used: false,
    },
    details: {},
    request_payload: {},
    requested_by: 'deploy-bot',
    created_at: Date.UTC(2026, 5, 16, 4, 0, 0),
    ...overrides,
  }
}

describe('native terminal branch gate status advisory', () => {
  beforeEach(() => {
    mockInvoke.mockReset()
  })

  it('keeps pending and unmapped terminal contexts visually quiet', () => {
    const pending = buildTerminalGovernanceTarget(null, null, 'main')
    expect(terminalBranchGateInitialStatus(pending, serverConfig)).toMatchObject({
      label: 'Gate...',
      visible: false,
    })

    const nonGit = buildTerminalGovernanceTarget({ ...gitContext, is_git_repo: false }, validation, 'main')
    expect(terminalBranchGateInitialStatus(nonGit, serverConfig)).toMatchObject({
      label: 'Gate n/a',
      visible: false,
    })
  })

  it('makes Control Plane absence explicit without calling it', () => {
    const target = buildTerminalGovernanceTarget(gitContext, validation, 'main')
    const status = terminalBranchGateInitialStatus(target, null)

    expect(status).toMatchObject({
      label: 'Gate n/a',
      tone: 'muted',
      visible: true,
    })
    expect(status.title).toContain('Advisory only')
    expect(status.title).toContain('Does not block')
  })

  it('summarizes approved evidence as ready while preserving non-blocking language', () => {
    const status = summarizeTerminalBranchGateStatus(authorization())

    expect(status).toMatchObject({
      label: 'Gate ready',
      tone: 'ready',
      visible: true,
    })
    expect(status.title).toContain('Advisory only')
    expect(status.title).toContain('Does not block terminal commands')
  })

  it('surfaces gate evidence that would block or lacks evidence as manual review', () => {
    expect(
      summarizeTerminalBranchGateStatus(
        authorization({
          decision: 'blocked',
          approved: false,
          blocking: true,
          would_block: true,
          reason: 'Missing release approval.',
        }),
      ),
    ).toMatchObject({
      label: 'Gate review',
      tone: 'review',
    })

    expect(
      summarizeTerminalBranchGateStatus(
        authorization({
          approved: true,
          governance_decision: {
            consumer_type: 'deployment_gate',
            decision: 'insufficient_evidence',
          },
          reason: 'The gate is advisory but evidence is incomplete.',
        }),
      ),
    ).toMatchObject({
      label: 'Gate review',
      tone: 'review',
    })
  })

  it('renders a compact badge and loads the latest gate for the detected repo and branch', async () => {
    mockInvoke.mockResolvedValue({
      items: [authorization()],
      total: 1,
      limit: 1,
      offset: 0,
    })

    render(
      <TerminalBranchGateStatusBadge
        context={gitContext}
        validation={validation}
        currentBranch="main"
        serverConfig={serverConfig}
        selectedOrgName="yohandry10"
      />,
    )

    expect(screen.getByText('Gate...')).toBeInTheDocument()

    await waitFor(() => expect(screen.getByText('Gate ready')).toBeInTheDocument())

    expect(mockInvoke).toHaveBeenCalledWith('cmd_server_list_deployment_gate_authorizations', {
      config: serverConfig,
      query: {
        org_name: 'yohandry10',
        repository_full_name: 'yohandry10/Git-Gov',
        branch: 'main',
        limit: 1,
        offset: 0,
      },
    })
    expect(screen.getByLabelText(/Advisory only/)).toHaveTextContent('Gate ready')
    expect(screen.getByLabelText(/Advisory only/).getAttribute('aria-label')).not.toContain('customer-secret-path')
  })

  it('opens the existing read-only governance context from the compact badge', async () => {
    const onOpenContext = vi.fn()
    mockInvoke.mockResolvedValue({
      items: [authorization()],
      total: 1,
      limit: 1,
      offset: 0,
    })

    render(
      <TerminalBranchGateStatusBadge
        context={gitContext}
        validation={validation}
        currentBranch="main"
        serverConfig={serverConfig}
        selectedOrgName="yohandry10"
        onOpenContext={onOpenContext}
      />,
    )

    await waitFor(() => expect(screen.getByText('Gate ready')).toBeInTheDocument())
    fireEvent.click(screen.getByRole('button', { name: /Open read-only governance context/ }))

    expect(onOpenContext).toHaveBeenCalledTimes(1)
    expect(mockInvoke).toHaveBeenCalledTimes(1)
  })

  it('loads existing read-only gate, risk, and executive context when externally opened', async () => {
    const onOpenChange = vi.fn()
    mockInvoke.mockImplementation((command: string) => {
      if (command === 'cmd_server_list_deployment_gate_authorizations') {
        return Promise.resolve({
          items: [authorization()],
          total: 1,
          limit: 1,
          offset: 0,
        })
      }
      if (command === 'cmd_server_list_change_risk_evaluations') {
        return Promise.resolve({
          items: [
            {
              evaluation_id: 'cra_1',
              risk_level: 'medium',
              review_status: 'needs_review',
            },
          ],
          total: 1,
          limit: 1,
          offset: 0,
        })
      }
      if (command === 'cmd_server_get_multi_repo_executive_governance') {
        return Promise.resolve({
          repositories: [
            {
              repository_full_name: 'yohandry10/Git-Gov',
              posture: 'review',
            },
          ],
          total: 1,
          limit: 1,
          offset: 0,
        })
      }
      return Promise.reject(new Error(`Unexpected command ${command}`))
    })

    render(
      <TerminalGovernanceContextPanel
        context={gitContext}
        validation={validation}
        currentBranch="main"
        serverConfig={serverConfig}
        selectedOrgName="yohandry10"
        connectionStatus="connected"
        isOpen
        onOpenChange={onOpenChange}
      />,
    )

    expect(screen.getByText('Governance context')).toBeInTheDocument()
    await waitFor(() => expect(screen.getByText('approved')).toBeInTheDocument())
    expect(screen.getByText('medium')).toBeInTheDocument()
    expect(screen.getByText('review')).toBeInTheDocument()

    expect(mockInvoke).toHaveBeenCalledWith('cmd_server_list_deployment_gate_authorizations', {
      config: serverConfig,
      query: {
        org_name: 'yohandry10',
        repository_full_name: 'yohandry10/Git-Gov',
        branch: 'main',
        limit: 1,
        offset: 0,
      },
    })
    expect(mockInvoke).toHaveBeenCalledWith('cmd_server_list_change_risk_evaluations', expect.any(Object))
    expect(mockInvoke).toHaveBeenCalledWith('cmd_server_get_multi_repo_executive_governance', expect.any(Object))
    expect(JSON.stringify(mockInvoke.mock.calls)).not.toContain('customer-secret-path')

    fireEvent.click(screen.getByRole('button', { name: /Context/ }))
    expect(onOpenChange).toHaveBeenCalledWith(false)
  })

  it('does not show a previously loaded governance snapshot after the org changes', async () => {
    mockInvoke.mockImplementation((command: string, payload?: { query?: { org_name?: string } }) => {
      if (command === 'cmd_server_list_deployment_gate_authorizations') {
        return Promise.resolve({
          items: [
            payload?.query?.org_name === 'enterprise-b'
              ? authorization({
                  authorization_id: 'dga_enterprise_b',
                  decision: 'blocked',
                  approved: false,
                  blocking: true,
                  would_block: true,
                  reason: 'Enterprise B needs manual approval.',
                })
              : authorization({
                  authorization_id: 'dga_enterprise_a',
                  decision: 'approved',
                  approved: true,
                  blocking: false,
                  would_block: false,
                }),
          ],
          total: 1,
          limit: 1,
          offset: 0,
        })
      }
      if (command === 'cmd_server_list_change_risk_evaluations') {
        return Promise.resolve({ items: [], total: 0, limit: 1, offset: 0 })
      }
      if (command === 'cmd_server_get_multi_repo_executive_governance') {
        return Promise.resolve({ repositories: [], total: 0, limit: 1, offset: 0 })
      }
      return Promise.reject(new Error(`Unexpected command ${command}`))
    })

    const { rerender } = render(
      <TerminalGovernanceContextPanel
        context={gitContext}
        validation={validation}
        currentBranch="main"
        serverConfig={serverConfig}
        selectedOrgName="enterprise-a"
        connectionStatus="connected"
        isOpen
      />,
    )

    await waitFor(() => expect(screen.getByText('approved')).toBeInTheDocument())

    rerender(
      <TerminalGovernanceContextPanel
        context={gitContext}
        validation={validation}
        currentBranch="main"
        serverConfig={serverConfig}
        selectedOrgName="enterprise-b"
        connectionStatus="connected"
        isOpen
      />,
    )

    expect(screen.queryByText('approved')).not.toBeInTheDocument()
    await waitFor(() => expect(screen.getByText('blocked')).toBeInTheDocument())
    expect(mockInvoke).toHaveBeenCalledWith('cmd_server_list_deployment_gate_authorizations', {
      config: serverConfig,
      query: expect.objectContaining({
        org_name: 'enterprise-b',
        repository_full_name: 'yohandry10/Git-Gov',
        branch: 'main',
      }),
    })
  })

  it('ignores out-of-order governance context responses from a stale org', async () => {
    let resolveEnterpriseA: ((response: DeploymentGateAuthorizationListResponse) => void) | null = null
    const enterpriseAGate = new Promise<DeploymentGateAuthorizationListResponse>((resolve) => {
      resolveEnterpriseA = resolve
    })

    mockInvoke.mockImplementation((command: string, payload?: { query?: { org_name?: string } }) => {
      if (command === 'cmd_server_list_deployment_gate_authorizations') {
        if (payload?.query?.org_name === 'enterprise-a') {
          return enterpriseAGate
        }
        return Promise.resolve({
          items: [
            authorization({
              authorization_id: 'dga_enterprise_b',
              decision: 'blocked',
              approved: false,
              blocking: true,
              would_block: true,
              reason: 'Enterprise B needs manual approval.',
            }),
          ],
          total: 1,
          limit: 1,
          offset: 0,
        })
      }
      if (command === 'cmd_server_list_change_risk_evaluations') {
        return Promise.resolve({ items: [], total: 0, limit: 1, offset: 0 })
      }
      if (command === 'cmd_server_get_multi_repo_executive_governance') {
        return Promise.resolve({ repositories: [], total: 0, limit: 1, offset: 0 })
      }
      return Promise.reject(new Error(`Unexpected command ${command}`))
    })

    const { rerender } = render(
      <TerminalGovernanceContextPanel
        context={gitContext}
        validation={validation}
        currentBranch="main"
        serverConfig={serverConfig}
        selectedOrgName="enterprise-a"
        connectionStatus="connected"
        isOpen
      />,
    )

    rerender(
      <TerminalGovernanceContextPanel
        context={gitContext}
        validation={validation}
        currentBranch="main"
        serverConfig={serverConfig}
        selectedOrgName="enterprise-b"
        connectionStatus="connected"
        isOpen
      />,
    )

    await waitFor(() => expect(screen.getByText('blocked')).toBeInTheDocument())

    await act(async () => {
      resolveEnterpriseA?.({
        items: [
          authorization({
            authorization_id: 'dga_enterprise_a',
            decision: 'approved',
            approved: true,
            blocking: false,
            would_block: false,
          }),
        ],
        total: 1,
        limit: 1,
        offset: 0,
      })
      await Promise.resolve()
    })

    expect(screen.getByText('blocked')).toBeInTheDocument()
    expect(screen.queryByText('approved')).not.toBeInTheDocument()
  })

  it('renders no badge and performs no API call outside a mapped Git repository', () => {
    render(
      <TerminalBranchGateStatusBadge
        context={{ ...gitContext, is_git_repo: false }}
        validation={validation}
        currentBranch="main"
        serverConfig={serverConfig}
        selectedOrgName="yohandry10"
      />,
    )

    expect(screen.queryByText(/Gate/)).not.toBeInTheDocument()
    expect(mockInvoke).not.toHaveBeenCalled()
  })
})
