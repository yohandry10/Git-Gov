import { parseCommandError, tauriInvoke } from '@/lib/tauri'
import type {
  CompliancePeriodReportAccessLogResponse,
  CompliancePeriodReportPdfDownloadResponse,
  CompliancePeriodReportPdfExportResponse,
  CompliancePeriodReportProvenanceManifestResponse,
  CompliancePeriodReportSharePackageListResponse,
  CompliancePeriodReportSharePackageResponse,
  CompliancePeriodReportProfileListResponse,
  CompliancePeriodReportProfilePatchRequest,
  CompliancePeriodReportProfileRequest,
  CompliancePeriodReportProfileResponse,
  CompliancePeriodReportProfileRunResponse,
  CompliancePeriodReportListResponse,
  CompliancePeriodReportResponse,
  CompliancePeriodReportReviewRequest,
  ControlPlaneActions,
} from '../types'
import type { ControlPlaneGet, ControlPlaneSet } from '../store-types'

type PeriodReportActionKeys =
  | 'createCompliancePeriodReport'
  | 'loadCompliancePeriodReports'
  | 'createCompliancePeriodReportProfile'
  | 'loadCompliancePeriodReportProfiles'
  | 'updateCompliancePeriodReportProfile'
  | 'archiveCompliancePeriodReportProfile'
  | 'runCompliancePeriodReportProfile'
  | 'downloadCompliancePeriodReport'
  | 'reviewCompliancePeriodReport'
  | 'updateCompliancePeriodReportRetention'
  | 'loadCompliancePeriodReportAccessLog'
  | 'createCompliancePeriodReportPdfExport'
  | 'downloadCompliancePeriodReportPdfExport'
  | 'createCompliancePeriodReportProvenanceManifest'
  | 'downloadCompliancePeriodReportProvenanceManifest'
  | 'createCompliancePeriodReportSharePackage'
  | 'loadCompliancePeriodReportSharePackages'
  | 'downloadCompliancePeriodReportSharePackage'
  | 'revokeCompliancePeriodReportSharePackage'

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

    createCompliancePeriodReportProfile: async (payload) => {
      const { serverConfig, selectedOrgName } = get()
      const normalizedName = payload.name.trim()
      const normalizedPeriodType = payload.period_type.trim().toLowerCase()
      const normalizedFrameworkId = payload.framework_id?.trim() || null
      if (!serverConfig || !normalizedName || !normalizedPeriodType) return null

      set({
        isCompliancePeriodReportProfileCreating: true,
        complianceEvidenceError: null,
      })
      try {
        const request: CompliancePeriodReportProfileRequest = {
          org_name: selectedOrgName.trim() || null,
          name: normalizedName,
          period_type: normalizedPeriodType,
          framework_id: normalizedFrameworkId,
          framework_owner_type: payload.framework_owner_type ?? null,
          include_pdf: payload.include_pdf ?? true,
          include_manifest: payload.include_manifest ?? true,
          retention_days: payload.retention_days ?? 2555,
          filters: payload.filters ?? {},
        }
        const response = await tauriInvoke<CompliancePeriodReportProfileResponse>(
          'cmd_server_create_compliance_period_report_profile',
          {
            config: serverConfig,
            payload: request,
          },
        )
        const currentProfiles = get().compliancePeriodReportProfiles
        set({
          compliancePeriodReportProfile: response,
          compliancePeriodReportProfiles: currentProfiles
            ? {
              ...currentProfiles,
              items: [
                response.profile,
                ...currentProfiles.items.filter((item) => item.profile_id !== response.profile.profile_id),
              ].slice(0, currentProfiles.limit),
              count: Math.min(currentProfiles.limit, currentProfiles.count + 1),
            }
            : {
              items: [response.profile],
              count: 1,
              limit: 25,
            },
          isCompliancePeriodReportProfileCreating: false,
        })
        return response
      } catch (e) {
        const message = parseCommandError(String(e)).message
        set({
          complianceEvidenceError: message,
          isCompliancePeriodReportProfileCreating: false,
        })
        return null
      }
    },

    loadCompliancePeriodReportProfiles: async (filters = {}) => {
      const { serverConfig, selectedOrgName } = get()
      if (!serverConfig) return null

      set({
        isCompliancePeriodReportProfilesLoading: true,
        complianceEvidenceError: null,
      })
      try {
        const response = await tauriInvoke<CompliancePeriodReportProfileListResponse>(
          'cmd_server_list_compliance_period_report_profiles',
          {
            config: serverConfig,
            query: {
              org_name: selectedOrgName.trim() || null,
              framework_id: filters.framework_id?.trim() || null,
              status: filters.status ?? 'active',
              limit: filters.limit ?? 25,
            },
          },
        )
        set({
          compliancePeriodReportProfiles: response,
          isCompliancePeriodReportProfilesLoading: false,
        })
        return response
      } catch (e) {
        const message = parseCommandError(String(e)).message
        set({
          complianceEvidenceError: message,
          isCompliancePeriodReportProfilesLoading: false,
        })
        return null
      }
    },

    updateCompliancePeriodReportProfile: async (profileId, payload) => {
      const { serverConfig, selectedOrgName } = get()
      const normalizedProfileId = profileId.trim()
      if (!serverConfig || !normalizedProfileId) return null

      set({
        isCompliancePeriodReportProfileUpdating: true,
        complianceEvidenceError: null,
      })
      try {
        const request: CompliancePeriodReportProfilePatchRequest = {
          org_name: selectedOrgName.trim() || null,
          name: payload.name?.trim() || null,
          period_type: payload.period_type?.trim().toLowerCase() || null,
          framework_id: payload.framework_id?.trim() || null,
          framework_owner_type: payload.framework_owner_type ?? null,
          include_pdf: payload.include_pdf ?? null,
          include_manifest: payload.include_manifest ?? null,
          retention_days: payload.retention_days ?? null,
          filters: payload.filters ?? null,
        }
        const response = await tauriInvoke<CompliancePeriodReportProfileResponse>(
          'cmd_server_update_compliance_period_report_profile',
          {
            config: serverConfig,
            profileId: normalizedProfileId,
            payload: request,
          },
        )
        const currentProfiles = get().compliancePeriodReportProfiles
        set({
          compliancePeriodReportProfile: response,
          compliancePeriodReportProfiles: currentProfiles
            ? {
              ...currentProfiles,
              items: currentProfiles.items.map((item) =>
                item.profile_id === response.profile.profile_id ? response.profile : item,
              ),
            }
            : currentProfiles,
          isCompliancePeriodReportProfileUpdating: false,
        })
        return response
      } catch (e) {
        const message = parseCommandError(String(e)).message
        set({
          complianceEvidenceError: message,
          isCompliancePeriodReportProfileUpdating: false,
        })
        return null
      }
    },

    archiveCompliancePeriodReportProfile: async (profileId) => {
      const { serverConfig, selectedOrgName } = get()
      const normalizedProfileId = profileId.trim()
      if (!serverConfig || !normalizedProfileId) return null

      set({
        isCompliancePeriodReportProfileArchiving: true,
        complianceEvidenceError: null,
      })
      try {
        const response = await tauriInvoke<CompliancePeriodReportProfileResponse>(
          'cmd_server_archive_compliance_period_report_profile',
          {
            config: serverConfig,
            profileId: normalizedProfileId,
            payload: {
              org_name: selectedOrgName.trim() || null,
            },
          },
        )
        const currentProfiles = get().compliancePeriodReportProfiles
        set({
          compliancePeriodReportProfile: response,
          compliancePeriodReportProfiles: currentProfiles
            ? {
              ...currentProfiles,
              items: currentProfiles.items.filter((item) => item.profile_id !== response.profile.profile_id),
              count: Math.max(0, currentProfiles.count - 1),
            }
            : currentProfiles,
          isCompliancePeriodReportProfileArchiving: false,
        })
        return response
      } catch (e) {
        const message = parseCommandError(String(e)).message
        set({
          complianceEvidenceError: message,
          isCompliancePeriodReportProfileArchiving: false,
        })
        return null
      }
    },

    runCompliancePeriodReportProfile: async (profileId, payload = {}) => {
      const { serverConfig, selectedOrgName } = get()
      const normalizedProfileId = profileId.trim()
      if (!serverConfig || !normalizedProfileId) return null

      set({
        isCompliancePeriodReportProfileRunning: true,
        complianceEvidenceError: null,
      })
      try {
        const response = await tauriInvoke<CompliancePeriodReportProfileRunResponse>(
          'cmd_server_run_compliance_period_report_profile',
          {
            config: serverConfig,
            profileId: normalizedProfileId,
            payload: {
              org_name: selectedOrgName.trim() || null,
              date_range_start: payload.date_range_start ?? null,
              date_range_end: payload.date_range_end ?? null,
            },
          },
        )
        const currentProfiles = get().compliancePeriodReportProfiles
        const currentReports = get().compliancePeriodReports
        set({
          compliancePeriodReportProfileRun: response,
          compliancePeriodReportProfile: { profile: response.profile },
          compliancePeriodReport: {
            period_report: response.period_report,
            download_url: response.download_url,
            artifact: null,
          },
          compliancePeriodReportPdfExport: response.pdf_export
            ? {
              pdf_export: response.pdf_export,
              download_url: `/compliance/period-reports/${response.period_report.period_report_id}/pdf-export/download?pdf_export_id=${response.pdf_export.pdf_export_id}`,
            }
            : null,
          compliancePeriodReportProvenanceManifest: response.manifest
            ? {
              manifest: response.manifest,
              download_url: `/compliance/period-reports/${response.period_report.period_report_id}/provenance-manifests/${response.manifest.manifest_id}`,
              artifact: {},
            }
            : null,
          compliancePeriodReportProfiles: currentProfiles
            ? {
              ...currentProfiles,
              items: currentProfiles.items.map((item) =>
                item.profile_id === response.profile.profile_id ? response.profile : item,
              ),
            }
            : currentProfiles,
          compliancePeriodReports: currentReports
            ? {
              ...currentReports,
              items: [
                response.period_report,
                ...currentReports.items.filter(
                  (item) => item.period_report_id !== response.period_report.period_report_id,
                ),
              ].slice(0, currentReports.limit),
              count: Math.min(currentReports.limit, currentReports.count + 1),
            }
            : {
              items: [response.period_report],
              count: 1,
              limit: 25,
            },
          isCompliancePeriodReportProfileRunning: false,
        })
        return response
      } catch (e) {
        const message = parseCommandError(String(e)).message
        set({
          complianceEvidenceError: message,
          isCompliancePeriodReportProfileRunning: false,
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

    reviewCompliancePeriodReport: async (periodReportId, reviewStatus, reviewNotesSafe = null) => {
      const { serverConfig, selectedOrgName } = get()
      const normalizedPeriodReportId = periodReportId.trim()
      const normalizedReviewStatus = reviewStatus.trim().toLowerCase()
      const normalizedNotes = reviewNotesSafe?.trim() || null
      if (!serverConfig || !normalizedPeriodReportId || !normalizedReviewStatus) return null

      set({
        isCompliancePeriodReportReviewing: true,
        complianceEvidenceError: null,
      })
      try {
        const payload: CompliancePeriodReportReviewRequest = {
          org_name: selectedOrgName.trim() || null,
          review_status: normalizedReviewStatus,
          review_notes_safe: normalizedNotes,
        }
        const response = await tauriInvoke<CompliancePeriodReportResponse>(
          'cmd_server_review_compliance_period_report',
          {
            config: serverConfig,
            periodReportId: normalizedPeriodReportId,
            payload,
          },
        )
        const currentReports = get().compliancePeriodReports
        set({
          compliancePeriodReport: response,
          compliancePeriodReports: currentReports
            ? {
              ...currentReports,
              items: currentReports.items.map((item) =>
                item.period_report_id === response.period_report.period_report_id
                  ? response.period_report
                  : item,
              ),
            }
            : currentReports,
          isCompliancePeriodReportReviewing: false,
        })
        return response
      } catch (e) {
        const message = parseCommandError(String(e)).message
        set({
          complianceEvidenceError: message,
          isCompliancePeriodReportReviewing: false,
        })
        return null
      }
    },

    updateCompliancePeriodReportRetention: async (periodReportId, payload) => {
      const { serverConfig, selectedOrgName } = get()
      const normalizedPeriodReportId = periodReportId.trim()
      if (!serverConfig || !normalizedPeriodReportId) return null

      set({
        isCompliancePeriodReportRetentionUpdating: true,
        complianceEvidenceError: null,
      })
      try {
        const response = await tauriInvoke<CompliancePeriodReportResponse>(
          'cmd_server_update_compliance_period_report_retention',
          {
            config: serverConfig,
            periodReportId: normalizedPeriodReportId,
            payload: {
              org_name: selectedOrgName.trim() || null,
              retention_until: payload.retention_until ?? null,
              archive: payload.archive ?? false,
            },
          },
        )
        const currentReports = get().compliancePeriodReports
        set({
          compliancePeriodReport: response,
          compliancePeriodReports: currentReports
            ? {
              ...currentReports,
              items: currentReports.items.map((item) =>
                item.period_report_id === response.period_report.period_report_id
                  ? response.period_report
                  : item,
              ),
            }
            : currentReports,
          isCompliancePeriodReportRetentionUpdating: false,
        })
        return response
      } catch (e) {
        const message = parseCommandError(String(e)).message
        set({
          complianceEvidenceError: message,
          isCompliancePeriodReportRetentionUpdating: false,
        })
        return null
      }
    },

    loadCompliancePeriodReportAccessLog: async (periodReportId, query = {}) => {
      const { serverConfig, selectedOrgName } = get()
      const normalizedPeriodReportId = periodReportId.trim()
      if (!serverConfig || !normalizedPeriodReportId) return null

      set({
        isCompliancePeriodReportAccessLogLoading: true,
        complianceEvidenceError: null,
      })
      try {
        const response = await tauriInvoke<CompliancePeriodReportAccessLogResponse>(
          'cmd_server_list_compliance_period_report_access_log',
          {
            config: serverConfig,
            periodReportId: normalizedPeriodReportId,
            query: {
              org_name: selectedOrgName.trim() || null,
              limit: query.limit ?? 50,
            },
          },
        )
        set({
          compliancePeriodReportAccessLog: response,
          isCompliancePeriodReportAccessLogLoading: false,
        })
        return response
      } catch (e) {
        const message = parseCommandError(String(e)).message
        set({
          complianceEvidenceError: message,
          isCompliancePeriodReportAccessLogLoading: false,
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

    createCompliancePeriodReportProvenanceManifest: async (periodReportId) => {
      const { serverConfig, selectedOrgName } = get()
      const normalizedPeriodReportId = periodReportId.trim()
      if (!serverConfig || !normalizedPeriodReportId) return null

      set({
        isCompliancePeriodReportProvenanceManifestCreating: true,
        complianceEvidenceError: null,
      })
      try {
        const response = await tauriInvoke<CompliancePeriodReportProvenanceManifestResponse>(
          'cmd_server_create_compliance_period_report_provenance_manifest',
          {
            config: serverConfig,
            periodReportId: normalizedPeriodReportId,
            payload: {
              org_name: selectedOrgName.trim() || null,
            },
          },
        )
        set({
          compliancePeriodReportProvenanceManifest: response,
          isCompliancePeriodReportProvenanceManifestCreating: false,
        })
        return response
      } catch (e) {
        const message = parseCommandError(String(e)).message
        set({
          complianceEvidenceError: message,
          isCompliancePeriodReportProvenanceManifestCreating: false,
        })
        return null
      }
    },

    downloadCompliancePeriodReportProvenanceManifest: async (periodReportId, manifestId) => {
      const { serverConfig, selectedOrgName } = get()
      const normalizedPeriodReportId = periodReportId.trim()
      const normalizedManifestId = manifestId.trim()
      if (!serverConfig || !normalizedPeriodReportId || !normalizedManifestId) return null

      set({
        isCompliancePeriodReportProvenanceManifestDownloading: true,
        complianceEvidenceError: null,
      })
      try {
        const artifact = await tauriInvoke<Record<string, unknown>>(
          'cmd_server_download_compliance_period_report_provenance_manifest',
          {
            config: serverConfig,
            periodReportId: normalizedPeriodReportId,
            manifestId: normalizedManifestId,
            query: {
              org_name: selectedOrgName.trim() || null,
            },
          },
        )
        set({
          isCompliancePeriodReportProvenanceManifestDownloading: false,
        })
        return artifact
      } catch (e) {
        const message = parseCommandError(String(e)).message
        set({
          complianceEvidenceError: message,
          isCompliancePeriodReportProvenanceManifestDownloading: false,
        })
        return null
      }
    },

    createCompliancePeriodReportSharePackage: async (periodReportId) => {
      const { serverConfig, selectedOrgName } = get()
      const normalizedPeriodReportId = periodReportId.trim()
      if (!serverConfig || !normalizedPeriodReportId) return null

      set({
        isCompliancePeriodReportSharePackageCreating: true,
        complianceEvidenceError: null,
      })
      try {
        const response = await tauriInvoke<CompliancePeriodReportSharePackageResponse>(
          'cmd_server_create_compliance_period_report_share_package',
          {
            config: serverConfig,
            periodReportId: normalizedPeriodReportId,
            payload: {
              org_name: selectedOrgName.trim() || null,
            },
          },
        )
        const currentPackages = get().compliancePeriodReportSharePackages
        set({
          compliancePeriodReportSharePackage: response,
          compliancePeriodReportSharePackageArtifact: response.artifact ?? null,
          compliancePeriodReportSharePackages: currentPackages
            ? {
              ...currentPackages,
              items: [
                response.share_package,
                ...currentPackages.items.filter(
                  (item) => item.share_package_id !== response.share_package.share_package_id,
                ),
              ].slice(0, currentPackages.limit),
              count: Math.min(currentPackages.limit, currentPackages.count + 1),
            }
            : {
              items: [response.share_package],
              count: 1,
              limit: 25,
            },
          isCompliancePeriodReportSharePackageCreating: false,
        })
        return response
      } catch (e) {
        const message = parseCommandError(String(e)).message
        set({
          complianceEvidenceError: message,
          isCompliancePeriodReportSharePackageCreating: false,
        })
        return null
      }
    },

    loadCompliancePeriodReportSharePackages: async (periodReportId, query = {}) => {
      const { serverConfig, selectedOrgName } = get()
      const normalizedPeriodReportId = periodReportId.trim()
      if (!serverConfig || !normalizedPeriodReportId) return null

      set({
        isCompliancePeriodReportSharePackagesLoading: true,
        complianceEvidenceError: null,
      })
      try {
        const response = await tauriInvoke<CompliancePeriodReportSharePackageListResponse>(
          'cmd_server_list_compliance_period_report_share_packages',
          {
            config: serverConfig,
            periodReportId: normalizedPeriodReportId,
            query: {
              org_name: selectedOrgName.trim() || null,
              status: query.status ?? null,
              limit: query.limit ?? 25,
            },
          },
        )
        set({
          compliancePeriodReportSharePackages: response,
          isCompliancePeriodReportSharePackagesLoading: false,
        })
        return response
      } catch (e) {
        const message = parseCommandError(String(e)).message
        set({
          complianceEvidenceError: message,
          isCompliancePeriodReportSharePackagesLoading: false,
        })
        return null
      }
    },

    downloadCompliancePeriodReportSharePackage: async (sharePackageId) => {
      const { serverConfig, selectedOrgName } = get()
      const normalizedSharePackageId = sharePackageId.trim()
      if (!serverConfig || !normalizedSharePackageId) return null

      set({
        isCompliancePeriodReportSharePackageDownloading: true,
        complianceEvidenceError: null,
      })
      try {
        const artifact = await tauriInvoke<Record<string, unknown>>(
          'cmd_server_download_compliance_period_report_share_package',
          {
            config: serverConfig,
            sharePackageId: normalizedSharePackageId,
            query: {
              org_name: selectedOrgName.trim() || null,
            },
          },
        )
        const currentPackages = get().compliancePeriodReportSharePackages
        set({
          compliancePeriodReportSharePackageArtifact: artifact,
          compliancePeriodReportSharePackages: currentPackages
            ? {
              ...currentPackages,
              items: currentPackages.items.map((item) =>
                item.share_package_id === normalizedSharePackageId
                  ? {
                    ...item,
                    download_count: item.download_count + 1,
                    last_downloaded_at: Date.now(),
                    downloaded_at: item.downloaded_at ?? Date.now(),
                  }
                  : item,
              ),
            }
            : currentPackages,
          isCompliancePeriodReportSharePackageDownloading: false,
        })
        return artifact
      } catch (e) {
        const message = parseCommandError(String(e)).message
        set({
          complianceEvidenceError: message,
          isCompliancePeriodReportSharePackageDownloading: false,
        })
        return null
      }
    },

    revokeCompliancePeriodReportSharePackage: async (sharePackageId) => {
      const { serverConfig, selectedOrgName } = get()
      const normalizedSharePackageId = sharePackageId.trim()
      if (!serverConfig || !normalizedSharePackageId) return null

      set({
        isCompliancePeriodReportSharePackageRevoking: true,
        complianceEvidenceError: null,
      })
      try {
        const response = await tauriInvoke<CompliancePeriodReportSharePackageResponse>(
          'cmd_server_revoke_compliance_period_report_share_package',
          {
            config: serverConfig,
            sharePackageId: normalizedSharePackageId,
            payload: {
              org_name: selectedOrgName.trim() || null,
            },
          },
        )
        const currentPackages = get().compliancePeriodReportSharePackages
        set({
          compliancePeriodReportSharePackage: response,
          compliancePeriodReportSharePackages: currentPackages
            ? {
              ...currentPackages,
              items: currentPackages.items.map((item) =>
                item.share_package_id === response.share_package.share_package_id
                  ? response.share_package
                  : item,
              ),
            }
            : currentPackages,
          isCompliancePeriodReportSharePackageRevoking: false,
        })
        return response
      } catch (e) {
        const message = parseCommandError(String(e)).message
        set({
          complianceEvidenceError: message,
          isCompliancePeriodReportSharePackageRevoking: false,
        })
        return null
      }
    },
  }
}
