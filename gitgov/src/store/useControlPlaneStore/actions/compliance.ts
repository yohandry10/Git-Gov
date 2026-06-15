import { parseCommandError, tauriInvoke } from '@/lib/tauri'
import type {
  ComplianceControlFrameworkListResponse,
  ComplianceEvidenceExportResponse,
  ComplianceEvidenceMappingResponse,
  ComplianceFrameworkPackDiffResponse,
  ComplianceFrameworkPackRecord,
  ComplianceFrameworkPackReviewRequest,
  ComplianceFrameworkPackImportResponse,
  ComplianceFrameworkReviewReportAssignmentsResponse,
  ComplianceFrameworkReviewReportCommentRecord,
  ComplianceFrameworkReviewReportCommentsResponse,
  ComplianceFrameworkPackListResponse,
  ComplianceFrameworkReviewReportListResponse,
  ComplianceFrameworkReviewReportResponse,
  ComplianceReviewPackageResponse,
  ControlPlaneActions,
} from '../types'
import type { ControlPlaneGet, ControlPlaneSet } from '../store-types'
import { createComplianceFrameworkReviewReportArtifactActions } from './framework-review-report-artifacts'
import { createCompliancePeriodReportActions } from './period-reports'

const GITGOV_RELEASE_GOVERNANCE_BASELINE = 'gitgov_release_governance_baseline_v1'

