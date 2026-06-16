import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { MultiRepoExecutiveGovernancePanel } from '@/components/control_plane/MultiRepoExecutiveGovernancePanel'
import { useControlPlaneStore } from '@/store/useControlPlaneStore'

const loadMultiRepoExecutiveGovernance = vi.fn().mockResolvedValue(null)
const loadExecutiveGovernanceSnapshots = vi.fn().mockResolvedValue(null)
const createExecutiveGovernanceSnapshot = vi.fn().mockResolvedValue(null)
const getExecutiveGovernanceSnapshot = vi.fn().mockResolvedValue(null)
const downloadExecutiveGovernanceSnapshot = vi.fn().mockResolvedValue(null)
const archiveExecutiveGovernanceSnapshot = vi.fn().mockResolvedValue(null)

describe('MultiRepoExecutiveGovernancePanel', () => {
  beforeEach(() => {
    loadMultiRepoExecutiveGovernance.mockClear()
    loadExecutiveGovernanceSnapshots.mockClear()
    createExecutiveGovernanceSnapshot.mockClear()
    getExecutiveGovernanceSnapshot.mockClear()
    downloadExecutiveGovernanceSnapshot.mockClear()
    archiveExecutiveGovernanceSnapshot.mockClear()
    useControlPlaneStore.setState({
      selectedOrgName: 'yohandry10',
      displayTimezone: 'UTC',
      multiRepoExecutiveGovernanceUpdatedAt: Date.UTC(2026, 5, 16, 10, 0, 0),
      isMultiRepoExecutiveGovernanceLoading: false,
      multiRepoExecutiveGovernanceError: null,
      executiveGovernanceSnapshots: [],
      executiveGovernanceSnapshotsTotal: 0,
      executiveGovernanceSnapshotArtifact: null,
      executiveGovernanceSnapshotError: null,
      isExecutiveGovernanceSnapshotCreating: false,
      isExecutiveGovernanceSnapshotsLoading: false,
      isExecutiveGovernanceSnapshotDownloading: false,
      isExecutiveGovernanceSnapshotArchiving: false,
      loadMultiRepoExecutiveGovernance,
      loadExecutiveGovernanceSnapshots,
      createExecutiveGovernanceSnapshot,
      getExecutiveGovernanceSnapshot,
      downloadExecutiveGovernanceSnapshot,
      archiveExecutiveGovernanceSnapshot,
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
      repository: null,
      environment: null,
      posture: null,
      gate_decision: null,
      risk_level: null,
      review_status: null,
      limit: 25,
      offset: 0,
    })
    expect(screen.getByText('Executive Governance View')).toBeInTheDocument()
    expect(screen.getByText('yohandry10/Git-Gov')).toBeInTheDocument()
    expect(screen.getByText('attention')).toBeInTheDocument()
    expect(screen.getByText('1 repos')).toBeInTheDocument()
    expect(screen.getByText(/Does not approve, block, certify, deploy/)).toBeInTheDocument()
    expect(screen.getByText('Risk: high / accepted_risk')).toBeInTheDocument()
    expect(loadExecutiveGovernanceSnapshots).toHaveBeenCalledWith({
      org_name: 'yohandry10',
      status: 'active',
      limit: 10,
      offset: 0,
    })
  })

  it('applies executive governance filters without changing the read-only contract', async () => {
    render(<MultiRepoExecutiveGovernancePanel />)

    fireEvent.change(screen.getByLabelText('Repository'), { target: { value: 'Git-Gov' } })
    fireEvent.change(screen.getByLabelText('Environment'), { target: { value: 'production' } })
    fireEvent.change(screen.getByLabelText('Posture'), { target: { value: 'attention' } })
    fireEvent.change(screen.getByLabelText('Gate'), { target: { value: 'blocked' } })
    fireEvent.change(screen.getByLabelText('Risk'), { target: { value: 'high' } })
    fireEvent.change(screen.getByLabelText('Review'), { target: { value: 'accepted_risk' } })
    fireEvent.click(screen.getByText('Apply'))

    await waitFor(() => {
      expect(loadMultiRepoExecutiveGovernance).toHaveBeenLastCalledWith({
        org_name: 'yohandry10',
        repository: 'Git-Gov',
        environment: 'production',
        posture: 'attention',
        gate_decision: 'blocked',
        risk_level: 'high',
        review_status: 'accepted_risk',
        limit: 25,
        offset: 0,
      })
    })
    expect(screen.getByText('filtered')).toBeInTheDocument()
  })

  it('creates a hashable snapshot from the applied executive filters', async () => {
    render(<MultiRepoExecutiveGovernancePanel />)

    fireEvent.change(screen.getByLabelText('Environment'), { target: { value: 'production' } })
    fireEvent.change(screen.getByLabelText('Posture'), { target: { value: 'attention' } })
    fireEvent.change(screen.getByLabelText('Gate'), { target: { value: 'blocked' } })
    fireEvent.change(screen.getByLabelText('Risk'), { target: { value: 'high' } })
    fireEvent.change(screen.getByLabelText('Review'), { target: { value: 'accepted_risk' } })
    fireEvent.click(screen.getByText('Apply'))
    fireEvent.change(screen.getByDisplayValue('Executive governance snapshot'), {
      target: { value: 'Production risk snapshot' },
    })
    fireEvent.click(screen.getByText('Create Snapshot'))

    await waitFor(() => {
      expect(createExecutiveGovernanceSnapshot).toHaveBeenCalledWith({
        org_name: 'yohandry10',
        name: 'Production risk snapshot',
        filters: {
          org_name: null,
          repository: null,
          environment: 'production',
          posture: 'attention',
          gate_decision: 'blocked',
          risk_level: 'high',
          review_status: 'accepted_risk',
          limit: 100,
          offset: 0,
        },
        include_repository_rows: true,
        include_summary: true,
      })
    })
    expect(screen.getByText(/Hashable JSON artifact/)).toBeInTheDocument()
  })
})
