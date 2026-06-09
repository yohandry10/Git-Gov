import { parseCommandError, tauriInvoke } from '@/lib/tauri'
import type {
  ControlPlaneActions,
  EnterpriseAdoptionProfileRecord,
  EnterpriseAdoptionProfileResponse,
  EnterpriseOnboardingChecklistTrackingRecord,
  EnterpriseOnboardingChecklistTrackingResponse,
  EnterpriseReleaseApprovalListResponse,
  EnterpriseReleaseApprovalQuery,
  EnterpriseReleaseApprovalRecord,
  EnterpriseReleaseGovernanceEvaluationQuery,
  EnterpriseReleaseGovernanceEvaluationResponse,
  ExportLogEntry,
  ExportResponse,
} from '../types'
import type { ControlPlaneGet, ControlPlaneSet } from '../store-types'

type EnterpriseActionKeys =
  | 'loadEnterpriseAdoptionProfile'
  | 'saveEnterpriseAdoptionProfile'
  | 'loadEnterpriseOnboardingChecklistTracking'
  | 'saveEnterpriseOnboardingChecklistTracking'
  | 'loadEnterpriseReleaseApprovals'
  | 'evaluateEnterpriseReleaseGovernance'
  | 'createEnterpriseReleaseApproval'
  | 'exportAuditData'
  | 'loadExportLogs'

