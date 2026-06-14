import { render, screen } from '@testing-library/react'
import { DeploymentGateHistoryPanel } from '@/components/control_plane/DeploymentGateHistoryPanel'
import { useControlPlaneStore } from '@/store/useControlPlaneStore'
import type { DeploymentGateAuthorizationRecord } from '@/store/useControlPlaneStore/types'

const loadDeploymentGateAuthorizations = vi.fn().mockResolvedValue(null)

function authorization(overrides: Partial<DeploymentGateAuthorizationRecord> = {}): DeploymentGateAuthorizationRecord {
  return {
    id: 'row-1',
    authorization_id: 'dga_breakglass',
    org_id: 'org-1',
    release_id: 'KAN-87',
    repository_full_name: 'yohandry10/Git-Gov',
    branch: 'main',
    target_sha: 'abcdef1234567890abcdef1234567890abcdef12',
    environment: 'production',
    deployer: 'github-actions',
    ticket_id: 'KAN-87',
    evidence_packet_hash: 'e'.repeat(64),
    evidence_packet_uri: '/evidence/packets/tickets/KAN-87',
    decision: 'break_glass',
    approved: true,
    blocking: true,
    would_block: true,
    reason: 'Break-glass deployment authorized: production rollback required.',
    blocked_by: ['No valid release approval found for this release.'],
    warnings: [],
    policy_checksum: 'f'.repeat(64),
    break_glass_eligible: true,
    break_glass_used: true,
    break_glass_reason: 'Production incident INC-2026-0614 requires immediate rollback.',
    break_glass_authorized_by: 'incident.commander@example.com',
    break_glass_expires_at: Date.UTC(2026, 5, 14, 4, 0, 0),
    break_glass_approval_id: 'dgbga_1234567890abcdef',
    break_glass_approval_hash: 'abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890',
    evaluation: {
      status: 'blocked',
      policy_satisfied: false,
      blocking: true,
      would_block: true,
      valid_approval_count: 0,
      required_approval_count: 1,
      policy: {
        mode: 'approval-required',
        environment: 'production',
        approval_required: true,
        enforcement: 'blocking',
        policy_applies: true,
        quorum_enabled: false,
        quorum_rules: [],
      },
      approvals: [],
      issues: ['No valid release approval found for this release.'],
      next_steps: [],
    },
    governance_decision: {
      consumer_type: 'deployment_gate',
      decision: 'requires_approval',
      agent_governance_used: false,
    },
    details: {},
    request_payload: {},
    requested_by: 'deploy-bot',
    created_at: Date.UTC(2026, 5, 14, 3, 0, 0),
    ...overrides,
  }
}

describe('DeploymentGateHistoryPanel', () => {
  beforeEach(() => {
    loadDeploymentGateAuthorizations.mockClear()
    useControlPlaneStore.setState({
      selectedOrgName: 'yohandry10',
      enterpriseAdoptionProfile: {
        customer_name: 'GitGov',
        repository_full_name: 'yohandry10/Git-Gov',
        default_branch: 'main',
        jira_project_key: 'KAN',
        policy_preset: 'moderate',
        providers: ['github'],
        modules: ['formal-approval'],
      },
      jiraCoverageFilters: {
        repo_full_name: 'yohandry10/Git-Gov',
        branch: 'main',
        hours: 720,
      },
      deploymentGateAuthorizations: [authorization()],
      deploymentGateAuthorizationsTotal: 1,
      deploymentGateAuthorizationsUpdatedAt: Date.UTC(2026, 5, 14, 3, 5, 0),
      isDeploymentGateAuthorizationsLoading: false,
      displayTimezone: 'UTC',
      loadDeploymentGateAuthorizations,
    })
  })

  it('renders break-glass deployment authorization evidence explicitly', () => {
    render(<DeploymentGateHistoryPanel />)

    expect(screen.getByText('break_glass')).toBeInTheDocument()
    expect(screen.getByText('break-glass used')).toBeInTheDocument()
    expect(screen.getByText('Break-glass authorization')).toBeInTheDocument()
    expect(screen.getByText(/INC-2026-0614/)).toBeInTheDocument()
    expect(screen.getByText('incident.commander@example.com')).toBeInTheDocument()
    expect(screen.getByText('pre-approved')).toBeInTheDocument()
    expect(screen.getByText('dgbga_1234567890abcdef')).toBeInTheDocument()
    expect(screen.getByText('Would block:')).toBeInTheDocument()
    expect(screen.getByText('shared: requires_approval')).toBeInTheDocument()
    expect(screen.getByText('agent not used')).toBeInTheDocument()
    expect(screen.getByText('Mode:')).toBeInTheDocument()
    expect(screen.getByText('manual-first')).toBeInTheDocument()
  })
})
