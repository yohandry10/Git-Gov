import { render, screen, waitFor } from '@testing-library/react'
import type { RepoValidation } from '@/lib/types'
import type {
  DeploymentGateAuthorizationRecord,
  ServerConfig,
} from '@/store/useControlPlaneStore/types'
import type { NativeTerminalGitContext } from '@/components/cli/terminalGitContext'
import { buildTerminalGovernanceTarget } from '@/components/cli/terminalGovernanceContext'
import { TerminalBranchGateStatusBadge } from '@/components/cli/TerminalBranchGateStatusBadge'
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
