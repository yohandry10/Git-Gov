import { render, screen } from '@testing-library/react'
import { MultiRepoExecutiveGovernancePanel } from '@/components/control_plane/MultiRepoExecutiveGovernancePanel'
import { useControlPlaneStore } from '@/store/useControlPlaneStore'

const loadMultiRepoExecutiveGovernance = vi.fn().mockResolvedValue(null)

describe('MultiRepoExecutiveGovernancePanel', () => {
  beforeEach(() => {
    loadMultiRepoExecutiveGovernance.mockClear()
    useControlPlaneStore.setState({
      selectedOrgName: 'yohandry10',
      displayTimezone: 'UTC',
      multiRepoExecutiveGovernanceUpdatedAt: Date.UTC(2026, 5, 16, 10, 0, 0),
      isMultiRepoExecutiveGovernanceLoading: false,
      multiRepoExecutiveGovernanceError: null,
      loadMultiRepoExecutiveGovernance,
      multiRepoExecutiveGovernance: {
        org_id: 'org-1',
        generated_at: Date.UTC(2026, 5, 16, 10, 0, 0),
        repositories: [{
          repository_full_name: 'yohandry10/Git-Gov',
          posture: 'attention',
          gate_count: 2,
          blocked_gate_count: 1,
          advisory_gate_count: 1,
          break_glass_count: 0,
          latest_gate_id: 'dga_123',
          latest_gate_decision: 'blocked',
          latest_gate_created_at: Date.UTC(2026, 5, 16, 9, 0, 0),
          change_risk_count: 1,
          high_risk_count: 1,
          needs_review_count: 0,
          latest_risk_level: 'high',
          latest_review_status: 'accepted_risk',
          latest_risk_created_at: Date.UTC(2026, 5, 16, 9, 5, 0),
          cab_packet_count: 1,
          cab_manifest_count: 1,
          active_manifest_count: 1,
          revoked_manifest_count: 0,
          latest_manifest_hash: `sha256:${'a'.repeat(64)}`,
          latest_manifest_status: 'active',
          latest_manifest_created_at: Date.UTC(2026, 5, 16, 9, 10, 0),
        }],
        totals: {
          repositories: 1,
          gate_count: 2,
          blocked_gate_count: 1,
          advisory_gate_count: 1,
          break_glass_count: 0,
          change_risk_count: 1,
          high_risk_count: 1,
          needs_review_count: 0,
          cab_packet_count: 1,
          cab_manifest_count: 1,
          active_manifest_count: 1,
          revoked_manifest_count: 0,
        },
        limit: 25,
        offset: 0,
        advisory_only: true,
        enforcement_used: false,
        deployment_execution: false,
        provider_mutation: false,
        repository_mutation: false,
        llm_used: false,
        agent_governance_used: false,
        compliance_claim: false,
        certification: false,
      },
    })
  })

  it('renders repository governance posture without implying enforcement or certification', () => {
    render(<MultiRepoExecutiveGovernancePanel />)

    expect(loadMultiRepoExecutiveGovernance).toHaveBeenCalledWith({
      org_name: 'yohandry10',
      limit: 25,
      offset: 0,
    })
    expect(screen.getByText('Executive Governance View')).toBeInTheDocument()
    expect(screen.getByText('yohandry10/Git-Gov')).toBeInTheDocument()
    expect(screen.getByText('attention')).toBeInTheDocument()
    expect(screen.getByText('1 repos')).toBeInTheDocument()
    expect(screen.getByText(/Does not approve, block, certify, deploy/)).toBeInTheDocument()
    expect(screen.getByText('Risk: high / accepted_risk')).toBeInTheDocument()
  })
})
