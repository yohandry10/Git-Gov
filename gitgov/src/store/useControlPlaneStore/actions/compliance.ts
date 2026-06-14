import { parseCommandError, tauriInvoke } from '@/lib/tauri'
import type {
  ComplianceEvidenceExportResponse,
  ComplianceEvidenceMappingResponse,
  ComplianceReviewPackageResponse,
  ControlPlaneActions,
} from '../types'
import type { ControlPlaneGet, ControlPlaneSet } from '../store-types'

const GITGOV_RELEASE_GOVERNANCE_BASELINE = 'gitgov_release_governance_baseline_v1'

type ComplianceActionKeys =
  | 'createComplianceEvidenceExport'
  | 'createComplianceEvidenceMapping'
  | 'createComplianceReviewPackage'
  | 'downloadComplianceReviewPackage'
  | 'resetComplianceEvidenceFlow'

export function createComplianceActions(
  set: ControlPlaneSet,
  get: ControlPlaneGet,
): Pick<ControlPlaneActions, ComplianceActionKeys> {
  return {
    createComplianceEvidenceExport: async (deploymentGateId) => {
      const { serverConfig, selectedOrgName } = get()
      const normalizedGateId = deploymentGateId.trim()
      if (!serverConfig || !normalizedGateId) return null

      set({
        complianceEvidenceSelectedDeploymentGateId: normalizedGateId,
        isComplianceEvidenceExportCreating: true,
        complianceEvidenceError: null,
      })
      try {
        const response = await tauriInvoke<ComplianceEvidenceExportResponse>('cmd_server_create_compliance_evidence_export', {
          config: serverConfig,
          payload: {
            org_name: selectedOrgName.trim() || null,
            scope: 'deployment_gate',
            deployment_gate_id: normalizedGateId,
            format: 'json',
            include_sections: [
              'gate_decision',
              'policy',
              'release',
              'approvals',
              'evidence',
              'missing_evidence',
              'audit',
            ],
          },
        })
        set({
          complianceEvidenceExport: response,
          complianceEvidenceMapping: null,
          complianceReviewPackage: null,
          complianceReviewPackageArtifact: null,
          isComplianceEvidenceExportCreating: false,
        })
        return response
      } catch (e) {
        const message = parseCommandError(String(e)).message
        set({
          complianceEvidenceError: message,
          isComplianceEvidenceExportCreating: false,
        })
        return null
      }
    },

    createComplianceEvidenceMapping: async (exportId, frameworkId = GITGOV_RELEASE_GOVERNANCE_BASELINE) => {
      const { serverConfig, selectedOrgName } = get()
      const normalizedExportId = exportId.trim()
      const normalizedFrameworkId = frameworkId.trim() || GITGOV_RELEASE_GOVERNANCE_BASELINE
      if (!serverConfig || !normalizedExportId) return null

      set({
        isComplianceEvidenceMappingCreating: true,
        complianceEvidenceError: null,
      })
      try {
        const response = await tauriInvoke<ComplianceEvidenceMappingResponse>('cmd_server_create_compliance_evidence_mapping', {
          config: serverConfig,
          payload: {
            org_name: selectedOrgName.trim() || null,
            evidence_export_id: normalizedExportId,
            framework_id: normalizedFrameworkId,
          },
        })
        set({
          complianceEvidenceMapping: response,
          complianceReviewPackage: null,
          complianceReviewPackageArtifact: null,
          isComplianceEvidenceMappingCreating: false,
        })
        return response
      } catch (e) {
        const message = parseCommandError(String(e)).message
        set({
          complianceEvidenceError: message,
          isComplianceEvidenceMappingCreating: false,
        })
        return null
      }
    },

    createComplianceReviewPackage: async (mappingId) => {
      const { serverConfig, selectedOrgName } = get()
      const normalizedMappingId = mappingId.trim()
      if (!serverConfig || !normalizedMappingId) return null

      set({
        isComplianceReviewPackageCreating: true,
        complianceEvidenceError: null,
      })
      try {
        const response = await tauriInvoke<ComplianceReviewPackageResponse>('cmd_server_create_compliance_review_package', {
          config: serverConfig,
          payload: {
            org_name: selectedOrgName.trim() || null,
            mapping_id: normalizedMappingId,
            format: 'json',
            include_sections: [
              'source_export',
              'control_matrix',
              'missing_evidence',
              'hashes',
              'no_claims',
              'audit_metadata',
            ],
          },
        })
        set({
          complianceReviewPackage: response,
          complianceReviewPackageArtifact: response.artifact ?? null,
          isComplianceReviewPackageCreating: false,
        })
        return response
      } catch (e) {
        const message = parseCommandError(String(e)).message
        set({
          complianceEvidenceError: message,
          isComplianceReviewPackageCreating: false,
        })
        return null
      }
    },

    downloadComplianceReviewPackage: async (reviewPackageId) => {
      const { serverConfig, selectedOrgName } = get()
      const normalizedPackageId = reviewPackageId.trim()
      if (!serverConfig || !normalizedPackageId) return null

      set({
        isComplianceReviewPackageDownloading: true,
        complianceEvidenceError: null,
      })
      try {
        const artifact = await tauriInvoke<Record<string, unknown>>('cmd_server_download_compliance_review_package', {
          config: serverConfig,
          reviewPackageId: normalizedPackageId,
          query: {
            org_name: selectedOrgName.trim() || null,
          },
        })
        set({
          complianceReviewPackageArtifact: artifact,
          isComplianceReviewPackageDownloading: false,
        })
        return artifact
      } catch (e) {
        const message = parseCommandError(String(e)).message
        set({
          complianceEvidenceError: message,
          isComplianceReviewPackageDownloading: false,
        })
        return null
      }
    },

    resetComplianceEvidenceFlow: () => set({
      complianceEvidenceSelectedDeploymentGateId: null,
      complianceEvidenceExport: null,
      complianceEvidenceMapping: null,
      complianceReviewPackage: null,
      complianceReviewPackageArtifact: null,
      complianceEvidenceError: null,
    }),
  }
}