type ComplianceActionKeys =
  | 'loadComplianceFrameworks'
  | 'importComplianceFrameworkPack'
  | 'reviewComplianceFrameworkPack'
  | 'loadComplianceFrameworkPackDiff'
  | 'selectComplianceFramework'
  | 'createComplianceEvidenceExport'
  | 'createComplianceEvidenceMapping'
  | 'createComplianceReviewPackage'
  | 'downloadComplianceReviewPackage'
  | 'createComplianceFrameworkReviewReport'
  | 'loadComplianceFrameworkReviewReports'
  | 'loadAssignedComplianceFrameworkReviewReports'
  | 'loadComplianceFrameworkReviewReportAssignments'
  | 'saveComplianceFrameworkReviewReportAssignments'
  | 'loadComplianceFrameworkReviewReportComments'
  | 'createComplianceFrameworkReviewReportComment'
  | 'reviewComplianceFrameworkReviewReport'
  | 'downloadComplianceFrameworkReviewReport'
  | 'createComplianceFrameworkReviewReportProvenanceManifest'
  | 'createComplianceFrameworkReviewReportPdfExport'
  | 'downloadComplianceFrameworkReviewReportPdfExport'
  | 'createCompliancePeriodReport'
  | 'loadCompliancePeriodReports'
  | 'downloadCompliancePeriodReport'
  | 'updateCompliancePeriodReportRetention'
  | 'loadCompliancePeriodReportAccessLog'
  | 'createCompliancePeriodReportPdfExport'
  | 'downloadCompliancePeriodReportPdfExport'
  | 'createCompliancePeriodReportProvenanceManifest'
  | 'downloadCompliancePeriodReportProvenanceManifest'
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
        const importedFrameworkIsReady = frameworks.some((framework) => framework.framework_id === response.framework.framework_id)
        set({
          complianceFrameworkImportResponse: response,
          selectedComplianceFrameworkId: importedFrameworkIsReady
            ? response.framework.framework_id
            : GITGOV_RELEASE_GOVERNANCE_BASELINE,
          complianceControlFrameworks: frameworks,
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

    reviewComplianceFrameworkPack: async (frameworkPackId, reviewStatus, notes = {}) => {
      const { serverConfig, selectedOrgName } = get()
      const normalizedPackId = frameworkPackId.trim()
      if (!serverConfig || !normalizedPackId) return null

      set({
        isComplianceFrameworkPackReviewing: true,
        complianceEvidenceError: null,
      })
      try {
        const payload: ComplianceFrameworkPackReviewRequest = {
          org_name: selectedOrgName.trim() || null,
          review_status: reviewStatus,
          review_notes_safe: notes.review_notes_safe?.trim() || null,
          rejected_reason_safe: notes.rejected_reason_safe?.trim() || null,
        }
        const response = await tauriInvoke<ComplianceFrameworkPackRecord>('cmd_server_review_compliance_framework_pack', {
          config: serverConfig,
          frameworkPackId: normalizedPackId,
          payload,
        })
        await get().loadComplianceFrameworks()
        set((state) => ({
          complianceFrameworkPacks: state.complianceFrameworkPacks.map((pack) =>
            pack.framework_pack_id === response.framework_pack_id ? response : pack,
          ).filter((pack) => pack.review_status !== 'archived'),
          selectedComplianceFrameworkId:
            response.review_status === 'reviewed'
              ? response.framework_id
              : state.selectedComplianceFrameworkId === response.framework_id
                ? GITGOV_RELEASE_GOVERNANCE_BASELINE
                : state.selectedComplianceFrameworkId,
          complianceEvidenceMapping:
            state.selectedComplianceFrameworkId === response.framework_id ? null : state.complianceEvidenceMapping,
          complianceReviewPackage:
            state.selectedComplianceFrameworkId === response.framework_id ? null : state.complianceReviewPackage,
          complianceReviewPackageArtifact:
            state.selectedComplianceFrameworkId === response.framework_id ? null : state.complianceReviewPackageArtifact,
          complianceFrameworkReviewReport:
            state.selectedComplianceFrameworkId === response.framework_id ? null : state.complianceFrameworkReviewReport,
          complianceFrameworkReviewReportArtifact:
            state.selectedComplianceFrameworkId === response.framework_id ? null : state.complianceFrameworkReviewReportArtifact,
          isComplianceFrameworkPackReviewing: false,
        }))
        return response
      } catch (e) {
        const message = parseCommandError(String(e)).message
        set({
          complianceEvidenceError: message,
          isComplianceFrameworkPackReviewing: false,
        })
        return null
      }
    },

    loadComplianceFrameworkPackDiff: async (basePackId, targetPackId) => {
      const { serverConfig, selectedOrgName } = get()
      const normalizedBasePackId = basePackId.trim()
      const normalizedTargetPackId = targetPackId.trim()
      if (!serverConfig || !normalizedBasePackId || !normalizedTargetPackId) return null

      set({
        isComplianceFrameworkPackDiffLoading: true,
        complianceEvidenceError: null,
      })
      try {
        const response = await tauriInvoke<ComplianceFrameworkPackDiffResponse>('cmd_server_diff_compliance_framework_packs', {
          config: serverConfig,
          query: {
            org_name: selectedOrgName.trim() || null,
            base_pack_id: normalizedBasePackId,
            target_pack_id: normalizedTargetPackId,
          },
        })
        set({
          complianceFrameworkPackDiff: response,
          isComplianceFrameworkPackDiffLoading: false,
        })
        return response
      } catch (e) {
        const message = parseCommandError(String(e)).message
        set({
          complianceEvidenceError: message,
          isComplianceFrameworkPackDiffLoading: false,
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
        complianceFrameworkReviewReport: null,
        complianceFrameworkReviewReportArtifact: null,
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
          complianceFrameworkReviewReport: null,
          complianceFrameworkReviewReportArtifact: null,
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
          complianceFrameworkReviewReport: null,
          complianceFrameworkReviewReportArtifact: null,
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
          complianceFrameworkReviewReport: null,
          complianceFrameworkReviewReportArtifact: null,
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

    createComplianceFrameworkReviewReport: async (mappingId, reviewPackageId) => {
      const { serverConfig, selectedOrgName } = get()
      const normalizedMappingId = mappingId.trim()
      const normalizedPackageId = reviewPackageId.trim()
      if (!serverConfig || !normalizedMappingId || !normalizedPackageId) return null

      set({
        isComplianceFrameworkReviewReportCreating: true,
        complianceEvidenceError: null,
      })
      try {
        const response = await tauriInvoke<ComplianceFrameworkReviewReportResponse>('cmd_server_create_compliance_framework_review_report', {
          config: serverConfig,
          payload: {
            org_name: selectedOrgName.trim() || null,
            mapping_id: normalizedMappingId,
            review_package_id: normalizedPackageId,
            format: 'json',
          },
        })
        const currentReports = get().complianceFrameworkReviewReports
        const updatedReportItems = currentReports
          ? [
              response.report,
              ...currentReports.items.filter((item) => item.report_id !== response.report.report_id),
            ].slice(0, currentReports.limit)
          : null
        set({
          complianceFrameworkReviewReport: response,
          complianceFrameworkReviewReports: currentReports
            ? {
                ...currentReports,
                items: updatedReportItems ?? currentReports.items,
                count: updatedReportItems?.length ?? currentReports.count,
              }
            : currentReports,
          complianceFrameworkReviewReportArtifact: response.artifact ?? null,
          isComplianceFrameworkReviewReportCreating: false,
        })
        return response
      } catch (e) {
        const message = parseCommandError(String(e)).message
        set({
          complianceEvidenceError: message,
          isComplianceFrameworkReviewReportCreating: false,
        })
        return null
      }
    },

    loadComplianceFrameworkReviewReports: async (filters = {}) => {
      const { serverConfig, selectedOrgName } = get()
      if (!serverConfig) return null

      set({
        isComplianceFrameworkReviewReportsLoading: true,
        complianceEvidenceError: null,
      })
      try {
        const response = await tauriInvoke<ComplianceFrameworkReviewReportListResponse>('cmd_server_list_compliance_framework_review_reports', {
          config: serverConfig,
          query: {
            org_name: selectedOrgName.trim() || null,
            framework_id: filters.framework_id?.trim() || null,
            mapping_id: filters.mapping_id?.trim() || null,
            review_package_id: filters.review_package_id?.trim() || null,
            limit: filters.limit ?? 25,
            assigned_to_me: filters.assigned_to_me ?? null,
          },
        })
        set({
          complianceFrameworkReviewReports: response,
          isComplianceFrameworkReviewReportsLoading: false,
        })
        return response
      } catch (e) {
        const message = parseCommandError(String(e)).message
        set({
          complianceEvidenceError: message,
          isComplianceFrameworkReviewReportsLoading: false,
        })
        return null
      }
    },

    loadAssignedComplianceFrameworkReviewReports: async (filters = {}) => {
      const { serverConfig, selectedOrgName } = get()
      if (!serverConfig) return null

      set({
        isAssignedComplianceFrameworkReviewReportsLoading: true,
        complianceEvidenceError: null,
      })
      try {
        const response = await tauriInvoke<ComplianceFrameworkReviewReportListResponse>('cmd_server_list_assigned_compliance_framework_review_reports', {
          config: serverConfig,
          query: {
            org_name: selectedOrgName.trim() || null,
            framework_id: filters.framework_id?.trim() || null,
            mapping_id: filters.mapping_id?.trim() || null,
            review_package_id: filters.review_package_id?.trim() || null,
            limit: filters.limit ?? 25,
            assigned_to_me: true,
          },
        })
        set({
          assignedComplianceFrameworkReviewReports: response,
          isAssignedComplianceFrameworkReviewReportsLoading: false,
        })
        return response
      } catch (e) {
        const message = parseCommandError(String(e)).message
        set({
          complianceEvidenceError: message,
          isAssignedComplianceFrameworkReviewReportsLoading: false,
        })
        return null
      }
    },

    loadComplianceFrameworkReviewReportAssignments: async (reportId) => {
      const { serverConfig, selectedOrgName } = get()
      const normalizedReportId = reportId.trim()
      if (!serverConfig || !normalizedReportId) return null

      set({
        isComplianceFrameworkReviewReportAssignmentsLoading: true,
        complianceEvidenceError: null,
      })
      try {
        const response = await tauriInvoke<ComplianceFrameworkReviewReportAssignmentsResponse>('cmd_server_list_compliance_framework_review_report_assignments', {
          config: serverConfig,
          reportId: normalizedReportId,
          query: {
            org_name: selectedOrgName.trim() || null,
          },
        })
        set({
          complianceFrameworkReviewReportAssignments: response,
          isComplianceFrameworkReviewReportAssignmentsLoading: false,
        })
        return response
      } catch (e) {
        const message = parseCommandError(String(e)).message
        set({
          complianceEvidenceError: message,
          isComplianceFrameworkReviewReportAssignmentsLoading: false,
        })
        return null
      }
    },

    saveComplianceFrameworkReviewReportAssignments: async (reportId, auditorClientIds, assignmentNotesSafe = null) => {
      const { serverConfig, selectedOrgName } = get()
      const normalizedReportId = reportId.trim()
      const normalizedAuditorClientIds = Array.from(new Set(
        auditorClientIds.map((value) => value.trim()).filter(Boolean),
      ))
      if (!serverConfig || !normalizedReportId) return null

      set({
        isComplianceFrameworkReviewReportAssignmentsSaving: true,
        complianceEvidenceError: null,
      })
      try {
        const response = await tauriInvoke<ComplianceFrameworkReviewReportAssignmentsResponse>('cmd_server_upsert_compliance_framework_review_report_assignments', {
          config: serverConfig,
          reportId: normalizedReportId,
          payload: {
            org_name: selectedOrgName.trim() || null,
            auditor_client_ids: normalizedAuditorClientIds,
            assignment_notes_safe: assignmentNotesSafe?.trim() || null,
          },
        })
        set({
          complianceFrameworkReviewReportAssignments: response,
          isComplianceFrameworkReviewReportAssignmentsSaving: false,
        })
        return response
      } catch (e) {
        const message = parseCommandError(String(e)).message
        set({
          complianceEvidenceError: message,
          isComplianceFrameworkReviewReportAssignmentsSaving: false,
        })
        return null
      }
    },

    loadComplianceFrameworkReviewReportComments: async (reportId) => {
      const { serverConfig, selectedOrgName } = get()
      const normalizedReportId = reportId.trim()
      if (!serverConfig || !normalizedReportId) return null

      set({
        isComplianceFrameworkReviewReportCommentsLoading: true,
        complianceEvidenceError: null,
      })
      try {
        const response = await tauriInvoke<ComplianceFrameworkReviewReportCommentsResponse>('cmd_server_list_compliance_framework_review_report_comments', {
          config: serverConfig,
          reportId: normalizedReportId,
          query: {
            org_name: selectedOrgName.trim() || null,
          },
        })
        set({
          complianceFrameworkReviewReportComments: response,
          isComplianceFrameworkReviewReportCommentsLoading: false,
        })
        return response
      } catch (e) {
        const message = parseCommandError(String(e)).message
        set({
          complianceEvidenceError: message,
          isComplianceFrameworkReviewReportCommentsLoading: false,
        })
        return null
      }
    },

    createComplianceFrameworkReviewReportComment: async (reportId, commentBodySafe, reviewStatusSuggestion = null) => {
      const { serverConfig, selectedOrgName } = get()
      const normalizedReportId = reportId.trim()
      const normalizedComment = commentBodySafe.trim()
      if (!serverConfig || !normalizedReportId || !normalizedComment) return null

      set({
        isComplianceFrameworkReviewReportCommenting: true,
        complianceEvidenceError: null,
      })
      try {
        const response = await tauriInvoke<ComplianceFrameworkReviewReportCommentRecord>('cmd_server_create_compliance_framework_review_report_comment', {
          config: serverConfig,
          reportId: normalizedReportId,
          payload: {
            org_name: selectedOrgName.trim() || null,
            comment_body_safe: normalizedComment,
            review_status_suggestion: reviewStatusSuggestion?.trim() || null,
          },
        })
        const currentComments = get().complianceFrameworkReviewReportComments
        set({
          complianceFrameworkReviewReportComments: currentComments
            ? {
                comments: [...currentComments.comments, response],
                count: currentComments.count + 1,
              }
            : {
                comments: [response],
                count: 1,
              },
          isComplianceFrameworkReviewReportCommenting: false,
        })
        return response
      } catch (e) {
        const message = parseCommandError(String(e)).message
        set({
          complianceEvidenceError: message,
          isComplianceFrameworkReviewReportCommenting: false,
        })
        return null
      }
    },

    reviewComplianceFrameworkReviewReport: async (reportId, reviewStatus, reviewNotesSafe = null) => {
      const { serverConfig, selectedOrgName } = get()
      const normalizedReportId = reportId.trim()
      const normalizedReviewStatus = reviewStatus.trim().toLowerCase()
      const normalizedNotes = reviewNotesSafe?.trim() || null
      if (!serverConfig || !normalizedReportId || !normalizedReviewStatus) return null

      set({
        isComplianceFrameworkReviewReportReviewing: true,
        complianceEvidenceError: null,
      })
      try {
        const response = await tauriInvoke<ComplianceFrameworkReviewReportResponse>('cmd_server_review_compliance_framework_review_report', {
          config: serverConfig,
          reportId: normalizedReportId,
          payload: {
            org_name: selectedOrgName.trim() || null,
            review_status: normalizedReviewStatus,
            review_notes_safe: normalizedNotes,
          },
        })
        const currentReports = get().complianceFrameworkReviewReports
        set({
          complianceFrameworkReviewReport: response,
          complianceFrameworkReviewReports: currentReports
            ? {
                ...currentReports,
                items: currentReports.items.map((item) => (
                  item.report_id === response.report.report_id ? response.report : item
                )),
              }
            : currentReports,
          isComplianceFrameworkReviewReportReviewing: false,
        })
        return response
      } catch (e) {
        const message = parseCommandError(String(e)).message
        set({
          complianceEvidenceError: message,
          isComplianceFrameworkReviewReportReviewing: false,
        })
        return null
      }
    },

    downloadComplianceFrameworkReviewReport: async (reportId) => {
      const { serverConfig, selectedOrgName } = get()
      const normalizedReportId = reportId.trim()
      if (!serverConfig || !normalizedReportId) return null

      set({
        isComplianceFrameworkReviewReportDownloading: true,
        complianceEvidenceError: null,
      })
      try {
        const artifact = await tauriInvoke<Record<string, unknown>>('cmd_server_download_compliance_framework_review_report', {
          config: serverConfig,
          reportId: normalizedReportId,
          query: {
            org_name: selectedOrgName.trim() || null,
          },
        })
        set({
          complianceFrameworkReviewReportArtifact: artifact,
          isComplianceFrameworkReviewReportDownloading: false,
        })
        return artifact
      } catch (e) {
        const message = parseCommandError(String(e)).message
        set({
          complianceEvidenceError: message,
          isComplianceFrameworkReviewReportDownloading: false,
        })
        return null
      }
    },

    ...createComplianceFrameworkReviewReportArtifactActions(set, get),

    ...createCompliancePeriodReportActions(set, get),

    resetComplianceEvidenceFlow: () => set({
      complianceEvidenceSelectedDeploymentGateId: null,
      selectedComplianceFrameworkId: GITGOV_RELEASE_GOVERNANCE_BASELINE,
      complianceFrameworkImportResponse: null,
      complianceFrameworkPackDiff: null,
      complianceEvidenceExport: null,
      complianceEvidenceMapping: null,
      complianceReviewPackage: null,
      complianceReviewPackageArtifact: null,
      complianceFrameworkReviewReport: null,
      complianceFrameworkReviewReports: null,
      assignedComplianceFrameworkReviewReports: null,
      complianceFrameworkReviewReportAssignments: null,
      complianceFrameworkReviewReportComments: null,
      complianceFrameworkReviewReportArtifact: null,
      complianceFrameworkReviewReportProvenanceManifest: null,
      complianceFrameworkReviewReportPdfExport: null,
      compliancePeriodReport: null,
      compliancePeriodReports: null,
      compliancePeriodReportArtifact: null,
      compliancePeriodReportAccessLog: null,
      compliancePeriodReportPdfExport: null,
      compliancePeriodReportProvenanceManifest: null,
      complianceEvidenceError: null,
    }),
  }
}
