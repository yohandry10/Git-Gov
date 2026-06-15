import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { CompliancePeriodReportSharePackagePanel } from '@/components/control_plane/CompliancePeriodReportSharePackagePanel'
import { useControlPlaneStore } from '@/store/useControlPlaneStore'
import type {
  CompliancePeriodReportRecord,
  CompliancePeriodReportSharePackageRecord,
} from '@/store/useControlPlaneStore/types'

const createCompliancePeriodReportSharePackage = vi.fn().mockResolvedValue(null)
const loadCompliancePeriodReportSharePackages = vi.fn().mockResolvedValue(null)
const downloadCompliancePeriodReportSharePackage = vi.fn().mockResolvedValue(null)
const revokeCompliancePeriodReportSharePackage = vi.fn().mockResolvedValue(null)

function periodReport(overrides: Partial<CompliancePeriodReportRecord> = {}): CompliancePeriodReportRecord {
  return {
    period_report_id: 'cpr_kan119',
    org_id: 'org-1',
    created_by_user_id: 'admin',
    framework_id: 'gitgov_release_governance_baseline_v1',
    date_range_start: Date.UTC(2026, 5, 1),
    date_range_end: Date.UTC(2026, 5, 15),
    report_count: 1,
    source_report_ids: ['frr_kan119'],
    format: 'json',
    status: 'generated',
    artifact_hash: 'sha256:' + '1'.repeat(64),
    compliance_claim: false,
    regulatory_claim: false,
    requires_auditor_review: true,
    certification: false,
    review_status: 'reviewed',
    reviewed_by_user_id: 'auditor',
    reviewed_at: Date.UTC(2026, 5, 15),
    review_notes_safe: 'Reviewed for customer share.',
    created_at: Date.UTC(2026, 5, 15),
    downloaded_at: null,
    retention_status: 'active',
    retention_until: Date.UTC(2027, 5, 15),
    download_count: 0,
    last_downloaded_at: null,
    archived_at: null,
    error_message_safe: null,
    ...overrides,
  }
}

function sharePackage(overrides: Partial<CompliancePeriodReportSharePackageRecord> = {}): CompliancePeriodReportSharePackageRecord {
  return {
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
    created_at: Date.UTC(2026, 5, 15),
    error_message_safe: null,
    ...overrides,
  }
}

describe('CompliancePeriodReportSharePackagePanel', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    useControlPlaneStore.setState({
      displayTimezone: 'UTC',
      compliancePeriodReportSharePackages: {
        items: [sharePackage()],
        count: 1,
        limit: 25,
      },
      compliancePeriodReportSharePackage: null,
      compliancePeriodReportSharePackageArtifact: null,
      isCompliancePeriodReportSharePackageCreating: false,
      isCompliancePeriodReportSharePackagesLoading: false,
      isCompliancePeriodReportSharePackageDownloading: false,
      isCompliancePeriodReportSharePackageRevoking: false,
      createCompliancePeriodReportSharePackage,
      loadCompliancePeriodReportSharePackages,
      downloadCompliancePeriodReportSharePackage,
      revokeCompliancePeriodReportSharePackage,
    })
  })

  it('renders manual no-claim package evidence and triggers explicit actions', async () => {
    render(<CompliancePeriodReportSharePackagePanel periodReport={periodReport()} displayTimezone="UTC" />)

    expect(screen.getByText('Share packages')).toBeInTheDocument()
    expect(screen.getByText('manual-ready')).toBeInTheDocument()
    expect(screen.getByText(/not a certification, legal attestation, compliance score, or official regulatory claim/)).toBeInTheDocument()
    expect(screen.getByText('cprsp_kan119')).toBeInTheDocument()
    expect(screen.getByText(/Claims c:false r:false score:false/)).toBeInTheDocument()

    fireEvent.click(screen.getByRole('button', { name: /Packages/i }))
    fireEvent.click(screen.getByRole('button', { name: /Create/i }))
    fireEvent.click(screen.getByRole('button', { name: /JSON/i }))
    fireEvent.click(screen.getByRole('button', { name: /Revoke/i }))

    await waitFor(() => {
      expect(loadCompliancePeriodReportSharePackages).toHaveBeenCalledWith('cpr_kan119', { limit: 25 })
      expect(createCompliancePeriodReportSharePackage).toHaveBeenCalledWith('cpr_kan119')
      expect(downloadCompliancePeriodReportSharePackage).toHaveBeenCalledWith('cprsp_kan119')
      expect(revokeCompliancePeriodReportSharePackage).toHaveBeenCalledWith('cprsp_kan119')
    })
  })

  it('blocks creation when the period report still needs review', () => {
    render(
      <CompliancePeriodReportSharePackagePanel
        periodReport={periodReport({ review_status: 'needs_review' })}
        displayTimezone="UTC"
      />,
    )

    expect(screen.getByText('needs prerequisites')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /Create/i })).toBeDisabled()
    expect(screen.getByText(/Creation requires a reviewed, non-archived period report/)).toBeInTheDocument()
  })
})
