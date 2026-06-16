import { parseCommandError, tauriInvoke } from '@/lib/tauri'
import type {
  ControlPlaneActions,
  ChangeRiskEvaluationListResponse,
  ChangeRiskEvaluationQuery,
  ChangeRiskEvaluationRecord,
  ChangeRiskEvaluationRequest,
  ChangeRiskEvaluationTraceResponse,
  ChangeRiskRuleCatalogResponse,
  DeploymentGateAuthorizationListResponse,
  DeploymentGateAuthorizationQuery,
  EnterpriseAdoptionProfileRecord,
  EnterpriseAdoptionProfileResponse,
  FirstGovernedRepoWizardActionRequest,
  FirstGovernedRepoWizardRunResponse,
  FirstGovernedRepoWizardStateResponse,
  FirstGovernedRepoSetupRecord,
  FirstGovernedRepoSetupResponse,
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
  | 'loadFirstGovernedRepoSetup'
  | 'saveFirstGovernedRepoSetup'
  | 'loadFirstGovernedRepoWizardState'
  | 'createFirstGovernedRepoWizardRun'
  | 'updateFirstGovernedRepoWizardRun'
  | 'validateFirstGovernedRepoWizardRun'
  | 'planFirstGovernedRepoWizardRun'
  | 'completeFirstGovernedRepoWizardRun'
  | 'loadEnterpriseReleaseApprovals'
  | 'loadDeploymentGateAuthorizations'
  | 'loadChangeRiskEvaluations'
  | 'loadChangeRiskRules'
  | 'getChangeRiskEvaluation'
  | 'loadChangeRiskEvaluationTrace'
  | 'createChangeRiskEvaluation'
  | 'evaluateEnterpriseReleaseGovernance'
  | 'createEnterpriseReleaseApproval'
  | 'exportAuditData'
  | 'loadExportLogs'

function withSelectedOrg<T extends { org_name?: string | null }>(
  payload: T,
  orgName: string | undefined,
): T {
  return {
    ...payload,
    org_name: orgName ?? null,
  }
}

function applyFirstGovernedRepoWizardRun(
  set: ControlPlaneSet,
  response: FirstGovernedRepoWizardRunResponse,
) {
  set({
    firstGovernedRepoSetup: response.setup,
    firstGovernedRepoSetupUpdatedAt: response.setup.updated_at,
    firstGovernedRepoWizardState: response.state,
    isFirstGovernedRepoWizardActionRunning: false,
  })
}

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

  loadFirstGovernedRepoSetup: async (orgNameParam) => {
    const { serverConfig, selectedOrgName } = get()
    if (!serverConfig) return null
    const orgName = orgNameParam?.trim() || selectedOrgName.trim() || undefined

    set({
      isFirstGovernedRepoSetupLoading: true,
      firstGovernedRepoSetupError: null,
    })
    try {
      const response = await tauriInvoke<FirstGovernedRepoSetupResponse>('cmd_server_get_first_governed_repo_setup', {
        config: serverConfig,
        orgName,
      })
      const record = response.found ? response.setup ?? null : null
      set({
        firstGovernedRepoSetup: record,
        firstGovernedRepoSetupUpdatedAt: record?.updated_at ?? null,
        isFirstGovernedRepoSetupLoading: false,
      })
      return record
    } catch (e) {
      const message = parseCommandError(String(e)).message
      set({
        firstGovernedRepoSetupError: message,
        isFirstGovernedRepoSetupLoading: false,
      })
      return null
    }
  },

  saveFirstGovernedRepoSetup: async (payload, orgNameParam) => {
    const { serverConfig, selectedOrgName } = get()
    if (!serverConfig) return null
    const orgName = orgNameParam?.trim() || selectedOrgName.trim() || undefined

    set({
      isFirstGovernedRepoSetupSaving: true,
      firstGovernedRepoSetupError: null,
    })
    try {
      const record = await tauriInvoke<FirstGovernedRepoSetupRecord>('cmd_server_upsert_first_governed_repo_setup', {
        config: serverConfig,
        payload: {
          ...payload,
          org_name: orgName ?? null,
        },
      })
      set({
        firstGovernedRepoSetup: record,
        firstGovernedRepoSetupUpdatedAt: record.updated_at,
        isFirstGovernedRepoSetupSaving: false,
      })
      return record
    } catch (e) {
      const message = parseCommandError(String(e)).message
      set({
        firstGovernedRepoSetupError: message,
        isFirstGovernedRepoSetupSaving: false,
      })
      return null
    }
  },

  loadFirstGovernedRepoWizardState: async (orgNameParam) => {
    const { serverConfig, selectedOrgName } = get()
    if (!serverConfig) return null
    const orgName = orgNameParam?.trim() || selectedOrgName.trim() || undefined

    set({
      isFirstGovernedRepoWizardLoading: true,
      firstGovernedRepoWizardError: null,
    })
    try {
      const response = await tauriInvoke<FirstGovernedRepoWizardStateResponse>('cmd_server_get_first_governed_repo_wizard_state', {
        config: serverConfig,
        orgName,
      })
      const record = response.found ? response.setup ?? null : null
      set({
        firstGovernedRepoSetup: record,
        firstGovernedRepoSetupUpdatedAt: record?.updated_at ?? null,
        firstGovernedRepoWizardState: response.state,
        isFirstGovernedRepoWizardLoading: false,
      })
      return response
    } catch (e) {
      const message = parseCommandError(String(e)).message
      set({
        firstGovernedRepoWizardError: message,
        isFirstGovernedRepoWizardLoading: false,
      })
      return null
    }
  },

  createFirstGovernedRepoWizardRun: async (payload, orgNameParam) => {
    const { serverConfig, selectedOrgName } = get()
    if (!serverConfig) return null
    const orgName = orgNameParam?.trim() || selectedOrgName.trim() || undefined
    const request: FirstGovernedRepoWizardActionRequest = withSelectedOrg(payload, orgName)

    set({ isFirstGovernedRepoWizardActionRunning: true, firstGovernedRepoWizardError: null })
    try {
      const response = await tauriInvoke<FirstGovernedRepoWizardRunResponse>('cmd_server_create_first_governed_repo_wizard_run', {
        config: serverConfig,
        payload: request,
      })
      applyFirstGovernedRepoWizardRun(set, response)
      return response
    } catch (e) {
      const message = parseCommandError(String(e)).message
      set({ firstGovernedRepoWizardError: message, isFirstGovernedRepoWizardActionRunning: false })
      return null
    }
  },

  updateFirstGovernedRepoWizardRun: async (runId, payload, orgNameParam) => {
    const { serverConfig, selectedOrgName } = get()
    if (!serverConfig) return null
    const orgName = orgNameParam?.trim() || selectedOrgName.trim() || undefined
    const request: FirstGovernedRepoWizardActionRequest = withSelectedOrg(payload, orgName)

    set({ isFirstGovernedRepoWizardActionRunning: true, firstGovernedRepoWizardError: null })
    try {
      const response = await tauriInvoke<FirstGovernedRepoWizardRunResponse>('cmd_server_update_first_governed_repo_wizard_run', {
        config: serverConfig,
        runId,
        payload: request,
      })
      applyFirstGovernedRepoWizardRun(set, response)
      return response
    } catch (e) {
      const message = parseCommandError(String(e)).message
      set({ firstGovernedRepoWizardError: message, isFirstGovernedRepoWizardActionRunning: false })
      return null
    }
  },

  validateFirstGovernedRepoWizardRun: async (runId, payload, orgNameParam) => {
    const { serverConfig, selectedOrgName } = get()
    if (!serverConfig) return null
    const orgName = orgNameParam?.trim() || selectedOrgName.trim() || undefined
    const request: FirstGovernedRepoWizardActionRequest = withSelectedOrg(payload, orgName)

    set({ isFirstGovernedRepoWizardActionRunning: true, firstGovernedRepoWizardError: null })
    try {
      const response = await tauriInvoke<FirstGovernedRepoWizardRunResponse>('cmd_server_validate_first_governed_repo_wizard_run', {
        config: serverConfig,
        runId,
        payload: request,
      })
      applyFirstGovernedRepoWizardRun(set, response)
      return response
    } catch (e) {
      const message = parseCommandError(String(e)).message
      set({ firstGovernedRepoWizardError: message, isFirstGovernedRepoWizardActionRunning: false })
      return null
    }
  },

  planFirstGovernedRepoWizardRun: async (runId, payload, orgNameParam) => {
    const { serverConfig, selectedOrgName } = get()
    if (!serverConfig) return null
    const orgName = orgNameParam?.trim() || selectedOrgName.trim() || undefined
    const request: FirstGovernedRepoWizardActionRequest = withSelectedOrg(payload, orgName)

    set({ isFirstGovernedRepoWizardActionRunning: true, firstGovernedRepoWizardError: null })
    try {
      const response = await tauriInvoke<FirstGovernedRepoWizardRunResponse>('cmd_server_plan_first_governed_repo_wizard_run', {
        config: serverConfig,
        runId,
        payload: request,
      })
      applyFirstGovernedRepoWizardRun(set, response)
      return response
    } catch (e) {
      const message = parseCommandError(String(e)).message
      set({ firstGovernedRepoWizardError: message, isFirstGovernedRepoWizardActionRunning: false })
      return null
    }
  },

  completeFirstGovernedRepoWizardRun: async (runId, payload, orgNameParam) => {
    const { serverConfig, selectedOrgName } = get()
    if (!serverConfig) return null
    const orgName = orgNameParam?.trim() || selectedOrgName.trim() || undefined
    const request: FirstGovernedRepoWizardActionRequest = withSelectedOrg(payload, orgName)

    set({ isFirstGovernedRepoWizardActionRunning: true, firstGovernedRepoWizardError: null })
    try {
      const response = await tauriInvoke<FirstGovernedRepoWizardRunResponse>('cmd_server_complete_first_governed_repo_wizard_run', {
        config: serverConfig,
        runId,
        payload: request,
      })
      applyFirstGovernedRepoWizardRun(set, response)
      return response
    } catch (e) {
      const message = parseCommandError(String(e)).message
      set({ firstGovernedRepoWizardError: message, isFirstGovernedRepoWizardActionRunning: false })
      return null
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
      branch: query.branch?.trim() || releaseApprovalsFilters.branch || null,
      target_sha: query.target_sha?.trim() || releaseApprovalsFilters.target_sha || null,
      release_id: query.release_id?.trim() || releaseApprovalsFilters.release_id || null,
      environment: query.environment?.trim() || releaseApprovalsFilters.environment || null,
      decision: query.decision ?? releaseApprovalsFilters.decision ?? null,
      evidence_packet_hash: query.evidence_packet_hash?.trim() || releaseApprovalsFilters.evidence_packet_hash || null,
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

  loadDeploymentGateAuthorizations: async (query = {}) => {
    const { serverConfig, selectedOrgName, deploymentGateAuthorizationsFilters } = get()
    if (!serverConfig) return null
    const orgName = query.org_name?.trim() || selectedOrgName.trim() || undefined
    const nextQuery: DeploymentGateAuthorizationQuery = {
      ...deploymentGateAuthorizationsFilters,
      ...query,
      org_name: orgName ?? null,
      authorization_id: query.authorization_id?.trim() || deploymentGateAuthorizationsFilters.authorization_id || null,
      repository_full_name: query.repository_full_name?.trim() || deploymentGateAuthorizationsFilters.repository_full_name || null,
      branch: query.branch?.trim() || deploymentGateAuthorizationsFilters.branch || null,
      target_sha: query.target_sha?.trim() || deploymentGateAuthorizationsFilters.target_sha || null,
      release_id: query.release_id?.trim() || deploymentGateAuthorizationsFilters.release_id || null,
      environment: query.environment?.trim() || deploymentGateAuthorizationsFilters.environment || null,
      decision: query.decision?.trim() || deploymentGateAuthorizationsFilters.decision || null,
      deployer: query.deployer?.trim() || deploymentGateAuthorizationsFilters.deployer || null,
      limit: query.limit ?? deploymentGateAuthorizationsFilters.limit ?? 10,
      offset: query.offset ?? deploymentGateAuthorizationsFilters.offset ?? 0,
    }

    set({
      isDeploymentGateAuthorizationsLoading: true,
      releaseApprovalError: null,
      deploymentGateAuthorizationsFilters: nextQuery,
    })
    try {
      const response = await tauriInvoke<DeploymentGateAuthorizationListResponse>('cmd_server_list_deployment_gate_authorizations', {
        config: serverConfig,
        query: nextQuery,
      })
      set({
        deploymentGateAuthorizations: response.items,
        deploymentGateAuthorizationsTotal: response.total,
        deploymentGateAuthorizationsUpdatedAt: Date.now(),
        isDeploymentGateAuthorizationsLoading: false,
      })
      return response
    } catch (e) {
      const message = parseCommandError(String(e)).message
      set({
        releaseApprovalError: message,
        isDeploymentGateAuthorizationsLoading: false,
      })
      return null
    }
  },

  loadChangeRiskEvaluations: async (query = {}) => {
    const { serverConfig, selectedOrgName, changeRiskEvaluationsFilters } = get()
    if (!serverConfig) return null
    const orgName = query.org_name?.trim() || selectedOrgName.trim() || undefined
    const nextQuery: ChangeRiskEvaluationQuery = {
      ...changeRiskEvaluationsFilters,
      ...query,
      org_name: orgName ?? null,
      evaluation_id:
        query.evaluation_id?.trim() ||
        changeRiskEvaluationsFilters.evaluation_id ||
        null,
      deployment_gate_id:
        query.deployment_gate_id?.trim() ||
        changeRiskEvaluationsFilters.deployment_gate_id ||
        null,
      repository_full_name:
        query.repository_full_name?.trim() ||
        changeRiskEvaluationsFilters.repository_full_name ||
        null,
      branch: query.branch?.trim() || changeRiskEvaluationsFilters.branch || null,
      change_id: query.change_id?.trim() || changeRiskEvaluationsFilters.change_id || null,
      commit_sha: query.commit_sha?.trim() || changeRiskEvaluationsFilters.commit_sha || null,
      release_id: query.release_id?.trim() || changeRiskEvaluationsFilters.release_id || null,
      environment: query.environment?.trim() || changeRiskEvaluationsFilters.environment || null,
      limit: query.limit ?? changeRiskEvaluationsFilters.limit ?? 10,
      offset: query.offset ?? changeRiskEvaluationsFilters.offset ?? 0,
    }

    set({
      isChangeRiskEvaluationsLoading: true,
      changeRiskError: null,
      changeRiskEvaluationsFilters: nextQuery,
    })
    try {
      const response = await tauriInvoke<ChangeRiskEvaluationListResponse>('cmd_server_list_change_risk_evaluations', {
        config: serverConfig,
        query: nextQuery,
      })
      set({
        changeRiskEvaluations: response.items,
        changeRiskEvaluationsTotal: response.total,
        isChangeRiskEvaluationsLoading: false,
      })
      return response
    } catch (e) {
      const message = parseCommandError(String(e)).message
      set({
        changeRiskError: message,
        isChangeRiskEvaluationsLoading: false,
      })
      return null
    }
  },

  loadChangeRiskRules: async () => {
    const { serverConfig } = get()
    if (!serverConfig) return null

    set({ isChangeRiskRulesLoading: true, changeRiskError: null })
    try {
      const response = await tauriInvoke<ChangeRiskRuleCatalogResponse>('cmd_server_get_change_risk_rules', {
        config: serverConfig,
      })
      set({
        changeRiskRuleCatalog: response,
        isChangeRiskRulesLoading: false,
      })
      return response
    } catch (e) {
      const message = parseCommandError(String(e)).message
      set({
        changeRiskError: message,
        isChangeRiskRulesLoading: false,
      })
      return null
    }
  },

  getChangeRiskEvaluation: async (evaluationId, query = {}) => {
    const { serverConfig, selectedOrgName } = get()
    if (!serverConfig) return null
    const orgName = query.org_name?.trim() || selectedOrgName.trim() || undefined
    const nextQuery: ChangeRiskEvaluationQuery = {
      ...query,
      org_name: orgName ?? null,
    }

    set({ isChangeRiskEvaluationsLoading: true, changeRiskError: null })
    try {
      const record = await tauriInvoke<ChangeRiskEvaluationRecord>('cmd_server_get_change_risk_evaluation', {
        config: serverConfig,
        evaluationId: evaluationId.trim(),
        query: nextQuery,
      })
      set({
        changeRiskSelectedEvaluation: record,
        changeRiskEvaluationTrace: record.evaluation_trace
          ? {
              evaluation_id: record.evaluation_id,
              org_id: record.org_id,
              ruleset_version: record.ruleset_version,
              triggered_rules: record.triggered_rules,
              non_triggered_rules: record.non_triggered_rules,
              evaluation_trace: record.evaluation_trace,
              trace_hash: record.trace_hash,
              advisory_only: record.advisory_only,
              llm_used: record.llm_used,
              agent_governance_used: record.agent_governance_used,
              compliance_claim: record.compliance_claim,
              certification: record.certification,
              created_at: record.created_at,
            }
          : null,
        isChangeRiskEvaluationsLoading: false,
      })
      return record
    } catch (e) {
      const message = parseCommandError(String(e)).message
      set({
        changeRiskError: message,
        isChangeRiskEvaluationsLoading: false,
      })
      return null
    }
  },

  loadChangeRiskEvaluationTrace: async (evaluationId, query = {}) => {
    const { serverConfig, selectedOrgName } = get()
    if (!serverConfig) return null
    const orgName = query.org_name?.trim() || selectedOrgName.trim() || undefined
    const nextQuery: ChangeRiskEvaluationQuery = {
      ...query,
      org_name: orgName ?? null,
    }

    set({ isChangeRiskTraceLoading: true, changeRiskError: null })
    try {
      const response = await tauriInvoke<ChangeRiskEvaluationTraceResponse>('cmd_server_get_change_risk_evaluation_trace', {
        config: serverConfig,
        evaluationId: evaluationId.trim(),
        query: nextQuery,
      })
      set({
        changeRiskEvaluationTrace: response,
        isChangeRiskTraceLoading: false,
      })
      return response
    } catch (e) {
      const message = parseCommandError(String(e)).message
      set({
        changeRiskError: message,
        isChangeRiskTraceLoading: false,
      })
      return null
    }
  },

  createChangeRiskEvaluation: async (payload) => {
    const { serverConfig, selectedOrgName } = get()
    if (!serverConfig) return null
    const effectiveOrgName = payload.org_name?.trim() || selectedOrgName.trim() || undefined
    const request: ChangeRiskEvaluationRequest = {
      ...payload,
      org_name: effectiveOrgName ?? null,
      repository_full_name: payload.repository_full_name.trim(),
      branch: payload.branch.trim(),
      environment: payload.environment.trim(),
      deployment_gate_id: payload.deployment_gate_id?.trim() || null,
      release_id: payload.release_id?.trim() || null,
      commit_sha: payload.commit_sha?.trim() || null,
      evidence_packet_hash: payload.evidence_packet_hash?.trim() || null,
      change_id: payload.change_id?.trim() || null,
      evidence_refs: (payload.evidence_refs ?? []).map((item) => item.trim()).filter(Boolean),
    }

    set({ isChangeRiskEvaluationCreating: true, changeRiskError: null })
    try {
      const record = await tauriInvoke<ChangeRiskEvaluationRecord>('cmd_server_create_change_risk_evaluation', {
        config: serverConfig,
        payload: request,
      })
      set((state) => ({
        changeRiskEvaluations: [record, ...state.changeRiskEvaluations].slice(0, state.changeRiskEvaluationsFilters.limit ?? 10),
        changeRiskEvaluationsTotal: state.changeRiskEvaluationsTotal + 1,
        changeRiskSelectedEvaluation: record,
        changeRiskEvaluationTrace: {
          evaluation_id: record.evaluation_id,
          org_id: record.org_id,
          ruleset_version: record.ruleset_version,
          triggered_rules: record.triggered_rules,
          non_triggered_rules: record.non_triggered_rules,
          evaluation_trace: record.evaluation_trace,
          trace_hash: record.trace_hash,
          advisory_only: record.advisory_only,
          llm_used: record.llm_used,
          agent_governance_used: record.agent_governance_used,
          compliance_claim: record.compliance_claim,
          certification: record.certification,
          created_at: record.created_at,
        },
        isChangeRiskEvaluationCreating: false,
        changeRiskError: null,
      }))
      return record
    } catch (e) {
      const message = parseCommandError(String(e)).message
      set({
        changeRiskError: message,
        isChangeRiskEvaluationCreating: false,
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
      branch: query.branch?.trim() || null,
      target_sha: query.target_sha?.trim() || null,
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
