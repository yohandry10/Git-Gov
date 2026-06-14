import { act, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { ComplianceEvidenceFlowPanel } from '@/components/control_plane/ComplianceEvidenceFlowPanel'
import { useControlPlaneStore } from '@/store/useControlPlaneStore'
import type { DeploymentGateAuthorizationRecord } from '@/store/useControlPlaneStore/types'

const createComplianceEvidenceExport = vi.fn()
const createComplianceEvidenceMapping = vi.fn()
const createComplianceReviewPackage = vi.fn()
const downloadComplianceReviewPackage = vi.fn()
const resetComplianceEvidenceFlow = vi.fn()

function authorization(overrides: Partial<DeploymentGateAuthorizationRecord> = {}): DeploymentGateAuthorizationRecord {
  return {
    id: 'row-1',
    authorization_id: 'dga_kan102_source',
    org_id: 'org-1',
    release_id: 'KAN-102',
    repository_full_name: 'yohandry10/Git-Gov',
    branch: 'main',
    target_sha: 'abcdef1234567890abcdef1234567890abcdef12',
    environment: 'production',
    deployer: 'github-actions',
    ticket_id: 'KAN-102',
    evidence_packet_hash: 'e'.repeat(64),
    evidence_packet_uri: '/evidence/packets/tickets/KAN-102',
    decision: 'approved',
    approved: true,
    blocking: false,
    would_block: false,
    reason: 'Deployment gate authorized.',
    blocked_by: [],
    warnings: [],
    policy_checksum: 'f'.repeat(64),
    break_glass_eligible: false,
    break_glass_used: false,
    break_glass_reason: null,
    break_glass_authorized_by: null,
    break_glass_expires_at: null,
    break_glass_approval_id: null,
    break_glass_approval_hash: null,
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
        enforcement: 'blocking',
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
    created_at: Date.UTC(2026, 5, 14, 3, 0, 0),
    ...overrides,
  }
}

describe('ComplianceEvidenceFlowPanel', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    useControlPlaneStore.setState({
      selectedOrgName: 'yohandry10',
      displayTimezone: 'UTC',
      deploymentGateAuthorizations: [authorization()],
      complianceEvidenceSelectedDeploymentGateId: null,
      complianceEvidenceExport: null,
      complianceEvidenceMapping: null,
      complianceReviewPackage: null,
      complianceReviewPackageArtifact: null,
      isComplianceEvidenceExportCreating: false,
      isComplianceEvidenceMappingCreating: false,
      isComplianceReviewPackageCreating: false,
      isComplianceReviewPackageDownloading: false,
      complianceEvidenceError: null,
      createComplianceEvidenceExport,
      createComplianceEvidenceMapping,
      createComplianceReviewPackage,
      downloadComplianceReviewPackage,
      resetComplianceEvidenceFlow,
    })
  })

  it('shows manual no-claim framing before any package is generated', () => {
    render(<ComplianceEvidenceFlowPanel />)

    expect(screen.getByText('Governance Evidence Review')).toBeInTheDocument()
    expect(screen.getByText('No certification claim')).toBeInTheDocument()
    expect(screen.getByText(/not SOC 2, ISO, NIST, PCI, SBS, LGPD/)).toBeInTheDocument()
    expect(screen.getByText('Manual-first:')).toBeInTheDocument()
    expect(screen.getByText('Agent required:')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /Export/i })).toBeEnabled()
    expect(screen.getByRole('button', { name: /Map/i })).toBeDisabled()
    expect(screen.getByRole('button', { name: /Package/i })).toBeDisabled()
    expect(screen.getByRole('button', { name: /JSON/i })).toBeDisabled()
  })

  it('runs the end-to-end review flow and downloads the server package artifact', async () => {
    createComplianceEvidenceExport.mockResolvedValue({
      export: {
        export_id: 'cee_kan102',
        org_id: 'org-1',
        created_by_user_id: 'admin',
        scope: 'deployment_gate',
        deployment_gate_id: 'dga_kan102_source',
        release_id: 'KAN-102',
        status: 'completed',
        format: 'json',
        artifact_hash: 'a'.repeat(64),
        policy_checksum: 'f'.repeat(64),
        gate_decision: 'approved',
        created_at: Date.UTC(2026, 5, 14, 3, 1, 0),
        completed_at: Date.UTC(2026, 5, 14, 3, 1, 1),
      },
      artifact: { compliance_claim: false },
    })
    createComplianceEvidenceMapping.mockResolvedValue({
      mapping: {
        mapping_id: 'cem_kan102',
        org_id: 'org-1',
        evidence_export_id: 'cee_kan102',
        evidence_export_hash: 'a'.repeat(64),
        framework_id: 'gitgov_release_governance_baseline_v1',
        framework_version: '1.0.0',
        created_by_user_id: 'admin',
        compliance_claim: false,
        regulatory_claim: false,
        requires_auditor_review: true,
        created_at: Date.UTC(2026, 5, 14, 3, 2, 0),
      },
      items: [
        {
          control_id: 'GOV-REL-001',
          control_title: 'Release authorization',
          status: 'covered',
          evidence_refs: ['deployment_gate:dga_kan102_source'],
          missing_evidence: [],
          notes_safe: 'Deployment Gate authorization was present.',
        },
        {
          control_id: 'GOV-REL-010',
          control_title: 'Auditor review handoff',
          status: 'partial',
          evidence_refs: [],
          missing_evidence: ['auditor_review'],
          notes_safe: 'Auditor review remains external.',
        },
      ],
    })
    createComplianceReviewPackage.mockResolvedValue({
      review_package: {
        review_package_id: 'crp_kan102',
        org_id: 'org-1',
        created_by_user_id: 'admin',
        mapping_id: 'cem_kan102',
        evidence_export_id: 'cee_kan102',
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
        created_at: Date.UTC(2026, 5, 14, 3, 3, 0),
      },
      download_url: '/compliance/review-packages/crp_kan102/download',
      artifact: { review_package_id: 'crp_kan102', compliance_claim: false },
    })
    downloadComplianceReviewPackage.mockResolvedValue({
      review_package_id: 'crp_kan102',
      artifact_hash: 'c'.repeat(64),
      compliance_claim: false,
      regulatory_claim: false,
      certification: false,
      missing_evidence: ['auditor_review'],
    })
    const clickSpy = vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(() => {})

    render(<ComplianceEvidenceFlowPanel />)

    fireEvent.click(screen.getByRole('button', { name: /Export/i }))
    await waitFor(() => expect(createComplianceEvidenceExport).toHaveBeenCalledWith('dga_kan102_source'))

    await act(async () => {
      useControlPlaneStore.setState({
        complianceEvidenceExport: await createComplianceEvidenceExport.mock.results[0].value,
      })
    })
    await waitFor(() => expect(screen.getByRole('button', { name: /Map/i })).toBeEnabled())
    fireEvent.click(screen.getByRole('button', { name: /Map/i }))
    await waitFor(() => expect(createComplianceEvidenceMapping).toHaveBeenCalledWith('cee_kan102'))

    await act(async () => {
      useControlPlaneStore.setState({
        complianceEvidenceMapping: await createComplianceEvidenceMapping.mock.results[0].value,
      })
    })
    await waitFor(() => expect(screen.getByRole('button', { name: /Package/i })).toBeEnabled())
    expect(screen.getByText('GOV-REL-010')).toBeInTheDocument()
    expect(screen.getByText(/Missing evidence: auditor_review/)).toBeInTheDocument()

    fireEvent.click(screen.getByRole('button', { name: /Package/i }))
    await waitFor(() => expect(createComplianceReviewPackage).toHaveBeenCalledWith('cem_kan102'))

    await act(async () => {
      useControlPlaneStore.setState({
        complianceReviewPackage: await createComplianceReviewPackage.mock.results[0].value,
      })
    })
    await waitFor(() => expect(screen.getByRole('button', { name: /JSON/i })).toBeEnabled())
    expect(screen.getAllByText('false').length).toBeGreaterThanOrEqual(2)
    expect(screen.getAllByText('true').length).toBeGreaterThanOrEqual(1)

    fireEvent.click(screen.getByRole('button', { name: /JSON/i }))
    await waitFor(() => expect(downloadComplianceReviewPackage).toHaveBeenCalledWith('crp_kan102'))
    expect(clickSpy).toHaveBeenCalled()
    clickSpy.mockRestore()
  })
})
