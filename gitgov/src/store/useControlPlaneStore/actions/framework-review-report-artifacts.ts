import { parseCommandError, tauriInvoke } from '@/lib/tauri'
import type {
  ComplianceFrameworkReviewReportPdfDownloadResponse,
  ComplianceFrameworkReviewReportPdfExportResponse,
  ComplianceFrameworkReviewReportProvenanceManifestResponse,
  ControlPlaneActions,
} from '../types'
import type { ControlPlaneGet, ControlPlaneSet } from '../store-types'

type FrameworkReviewReportArtifactActionKeys =
  | 'createComplianceFrameworkReviewReportProvenanceManifest'
  | 'createComplianceFrameworkReviewReportPdfExport'
  | 'downloadComplianceFrameworkReviewReportPdfExport'

export function createComplianceFrameworkReviewReportArtifactActions(
  set: ControlPlaneSet,
  get: ControlPlaneGet,
): Pick<ControlPlaneActions, FrameworkReviewReportArtifactActionKeys> {
  return {
    createComplianceFrameworkReviewReportProvenanceManifest: async (reportId) => {
      const { serverConfig, selectedOrgName } = get()
      const normalizedReportId = reportId.trim()
      if (!serverConfig || !normalizedReportId) return null

      set({
        isComplianceFrameworkReviewReportProvenanceManifestCreating: true,
        complianceEvidenceError: null,
      })
      try {
        const response = await tauriInvoke<ComplianceFrameworkReviewReportProvenanceManifestResponse>(
          'cmd_server_create_compliance_framework_review_report_provenance_manifest',
          {
            config: serverConfig,
            reportId: normalizedReportId,
            payload: {
              org_name: selectedOrgName.trim() || null,
            },
          },
        )
        set({
          complianceFrameworkReviewReportProvenanceManifest: response,
          isComplianceFrameworkReviewReportProvenanceManifestCreating: false,
        })
        return response
      } catch (e) {
        const message = parseCommandError(String(e)).message
        set({
          complianceEvidenceError: message,
          isComplianceFrameworkReviewReportProvenanceManifestCreating: false,
        })
        return null
      }
    },

    createComplianceFrameworkReviewReportPdfExport: async (reportId, manifestId = null) => {
      const { serverConfig, selectedOrgName } = get()
      const normalizedReportId = reportId.trim()
      const normalizedManifestId = manifestId?.trim() || null
      if (!serverConfig || !normalizedReportId) return null

      set({
        isComplianceFrameworkReviewReportPdfExportCreating: true,
        complianceEvidenceError: null,
      })
      try {
        const response = await tauriInvoke<ComplianceFrameworkReviewReportPdfExportResponse>(
          'cmd_server_create_compliance_framework_review_report_pdf_export',
          {
            config: serverConfig,
            reportId: normalizedReportId,
            payload: {
              org_name: selectedOrgName.trim() || null,
              manifest_id: normalizedManifestId,
            },
          },
        )
        set({
          complianceFrameworkReviewReportPdfExport: response,
          isComplianceFrameworkReviewReportPdfExportCreating: false,
        })
        return response
      } catch (e) {
        const message = parseCommandError(String(e)).message
        set({
          complianceEvidenceError: message,
          isComplianceFrameworkReviewReportPdfExportCreating: false,
        })
        return null
      }
    },

    downloadComplianceFrameworkReviewReportPdfExport: async (reportId, pdfExportId = null) => {
      const { serverConfig, selectedOrgName } = get()
      const normalizedReportId = reportId.trim()
      const normalizedPdfExportId = pdfExportId?.trim() || null
      if (!serverConfig || !normalizedReportId) return null

      set({
        isComplianceFrameworkReviewReportPdfExportDownloading: true,
        complianceEvidenceError: null,
      })
      try {
        const response = await tauriInvoke<ComplianceFrameworkReviewReportPdfDownloadResponse>(
          'cmd_server_download_compliance_framework_review_report_pdf_export',
          {
            config: serverConfig,
            reportId: normalizedReportId,
            query: {
              org_name: selectedOrgName.trim() || null,
              pdf_export_id: normalizedPdfExportId,
            },
          },
        )
        set({
          complianceFrameworkReviewReportPdfExport: {
            pdf_export: response.pdf_export,
            download_url: `/compliance/framework-review-reports/${normalizedReportId}/pdf-export/download?pdf_export_id=${response.pdf_export.pdf_export_id}`,
          },
          isComplianceFrameworkReviewReportPdfExportDownloading: false,
        })
        return response
      } catch (e) {
        const message = parseCommandError(String(e)).message
        set({
          complianceEvidenceError: message,
          isComplianceFrameworkReviewReportPdfExportDownloading: false,
        })
        return null
      }
    },
  }
}