export function createEnterpriseActions(
  set: ControlPlaneSet,
  get: ControlPlaneGet,
): Pick<ControlPlaneActions, EnterpriseActionKeys> {
  return {
  loadEnterpriseAdoptionProfile: async (orgNameParam) => {
    const { serverConfig, selectedOrgName } = get()
    if (!serverConfig) return null
    const orgName = orgNameParam?.trim() || selectedOrgName.trim() || undefined

    set({ isEnterpriseAdoptionProfileLoading: true, enterpriseAdoptionProfileError: null })
    try {
      const response = await tauriInvoke<EnterpriseAdoptionProfileResponse>('cmd_server_get_enterprise_adoption_profile', {
        config: serverConfig,
        orgName,
      })
      const record = response.found ? response.profile ?? null : null
      const profile = record?.profile ?? null
      set({
        enterpriseAdoptionProfile: profile,
        enterpriseAdoptionProfileUpdatedAt: record?.updated_at ?? null,
        isEnterpriseAdoptionProfileLoading: false,
      })
      return profile
    } catch (e) {
      const message = parseCommandError(String(e)).message
      set({
        enterpriseAdoptionProfileError: message,
        isEnterpriseAdoptionProfileLoading: false,
      })
      return null
    }
  },

  saveEnterpriseAdoptionProfile: async (profile, orgNameParam) => {
    const { serverConfig, selectedOrgName } = get()
    if (!serverConfig) return false
    const orgName = orgNameParam?.trim() || selectedOrgName.trim() || undefined

    set({ isEnterpriseAdoptionProfileSaving: true, enterpriseAdoptionProfileError: null })
    try {
      const record = await tauriInvoke<EnterpriseAdoptionProfileRecord>('cmd_server_upsert_enterprise_adoption_profile', {
        config: serverConfig,
        payload: {
          org_name: orgName ?? null,
          profile,
        },
      })
      set({
        enterpriseAdoptionProfile: record.profile,
        enterpriseAdoptionProfileUpdatedAt: record.updated_at,
        isEnterpriseAdoptionProfileSaving: false,
      })
      return true
    } catch (e) {
      const message = parseCommandError(String(e)).message
      set({
        enterpriseAdoptionProfileError: message,
        isEnterpriseAdoptionProfileSaving: false,
      })
      return false
    }
  },

  loadEnterpriseOnboardingChecklistTracking: async (orgNameParam) => {
    const { serverConfig, selectedOrgName } = get()
    if (!serverConfig) return null
    const orgName = orgNameParam?.trim() || selectedOrgName.trim() || undefined

    set({
      isEnterpriseOnboardingChecklistTrackingLoading: true,
      enterpriseOnboardingChecklistTrackingError: null,
    })
    try {
      const response = await tauriInvoke<EnterpriseOnboardingChecklistTrackingResponse>('cmd_server_get_enterprise_onboarding_checklist_tracking', {
        config: serverConfig,
        orgName,
      })
      const record = response.found ? response.tracking ?? null : null
      const tracking = record?.tracking ?? null
      set({
        enterpriseOnboardingChecklistTracking: tracking,
        enterpriseOnboardingChecklistTrackingUpdatedAt: record?.updated_at ?? null,
        isEnterpriseOnboardingChecklistTrackingLoading: false,
      })
      return tracking
    } catch (e) {
      const message = parseCommandError(String(e)).message
      set({
        enterpriseOnboardingChecklistTrackingError: message,
        isEnterpriseOnboardingChecklistTrackingLoading: false,
      })
      return null
    }
  },

  saveEnterpriseOnboardingChecklistTracking: async (tracking, orgNameParam) => {
    const { serverConfig, selectedOrgName } = get()
    if (!serverConfig) return false
    const orgName = orgNameParam?.trim() || selectedOrgName.trim() || undefined

    set({
      isEnterpriseOnboardingChecklistTrackingSaving: true,
      enterpriseOnboardingChecklistTrackingError: null,
    })
    try {
      const record = await tauriInvoke<EnterpriseOnboardingChecklistTrackingRecord>('cmd_server_upsert_enterprise_onboarding_checklist_tracking', {
        config: serverConfig,
        payload: {
          org_name: orgName ?? null,
          tracking,
        },
      })
      set({
        enterpriseOnboardingChecklistTracking: record.tracking,
        enterpriseOnboardingChecklistTrackingUpdatedAt: record.updated_at,
        isEnterpriseOnboardingChecklistTrackingSaving: false,
      })
      return true
    } catch (e) {
      const message = parseCommandError(String(e)).message
      set({
        enterpriseOnboardingChecklistTrackingError: message,
        isEnterpriseOnboardingChecklistTrackingSaving: false,
      })
      return false
    }
  },

  loadEnterpriseReleaseApprovals: async (query = {}) => {
    const { serverConfig, selectedOrgName, releaseApprovalsFilters } = get()
    if (!serverConfig) return null
    const orgName = query.org_name?.trim() || selectedOrgName.trim() || undefined
    const nextQuery: EnterpriseReleaseApprovalQuery = {
      ...releaseApprovalsFilters,
      ...query,
      org_name: orgName ?? null,
      repository_full_name: query.repository_full_name?.trim() || releaseApprovalsFilters.repository_full_name || null,
      release_id: query.release_id?.trim() || releaseApprovalsFilters.release_id || null,
      environment: query.environment?.trim() || releaseApprovalsFilters.environment || null,
      decision: query.decision ?? releaseApprovalsFilters.decision ?? null,
      limit: query.limit ?? releaseApprovalsFilters.limit ?? 10,
      offset: query.offset ?? releaseApprovalsFilters.offset ?? 0,
    }

    set({ isReleaseApprovalsLoading: true, releaseApprovalError: null, releaseApprovalsFilters: nextQuery })
    try {
      const response = await tauriInvoke<EnterpriseReleaseApprovalListResponse>('cmd_server_list_enterprise_release_approvals', {
        config: serverConfig,
        query: nextQuery,
      })
      set({
        releaseApprovals: response.items,
        releaseApprovalsTotal: response.total,
        isReleaseApprovalsLoading: false,
      })
      return response
    } catch (e) {
      const message = parseCommandError(String(e)).message
      set({
        releaseApprovalError: message,
        isReleaseApprovalsLoading: false,
      })
      return null
    }
  },

  evaluateEnterpriseReleaseGovernance: async (query) => {
    const { serverConfig, selectedOrgName } = get()
    if (!serverConfig) return null
    const orgName = query.org_name?.trim() || selectedOrgName.trim() || undefined
    const nextQuery: EnterpriseReleaseGovernanceEvaluationQuery = {
      org_name: orgName ?? null,
      repository_full_name: query.repository_full_name.trim(),
      release_id: query.release_id.trim(),
      environment: query.environment.trim(),
      evidence_packet_hash: query.evidence_packet_hash?.trim() || null,
    }

    set({ isReleaseGovernanceEvaluating: true, releaseApprovalError: null })
    try {
      const response = await tauriInvoke<EnterpriseReleaseGovernanceEvaluationResponse>('cmd_server_evaluate_enterprise_release_governance', {
        config: serverConfig,
        query: nextQuery,
      })
      set({
        releaseGovernanceEvaluation: response,
        isReleaseGovernanceEvaluating: false,
      })
      return response
    } catch (e) {
      const message = parseCommandError(String(e)).message
      set({
        releaseApprovalError: message,
        releaseGovernanceEvaluation: null,
        isReleaseGovernanceEvaluating: false,
      })
      return null
    }
  },

  createEnterpriseReleaseApproval: async (payload) => {
    const { serverConfig, selectedOrgName } = get()
    if (!serverConfig) return null
    const effectiveOrgName = payload.org_name?.trim() || selectedOrgName.trim() || undefined

    set({ isReleaseApprovalSubmitting: true, releaseApprovalError: null })
    try {
      const record = await tauriInvoke<EnterpriseReleaseApprovalRecord>('cmd_server_create_enterprise_release_approval', {
        config: serverConfig,
        payload: {
          ...payload,
          org_name: effectiveOrgName ?? null,
          release_id: payload.release_id.trim(),
          repository_full_name: payload.repository_full_name.trim(),
          branch: payload.branch?.trim() || null,
          target_sha: payload.target_sha?.trim() || null,
          environment: payload.environment.trim(),
          decision: payload.decision,
          approver: payload.approver.trim(),
          ticket_id: payload.ticket_id?.trim() || null,
          evidence_packet_hash: payload.evidence_packet_hash?.trim() || null,
          evidence_packet_uri: payload.evidence_packet_uri?.trim() || null,
          evidence_summary: payload.evidence_summary ?? {},
          risk_severity: payload.risk_severity ?? 'none',
          risk_acceptance_reason: payload.risk_acceptance_reason?.trim() || null,
          expires_at: payload.expires_at ?? null,
        },
      })
      set((state) => ({
        releaseApprovals: [record, ...state.releaseApprovals].slice(0, state.releaseApprovalsFilters.limit ?? 10),
        releaseApprovalsTotal: state.releaseApprovalsTotal + 1,
        isReleaseApprovalSubmitting: false,
        releaseApprovalError: null,
      }))
      return record
    } catch (e) {
      const message = parseCommandError(String(e)).message
      set({
        releaseApprovalError: message,
        isReleaseApprovalSubmitting: false,
      })
      return null
    }
  },

  exportAuditData: async (params) => {
    const { serverConfig } = get()
    if (!serverConfig) return null
    try {
      const result = await tauriInvoke<ExportResponse>('cmd_server_export', {
        config: serverConfig,
        exportType: params.exportType ?? 'events',
        startDate: params.startDate ?? null,
        endDate: params.endDate ?? null,
        orgName: params.orgName ?? null,
      })
      await get().loadExportLogs()
      return result
    } catch (e) {
      set({ error: parseCommandError(String(e)).message })
      return null
    }
  },

  loadExportLogs: async () => {
    const { serverConfig } = get()
    if (!serverConfig) return
    try {
      const logs = await tauriInvoke<ExportLogEntry[]>('cmd_server_list_exports', { config: serverConfig })
      set({ exportLogs: logs })
    } catch {
      // Non-fatal
    }
  },
  }
}
