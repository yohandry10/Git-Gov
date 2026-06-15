import { parseCommandError, tauriInvoke } from '@/lib/tauri'
import type {
  CompliancePeriodReportPdfDownloadResponse,
  CompliancePeriodReportPdfExportResponse,
  CompliancePeriodReportListResponse,
  CompliancePeriodReportResponse,
  ControlPlaneActions,
} from '../types'
import type { ControlPlaneGet, ControlPlaneSet } from '../store-types'

type PeriodReportActionKeys =
  | 'createCompliancePeriodReport'
  | 'loadCompliancePeriodReports'
  | 'downloadCompliancePeriodReport'
  | 'createCompliancePeriodReportPdfExport'
  | 'downloadCompliancePeriodReportPdfExport'

export function createCompliancePeriodReportActions(
  set: ControlPlaneSet,
  get: ControlPlaneGet,
): Pick<ControlPlaneActions, PeriodReportActionKeys> {
  return {
    createCompliancePeriodReport: async (dateRangeStart, dateRangeEnd, frameworkId = null) => {
      const { serverConfig, selectedOrgName } = get()
      const normalizedFrameworkId = frameworkId?.trim() || null
      if (!serverConfig || !dateRangeStart || !dateRangeEnd) return null

      set({
        isCompliancePeriodReportCreating: true,
        complianceEvidenceError: null,
      })
      try {
        const response = await tauriInvoke<CompliancePeriodReportResponse>('cmd_server_create_compliance_period_report', {
          config: serverConfig,
          payload: {
            org_name: selectedOrgName.trim() || null,
            date_range_start: dateRangeStart,
            date_range_end: dateRangeEnd,
            framework_id: normalizedFrameworkId,
            format: 'json',
          },
        })
        const currentReports = get().compliancePeriodReports
        set({
          compliancePeriodReport: response,
          compliancePeriodReportArtifact: response.artifact ?? null,
          compliancePeriodReports: currentReports
            ? {
              ...currentReports,
              items: [
                response.period_report,
                ...currentReports.items.filter((item) => item.period_report_id !== response.period_report.period_report_id),
              ].slice(0, currentReports.limit),
              count: Math.min(currentReports.limit, currentReports.count + 1),
            }
            : {
              items: [response.period_report],
              count: 1,
              limit: 25,
            },
          isCompliancePeriodReportCreating: false,
        })
        return response
      } catch (e) {
        const message = parseCommandError(String(e)).message
        set({
          complianceEvidenceError: message,
          isCompliancePeriodReportCreating: false,
        })
        return null
      }
    },

    loadCompliancePeriodReports: async (filters = {}) => {
      const { serverConfig, selectedOrgName } = get()
      if (!serverConfig) return null
      const normalizedFrameworkId = filters.framework_id?.trim() || null

      set({
        isCompliancePeriodReportsLoading: true,
        complianceEvidenceError: null,
      })
      try {
        const response = await tauriInvoke<CompliancePeriodReportListResponse>('cmd_server_list_compliance_period_reports', {
          config: serverConfig,
          query: {
            org_name: selectedOrgName.trim() || null,
            framework_id: normalizedFrameworkId,
            limit: filters.limit ?? 25,
          },
        })
        set({
          compliancePeriodReports: response,
          isCompliancePeriodReportsLoading: false,
        })
        return response
      } catch (e) {
        const message = parseCommandError(String(e)).message
        set({
          complianceEvidenceError: message,
          isCompliancePeriodReportsLoading: false,
        })
        return null
      }
    },

    downloadCompliancePeriodReport: async (periodReportId) => {
      const { serverConfig, selectedOrgName } = get()
      const normalizedPeriodReportId = periodReportId.trim()
      if (!serverConfig || !normalizedPeriodReportId) return null

      set({
        isCompliancePeriodReportDownloading: true,
        complianceEvidenceError: null,
      })
      try {
        const artifact = await tauriInvoke<Record<string, unknown>>('cmd_server_download_compliance_period_report', {
          config: serverConfig,
          periodReportId: normalizedPeriodReportId,
          query: {
            org_name: selectedOrgName.trim() || null,
          },
        })
        set({
          compliancePeriodReportArtifact: artifact,
          isCompliancePeriodReportDownloading: false,
        })
        return artifact
      } catch (e) {
        const message = parseCommandError(String(e)).message
        set({
          complianceEvidenceError: message,
          isCompliancePeriodReportDownloading: false,
        })
        return null
      }
    },

    createCompliancePeriodReportPdfExport: async (periodReportId) => {
      const { serverConfig, selectedOrgName } = get()
      const normalizedPeriodReportId = periodReportId.trim()
      if (!serverConfig || !normalizedPeriodReportId) return null

      set({
        isCompliancePeriodReportPdfExportCreating: true,
        complianceEvidenceError: null,
      })
      try {
        const response = await tauriInvoke<CompliancePeriodReportPdfExportResponse>(
          'cmd_server_create_compliance_period_report_pdf_export',
          {
            config: serverConfig,
            periodReportId: normalizedPeriodReportId,
            payload: {
              org_name: selectedOrgName.trim() || null,
            },
          },
        )
        set({
          compliancePeriodReportPdfExport: response,
          isCompliancePeriodReportPdfExportCreating: false,
        })
        return response
      } catch (e) {
        const message = parseCommandError(String(e)).message
        set({
          complianceEvidenceError: message,
          isCompliancePeriodReportPdfExportCreating: false,
        })
        return null
      }
    },

    downloadCompliancePeriodReportPdfExport: async (periodReportId, pdfExportId = null) => {
      const { serverConfig, selectedOrgName } = get()
      const normalizedPeriodReportId = periodReportId.trim()
      const normalizedPdfExportId = pdfExportId?.trim() || null
      if (!serverConfig || !normalizedPeriodReportId) return null

      set({
        isCompliancePeriodReportPdfExportDownloading: true,
        complianceEvidenceError: null,
      })
      try {
        const response = await tauriInvoke<CompliancePeriodReportPdfDownloadResponse>(
          'cmd_server_download_compliance_period_report_pdf_export',
          {
            config: serverConfig,
            periodReportId: normalizedPeriodReportId,
            query: {
              org_name: selectedOrgName.trim() || null,
              pdf_export_id: normalizedPdfExportId,
            },
          },
        )
        set({
          compliancePeriodReportPdfExport: {
            pdf_export: response.pdf_export,
            download_url: `/compliance/period-reports/${normalizedPeriodReportId}/pdf-export/download?pdf_export_id=${response.pdf_export.pdf_export_id}`,
          },
          isCompliancePeriodReportPdfExportDownloading: false,
        })
        return response
      } catch (e) {
        const message = parseCommandError(String(e)).message
        set({
          complianceEvidenceError: message,
          isCompliancePeriodReportPdfExportDownloading: false,
        })
        return null
      }
    },
  }
}
