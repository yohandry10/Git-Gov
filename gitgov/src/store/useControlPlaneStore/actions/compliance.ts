import { parseCommandError, tauriInvoke } from '@/lib/tauri'
import type {
  ComplianceControlFrameworkListResponse,
  ComplianceEvidenceExportResponse,
  ComplianceEvidenceMappingResponse,
  ComplianceFrameworkPackImportResponse,
  ComplianceFrameworkPackListResponse,
  ComplianceReviewPackageResponse,
  ControlPlaneActions,
} from '../types'
import type { ControlPlaneGet, ControlPlaneSet } from '../store-types'

const GITGOV_RELEASE_GOVERNANCE_BASELINE = 'gitgov_release_governance_baseline_v1'

type ComplianceActionKeys =
  | 'loadComplianceFrameworks'
  | 'importComplianceFrameworkPack'
  | 'selectComplianceFramework'
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
    loadComplianceFrameworks: async () => {
      const { serverConfig, selectedOrgName } = get()
      if (!serverConfig) return []

      set({ isComplianceFrameworksLoading: true, complianceEvidenceError: null })
      try {
        const query = { org_name: selectedOrgName.trim() || null }
        const [frameworksResponse, packsResponse] = await Promise.all([
          tauriInvoke<ComplianceControlFrameworkListResponse>('cmd_server_list_compliance_control_frameworks', {
            config: serverConfig,
            query,
          }),
          tauriInvoke<ComplianceFrameworkPackListResponse>('cmd_server_list_compliance_framework_packs', {
            config: serverConfig,
            query,
          }),
        ])
        const frameworks = frameworksResponse.frameworks
        const selected = get().selectedComplianceFrameworkId
        const selectedStillExists = frameworks.some((framework) => framework.framework_id === selected)
        set({
          complianceControlFrameworks: frameworks,
          complianceFrameworkPacks: packsResponse.framework_packs,
          selectedComplianceFrameworkId: selectedStillExists ? selected : GITGOV_RELEASE_GOVERNANCE_BASELINE,
          isComplianceFrameworksLoading: false,
        })
        return frameworks
      } catch (e) {
        const message = parseCommandError(String(e)).message
        set({
          complianceEvidenceError: message,
          isComplianceFrameworksLoading: false,
        })
        return []
      }
    },

    importComplianceFrameworkPack: async (content, format = 'json') => {
      const { serverConfig, selectedOrgName } = get()
      const normalizedContent = content.trim()
      if (!serverConfig || !normalizedContent) return null

      set({
        isComplianceFrameworkPackImporting: true,
        complianceEvidenceError: null,
      })
      try {
        const normalizedFormat = format === 'yml' ? 'yaml' : format
        const payload = normalizedFormat === 'json'
          ? {
              org_name: selectedOrgName.trim() || null,
              format: 'json',
              pack: JSON.parse(normalizedContent) as Record<string, unknown>,
              content: null,
            }
          : {
              org_name: selectedOrgName.trim() || null,
              format: normalizedFormat,
              pack: null,
              content: normalizedContent,
            }
        const response = await tauriInvoke<ComplianceFrameworkPackImportResponse>('cmd_server_import_compliance_framework_pack', {
          config: serverConfig,
          payload,
        })
        const frameworks = await get().loadComplianceFrameworks()
        set({
          complianceFrameworkImportResponse: response,
          selectedComplianceFrameworkId: response.framework.framework_id,
          complianceControlFrameworks: frameworks.length > 0 ? frameworks : [response.framework],
          isComplianceFrameworkPackImporting: false,
        })
        return response
      } catch (e) {
        const message = parseCommandError(String(e)).message
        set({
          complianceEvidenceError: message,
          isComplianceFrameworkPackImporting: false,
        })
        return null
      }
    },

    selectComplianceFramework: (frameworkId) => {
      const normalized = frameworkId.trim() || GITGOV_RELEASE_GOVERNANCE_BASELINE
      set({
        selectedComplianceFrameworkId: normalized,
        complianceEvidenceMapping: null,
        complianceReviewPackage: null,
        complianceReviewPackageArtifact: null,
      })
    },

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
              'readiness',
              'approvals',
              'evidence',
              'gaps',
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

    createComplianceEvidenceMapping: async (exportId, frameworkId) => {
      const { serverConfig, selectedOrgName } = get()
      const normalizedExportId = exportId.trim()
      const normalizedFrameworkId = (frameworkId ?? get().selectedComplianceFrameworkId).trim() || GITGOV_RELEASE_GOVERNANCE_BASELINE
      const selectedFramework = get().complianceControlFrameworks.find((framework) => framework.framework_id === normalizedFrameworkId)
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
            framework_version: selectedFramework?.version ?? null,
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
      selectedComplianceFrameworkId: GITGOV_RELEASE_GOVERNANCE_BASELINE,
      complianceFrameworkImportResponse: null,
      complianceEvidenceExport: null,
      complianceEvidenceMapping: null,
      complianceReviewPackage: null,
      complianceReviewPackageArtifact: null,
      complianceEvidenceError: null,
    }),
  }
}
