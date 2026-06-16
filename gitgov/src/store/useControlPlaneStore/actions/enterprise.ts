import { parseCommandError, tauriInvoke } from '@/lib/tauri'
import type {
  ControlPlaneActions,
  ChangeRiskCabDecisionManifestListResponse,
  ChangeRiskCabDecisionManifestQuery,
  ChangeRiskCabDecisionManifestRecord,
  ChangeRiskCabDecisionManifestRequest,
  ChangeRiskCabDecisionManifestResponse,
  ChangeRiskCabPacketListResponse,
  ChangeRiskCabPacketQuery,
  ChangeRiskCabPacketRecord,
  ChangeRiskCabPacketReviewRequest,
  ChangeRiskCabPacketReviewResponse,
  ChangeRiskCabPacketRequest,
  ChangeRiskCabPacketResponse,
  ChangeRiskEvaluationListResponse,
  ChangeRiskEvaluationQuery,
  ChangeRiskEvaluationRecord,
  ChangeRiskEvaluationRequest,
  ChangeRiskEvaluationReviewRequest,
  ChangeRiskEvaluationReviewResponse,
  ChangeRiskEvaluationTraceResponse,
  ChangeRiskRuleCatalogResponse,
  DeploymentGateAuthorizationListResponse,
  DeploymentGateAuthorizationQuery,
  DeploymentGateRiskContextResponse,
  MultiRepoExecutiveGovernanceQuery,
  MultiRepoExecutiveGovernanceResponse,
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
  | 'getDeploymentGateRiskContext'
  | 'loadMultiRepoExecutiveGovernance'
  | 'loadChangeRiskEvaluations'
  | 'loadChangeRiskRules'
  | 'getChangeRiskEvaluation'
  | 'loadChangeRiskEvaluationTrace'
  | 'loadChangeRiskEvaluationReview'
  | 'updateChangeRiskEvaluationReview'
  | 'createChangeRiskEvaluation'
  | 'createChangeRiskCabPacket'
  | 'loadChangeRiskCabPackets'
  | 'getChangeRiskCabPacket'
  | 'getChangeRiskCabPacketReview'
  | 'updateChangeRiskCabPacketReview'
  | 'downloadChangeRiskCabPacket'
  | 'archiveChangeRiskCabPacket'
  | 'createChangeRiskCabDecisionManifest'
  | 'loadChangeRiskCabDecisionManifests'
  | 'getChangeRiskCabDecisionManifest'
  | 'downloadChangeRiskCabDecisionManifest'
  | 'revokeChangeRiskCabDecisionManifest'
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

function applyChangeRiskReviewToRecord(
  record: ChangeRiskEvaluationRecord,
  review: ChangeRiskEvaluationReviewResponse,
): ChangeRiskEvaluationRecord {
  if (record.evaluation_id !== review.evaluation_id) return record
  return {
    ...record,
    review_status: review.review_status,
    reviewed_by_user_id: review.reviewed_by_user_id ?? null,
    reviewed_at: review.reviewed_at ?? null,
    review_notes_safe: review.review_notes_safe ?? null,
    mitigation_notes_safe: review.mitigation_notes_safe ?? null,
    decision_reason_safe: review.decision_reason_safe ?? null,
    review_updated_at: review.review_updated_at ?? null,
  }
}

function matchesChangeRiskReviewFilter(
  record: ChangeRiskEvaluationRecord,
  reviewStatus?: ChangeRiskEvaluationQuery['review_status'],
): boolean {
  return !reviewStatus || record.review_status === reviewStatus
}

function applyChangeRiskCabPacketToList(
  items: ChangeRiskCabPacketRecord[],
  packet: ChangeRiskCabPacketRecord,
): ChangeRiskCabPacketRecord[] {
  const next = items.filter((item) => item.packet_id !== packet.packet_id)
  return [packet, ...next]
}

function applyChangeRiskCabPacketReview(
  packet: ChangeRiskCabPacketRecord,
  review: ChangeRiskCabPacketReviewResponse,
): ChangeRiskCabPacketRecord {
  if (packet.packet_id !== review.packet_id) return packet
  return {
    ...packet,
    review_status: review.review_status,
    reviewed_by_user_id: review.reviewed_by_user_id ?? null,
    reviewed_at: review.reviewed_at ?? null,
    review_notes_safe: review.review_notes_safe ?? null,
    mitigation_notes_safe: review.mitigation_notes_safe ?? null,
    decision_reason_safe: review.decision_reason_safe ?? null,
    follow_up_required: review.follow_up_required,
    follow_up_owner_safe: review.follow_up_owner_safe ?? null,
    review_updated_at: review.review_updated_at ?? null,
  }
}

function applyChangeRiskCabDecisionManifestToList(
  items: ChangeRiskCabDecisionManifestRecord[],
  manifest: ChangeRiskCabDecisionManifestRecord,
): ChangeRiskCabDecisionManifestRecord[] {
  const next = items.filter((item) => item.manifest_id !== manifest.manifest_id)
  return [manifest, ...next]
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

  getDeploymentGateRiskContext: async (deploymentGateId, query = {}) => {
    const { serverConfig, selectedOrgName } = get()
    if (!serverConfig) return null
    const normalizedGateId = deploymentGateId.trim()
    if (!normalizedGateId) return null
    const orgName = query.org_name?.trim() || selectedOrgName.trim() || undefined
    const nextQuery: DeploymentGateAuthorizationQuery = {
      org_name: orgName ?? null,
    }

    set({
      isDeploymentGateRiskContextLoading: true,
      deploymentGateRiskContextError: null,
    })
    try {
      const response = await tauriInvoke<DeploymentGateRiskContextResponse>('cmd_server_get_deployment_gate_risk_context', {
        config: serverConfig,
        deploymentGateId: normalizedGateId,
        query: nextQuery,
      })
      set((state) => ({
        deploymentGateRiskContexts: {
          ...state.deploymentGateRiskContexts,
          [normalizedGateId]: response,
        },
        isDeploymentGateRiskContextLoading: false,
      }))
      return response
    } catch (e) {
      const message = parseCommandError(String(e)).message
      set({
        deploymentGateRiskContextError: message,
        isDeploymentGateRiskContextLoading: false,
      })
      return null
    }
  },

  loadMultiRepoExecutiveGovernance: async (query = {}) => {
    const { serverConfig, selectedOrgName } = get()
    if (!serverConfig) return null
    const scopedQuery: MultiRepoExecutiveGovernanceQuery = withSelectedOrg(query, selectedOrgName)

    set({
      isMultiRepoExecutiveGovernanceLoading: true,
      multiRepoExecutiveGovernanceError: null,
    })
    try {
      const response = await tauriInvoke<MultiRepoExecutiveGovernanceResponse>('cmd_server_get_multi_repo_executive_governance', {
        config: serverConfig,
        query: scopedQuery,
      })
      set({
        multiRepoExecutiveGovernance: response,
        multiRepoExecutiveGovernanceUpdatedAt: response.generated_at,
        isMultiRepoExecutiveGovernanceLoading: false,
      })
      return response
    } catch (e) {
      const message = parseCommandError(String(e)).message
      set({
        multiRepoExecutiveGovernanceError: message,
        isMultiRepoExecutiveGovernanceLoading: false,
      })
      return null
    }
  },

  loadChangeRiskEvaluations: async (query = {}) => {
    const { serverConfig, selectedOrgName, changeRiskEvaluationsFilters } = get()
    if (!serverConfig) return null
    const orgName = query.org_name?.trim() || selectedOrgName.trim() || undefined
    const hasReviewStatusFilter = Object.prototype.hasOwnProperty.call(query, 'review_status')
    const reviewStatusFilter = hasReviewStatusFilter
      ? (query.review_status?.trim() as ChangeRiskEvaluationQuery['review_status'] | undefined) || null
      : changeRiskEvaluationsFilters.review_status || null
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
      review_status: reviewStatusFilter,
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

  loadChangeRiskEvaluationReview: async (evaluationId, query = {}) => {
    const { serverConfig, selectedOrgName } = get()
    if (!serverConfig) return null
    const orgName = query.org_name?.trim() || selectedOrgName.trim() || undefined
    const nextQuery: ChangeRiskEvaluationQuery = {
      ...query,
      org_name: orgName ?? null,
    }

    set({ isChangeRiskReviewLoading: true, changeRiskError: null })
    try {
      const response = await tauriInvoke<ChangeRiskEvaluationReviewResponse>('cmd_server_get_change_risk_evaluation_review', {
        config: serverConfig,
        evaluationId: evaluationId.trim(),
        query: nextQuery,
      })
      set({
        changeRiskEvaluationReview: response,
        isChangeRiskReviewLoading: false,
      })
      return response
    } catch (e) {
      const message = parseCommandError(String(e)).message
      set({
        changeRiskError: message,
        isChangeRiskReviewLoading: false,
      })
      return null
    }
  },

  updateChangeRiskEvaluationReview: async (evaluationId, payload) => {
    const { serverConfig, selectedOrgName } = get()
    if (!serverConfig) return null
    const effectiveOrgName = payload.org_name?.trim() || selectedOrgName.trim() || undefined
    const request: ChangeRiskEvaluationReviewRequest = {
      ...payload,
      org_name: effectiveOrgName ?? null,
      review_status: payload.review_status.trim(),
      review_notes: payload.review_notes?.trim() || null,
      mitigation_notes: payload.mitigation_notes?.trim() || null,
      decision_reason: payload.decision_reason?.trim() || null,
    }

    set({ isChangeRiskReviewUpdating: true, changeRiskError: null })
    try {
      const response = await tauriInvoke<ChangeRiskEvaluationReviewResponse>('cmd_server_update_change_risk_evaluation_review', {
        config: serverConfig,
        evaluationId: evaluationId.trim(),
        payload: request,
      })
      set((state) => ({
        changeRiskEvaluationReview: response,
        changeRiskSelectedEvaluation: state.changeRiskSelectedEvaluation
          ? applyChangeRiskReviewToRecord(state.changeRiskSelectedEvaluation, response)
          : state.changeRiskSelectedEvaluation,
        changeRiskEvaluations: state.changeRiskEvaluations
          .map((record) => applyChangeRiskReviewToRecord(record, response))
          .filter((record) =>
            matchesChangeRiskReviewFilter(record, state.changeRiskEvaluationsFilters.review_status),
          ),
        isChangeRiskReviewUpdating: false,
        changeRiskError: null,
      }))
      return response
    } catch (e) {
      const message = parseCommandError(String(e)).message
      set({
        changeRiskError: message,
        isChangeRiskReviewUpdating: false,
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
        changeRiskEvaluations: matchesChangeRiskReviewFilter(record, state.changeRiskEvaluationsFilters.review_status)
          ? [record, ...state.changeRiskEvaluations].slice(0, state.changeRiskEvaluationsFilters.limit ?? 10)
          : state.changeRiskEvaluations,
        changeRiskEvaluationsTotal: matchesChangeRiskReviewFilter(record, state.changeRiskEvaluationsFilters.review_status)
          ? state.changeRiskEvaluationsTotal + 1
          : state.changeRiskEvaluationsTotal,
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

  loadChangeRiskCabPackets: async (query = {}) => {
    const { serverConfig, selectedOrgName, changeRiskCabPacketsFilters } = get()
    if (!serverConfig) return null
    const orgName = query.org_name?.trim() || selectedOrgName.trim() || undefined
    const nextQuery: ChangeRiskCabPacketQuery = {
      ...changeRiskCabPacketsFilters,
      ...query,
      org_name: orgName ?? null,
      status: query.status?.trim() || changeRiskCabPacketsFilters.status || null,
      limit: query.limit ?? changeRiskCabPacketsFilters.limit ?? 10,
      offset: query.offset ?? changeRiskCabPacketsFilters.offset ?? 0,
    }

    set({
      isChangeRiskCabPacketsLoading: true,
      changeRiskError: null,
      changeRiskCabPacketsFilters: nextQuery,
    })
    try {
      const response = await tauriInvoke<ChangeRiskCabPacketListResponse>('cmd_server_list_change_risk_cab_packets', {
        config: serverConfig,
        query: nextQuery,
      })
      set({
        changeRiskCabPackets: response.items,
        changeRiskCabPacketsTotal: response.total,
        isChangeRiskCabPacketsLoading: false,
      })
      return response
    } catch (e) {
      const message = parseCommandError(String(e)).message
      set({
        changeRiskError: message,
        isChangeRiskCabPacketsLoading: false,
      })
      return null
    }
  },

  createChangeRiskCabPacket: async (payload) => {
    const { serverConfig, selectedOrgName } = get()
    if (!serverConfig) return null
    const effectiveOrgName = payload.org_name?.trim() || selectedOrgName.trim() || undefined
    const request: ChangeRiskCabPacketRequest = {
      ...payload,
      org_name: effectiveOrgName ?? null,
      name: payload.name.trim(),
      repository_full_name: payload.repository_full_name?.trim() || null,
      branch: payload.branch?.trim() || null,
      environment: payload.environment?.trim() || null,
      risk_level: payload.risk_level?.trim() || null,
      review_status: payload.review_status?.trim() || null,
      date_range_start: payload.date_range_start ?? null,
      date_range_end: payload.date_range_end ?? null,
      evaluation_ids: (payload.evaluation_ids ?? []).map((item) => item.trim()).filter(Boolean),
      deployment_gate_ids: (payload.deployment_gate_ids ?? []).map((item) => item.trim()).filter(Boolean),
    }

    set({ isChangeRiskCabPacketCreating: true, changeRiskError: null })
    try {
      const response = await tauriInvoke<ChangeRiskCabPacketResponse>('cmd_server_create_change_risk_cab_packet', {
        config: serverConfig,
        payload: request,
      })
      set((state) => ({
        changeRiskCabPackets: applyChangeRiskCabPacketToList(
          state.changeRiskCabPackets,
          response.packet,
        ).slice(0, state.changeRiskCabPacketsFilters.limit ?? 10),
        changeRiskCabPacketsTotal: state.changeRiskCabPacketsTotal + 1,
        changeRiskCabPacket: response,
        changeRiskCabPacketArtifact: response.artifact ?? null,
        isChangeRiskCabPacketCreating: false,
        changeRiskError: null,
      }))
      return response
    } catch (e) {
      const message = parseCommandError(String(e)).message
      set({
        changeRiskError: message,
        isChangeRiskCabPacketCreating: false,
      })
      return null
    }
  },

  getChangeRiskCabPacket: async (packetId, query = {}) => {
    const { serverConfig, selectedOrgName } = get()
    if (!serverConfig) return null
    const orgName = query.org_name?.trim() || selectedOrgName.trim() || undefined
    const nextQuery: ChangeRiskCabPacketQuery = {
      ...query,
      org_name: orgName ?? null,
    }

    set({ isChangeRiskCabPacketsLoading: true, changeRiskError: null })
    try {
      const response = await tauriInvoke<ChangeRiskCabPacketResponse>('cmd_server_get_change_risk_cab_packet', {
        config: serverConfig,
        packetId: packetId.trim(),
        query: nextQuery,
      })
      set({
        changeRiskCabPacket: response,
        changeRiskCabPacketArtifact: response.artifact ?? null,
        isChangeRiskCabPacketsLoading: false,
      })
      return response
    } catch (e) {
      const message = parseCommandError(String(e)).message
      set({
        changeRiskError: message,
        isChangeRiskCabPacketsLoading: false,
      })
      return null
    }
  },

  getChangeRiskCabPacketReview: async (packetId, query = {}) => {
    const { serverConfig, selectedOrgName } = get()
    if (!serverConfig) return null
    const orgName = query.org_name?.trim() || selectedOrgName.trim() || undefined
    const nextQuery: ChangeRiskCabPacketQuery = {
      ...query,
      org_name: orgName ?? null,
    }

    set({ isChangeRiskCabPacketReviewLoading: true, changeRiskError: null })
    try {
      const response = await tauriInvoke<ChangeRiskCabPacketReviewResponse>('cmd_server_get_change_risk_cab_packet_review', {
        config: serverConfig,
        packetId: packetId.trim(),
        query: nextQuery,
      })
      set({ changeRiskCabPacketReview: response, isChangeRiskCabPacketReviewLoading: false })
      return response
    } catch (e) {
      const message = parseCommandError(String(e)).message
      set({
        changeRiskError: message,
        isChangeRiskCabPacketReviewLoading: false,
      })
      return null
    }
  },

  updateChangeRiskCabPacketReview: async (packetId, payload) => {
    const { serverConfig, selectedOrgName } = get()
    if (!serverConfig) return null
    const orgName = payload.org_name?.trim() || selectedOrgName.trim() || undefined
    const request: ChangeRiskCabPacketReviewRequest = {
      org_name: orgName ?? null,
      review_status: payload.review_status.trim(),
      review_notes: payload.review_notes?.trim() || null,
      mitigation_notes: payload.mitigation_notes?.trim() || null,
      decision_reason: payload.decision_reason?.trim() || null,
      follow_up_required: Boolean(payload.follow_up_required),
      follow_up_owner: payload.follow_up_owner?.trim() || null,
    }

    set({ isChangeRiskCabPacketReviewUpdating: true, changeRiskError: null })
    try {
      const response = await tauriInvoke<ChangeRiskCabPacketReviewResponse>('cmd_server_update_change_risk_cab_packet_review', {
        config: serverConfig,
        packetId: packetId.trim(),
        payload: request,
      })
      set((state) => ({
        changeRiskCabPacketReview: response,
        changeRiskCabPackets: state.changeRiskCabPackets.map((packet) =>
          applyChangeRiskCabPacketReview(packet, response),
        ),
        changeRiskCabPacket: state.changeRiskCabPacket
          ? {
              ...state.changeRiskCabPacket,
              packet: applyChangeRiskCabPacketReview(state.changeRiskCabPacket.packet, response),
            }
          : null,
        isChangeRiskCabPacketReviewUpdating: false,
      }))
      return response
    } catch (e) {
      const message = parseCommandError(String(e)).message
      set({
        changeRiskError: message,
        isChangeRiskCabPacketReviewUpdating: false,
      })
      return null
    }
  },

  downloadChangeRiskCabPacket: async (packetId, query = {}) => {
    const { serverConfig, selectedOrgName } = get()
    if (!serverConfig) return null
    const orgName = query.org_name?.trim() || selectedOrgName.trim() || undefined
    const nextQuery: ChangeRiskCabPacketQuery = {
      ...query,
      org_name: orgName ?? null,
    }

    set({ isChangeRiskCabPacketDownloading: true, changeRiskError: null })
    try {
      const artifact = await tauriInvoke<Record<string, unknown>>('cmd_server_download_change_risk_cab_packet', {
        config: serverConfig,
        packetId: packetId.trim(),
        query: nextQuery,
      })
      set((state) => ({
        changeRiskCabPacketArtifact: artifact,
        changeRiskCabPackets: state.changeRiskCabPackets.map((packet) =>
          packet.packet_id === packetId.trim()
            ? {
                ...packet,
                download_count: packet.download_count + 1,
                downloaded_at: Date.now(),
              }
            : packet,
        ),
        isChangeRiskCabPacketDownloading: false,
        changeRiskError: null,
      }))
      return artifact
    } catch (e) {
      const message = parseCommandError(String(e)).message
      set({
        changeRiskError: message,
        isChangeRiskCabPacketDownloading: false,
      })
      return null
    }
  },

  archiveChangeRiskCabPacket: async (packetId, orgNameParam) => {
    const { serverConfig, selectedOrgName } = get()
    if (!serverConfig) return null
    const orgName = orgNameParam?.trim() || selectedOrgName.trim() || undefined
    const payload: ChangeRiskCabPacketRequest = {
      org_name: orgName ?? null,
      name: '',
      evaluation_ids: [],
      deployment_gate_ids: [],
    }

    set({ isChangeRiskCabPacketArchiving: true, changeRiskError: null })
    try {
      const response = await tauriInvoke<ChangeRiskCabPacketResponse>('cmd_server_archive_change_risk_cab_packet', {
        config: serverConfig,
        packetId: packetId.trim(),
        payload,
      })
      set((state) => ({
        changeRiskCabPackets: state.changeRiskCabPackets.map((packet) =>
          packet.packet_id === response.packet.packet_id ? response.packet : packet,
        ),
        changeRiskCabPacket:
          state.changeRiskCabPacket?.packet.packet_id === response.packet.packet_id
            ? response
            : state.changeRiskCabPacket,
        isChangeRiskCabPacketArchiving: false,
        changeRiskError: null,
      }))
      return response
    } catch (e) {
      const message = parseCommandError(String(e)).message
      set({
        changeRiskError: message,
        isChangeRiskCabPacketArchiving: false,
      })
      return null
    }
  },

  createChangeRiskCabDecisionManifest: async (packetId, payload = {}) => {
    const { serverConfig, selectedOrgName } = get()
    if (!serverConfig) return null
    const orgName = payload.org_name?.trim() || selectedOrgName.trim() || undefined
    const request: ChangeRiskCabDecisionManifestRequest = {
      org_name: orgName ?? null,
    }

    set({ isChangeRiskCabDecisionManifestCreating: true, changeRiskError: null })
    try {
      const response = await tauriInvoke<ChangeRiskCabDecisionManifestResponse>('cmd_server_create_change_risk_cab_decision_manifest', {
        config: serverConfig,
        packetId: packetId.trim(),
        payload: request,
      })
      set((state) => ({
        changeRiskCabDecisionManifest: response,
        changeRiskCabDecisionManifestArtifact: response.artifact ?? null,
        changeRiskCabDecisionManifests: applyChangeRiskCabDecisionManifestToList(
          state.changeRiskCabDecisionManifests,
          response.manifest,
        ),
        changeRiskCabDecisionManifestsTotal: state.changeRiskCabDecisionManifestsTotal + 1,
        isChangeRiskCabDecisionManifestCreating: false,
        changeRiskError: null,
      }))
      return response
    } catch (e) {
      const message = parseCommandError(String(e)).message
      set({
        changeRiskError: message,
        isChangeRiskCabDecisionManifestCreating: false,
      })
      return null
    }
  },

  loadChangeRiskCabDecisionManifests: async (packetId, query = {}) => {
    const { serverConfig, selectedOrgName } = get()
    if (!serverConfig) return null
    const orgName = query.org_name?.trim() || selectedOrgName.trim() || undefined
    const nextQuery: ChangeRiskCabDecisionManifestQuery = {
      ...query,
      org_name: orgName ?? null,
      status: query.status?.trim() || null,
      limit: query.limit ?? 10,
      offset: query.offset ?? 0,
    }

    set({ isChangeRiskCabDecisionManifestsLoading: true, changeRiskError: null })
    try {
      const response = await tauriInvoke<ChangeRiskCabDecisionManifestListResponse>('cmd_server_list_change_risk_cab_decision_manifests', {
        config: serverConfig,
        packetId: packetId.trim(),
        query: nextQuery,
      })
      set({
        changeRiskCabDecisionManifests: response.items,
        changeRiskCabDecisionManifestsTotal: response.total,
        isChangeRiskCabDecisionManifestsLoading: false,
        changeRiskError: null,
      })
      return response
    } catch (e) {
      const message = parseCommandError(String(e)).message
      set({
        changeRiskError: message,
        isChangeRiskCabDecisionManifestsLoading: false,
      })
      return null
    }
  },

  getChangeRiskCabDecisionManifest: async (manifestId, query = {}) => {
    const { serverConfig, selectedOrgName } = get()
    if (!serverConfig) return null
    const orgName = query.org_name?.trim() || selectedOrgName.trim() || undefined
    const nextQuery: ChangeRiskCabDecisionManifestQuery = {
      ...query,
      org_name: orgName ?? null,
    }

    set({ isChangeRiskCabDecisionManifestsLoading: true, changeRiskError: null })
    try {
      const response = await tauriInvoke<ChangeRiskCabDecisionManifestResponse>('cmd_server_get_change_risk_cab_decision_manifest', {
        config: serverConfig,
        manifestId: manifestId.trim(),
        query: nextQuery,
      })
      set({
        changeRiskCabDecisionManifest: response,
        changeRiskCabDecisionManifestArtifact: response.artifact ?? null,
        isChangeRiskCabDecisionManifestsLoading: false,
        changeRiskError: null,
      })
      return response
    } catch (e) {
      const message = parseCommandError(String(e)).message
      set({
        changeRiskError: message,
        isChangeRiskCabDecisionManifestsLoading: false,
      })
      return null
    }
  },

  downloadChangeRiskCabDecisionManifest: async (manifestId, query = {}) => {
    const { serverConfig, selectedOrgName } = get()
    if (!serverConfig) return null
    const orgName = query.org_name?.trim() || selectedOrgName.trim() || undefined
    const nextQuery: ChangeRiskCabDecisionManifestQuery = {
      ...query,
      org_name: orgName ?? null,
    }

    set({ isChangeRiskCabDecisionManifestDownloading: true, changeRiskError: null })
    try {
      const artifact = await tauriInvoke<Record<string, unknown>>('cmd_server_download_change_risk_cab_decision_manifest', {
        config: serverConfig,
        manifestId: manifestId.trim(),
        query: nextQuery,
      })
      set((state) => ({
        changeRiskCabDecisionManifestArtifact: artifact,
        changeRiskCabDecisionManifests: state.changeRiskCabDecisionManifests.map((manifest) =>
          manifest.manifest_id === manifestId.trim()
            ? {
                ...manifest,
                download_count: manifest.download_count + 1,
                downloaded_at: Date.now(),
              }
            : manifest,
        ),
        isChangeRiskCabDecisionManifestDownloading: false,
        changeRiskError: null,
      }))
      return artifact
    } catch (e) {
      const message = parseCommandError(String(e)).message
      set({
        changeRiskError: message,
        isChangeRiskCabDecisionManifestDownloading: false,
      })
      return null
    }
  },

  revokeChangeRiskCabDecisionManifest: async (manifestId, orgNameParam) => {
    const { serverConfig, selectedOrgName } = get()
    if (!serverConfig) return null
    const orgName = orgNameParam?.trim() || selectedOrgName.trim() || undefined
    const payload: ChangeRiskCabDecisionManifestRequest = {
      org_name: orgName ?? null,
    }

    set({ isChangeRiskCabDecisionManifestRevoking: true, changeRiskError: null })
    try {
      const response = await tauriInvoke<ChangeRiskCabDecisionManifestResponse>('cmd_server_revoke_change_risk_cab_decision_manifest', {
        config: serverConfig,
        manifestId: manifestId.trim(),
        payload,
      })
      set((state) => ({
        changeRiskCabDecisionManifest: response,
        changeRiskCabDecisionManifests: state.changeRiskCabDecisionManifests.map((manifest) =>
          manifest.manifest_id === response.manifest.manifest_id ? response.manifest : manifest,
        ),
        isChangeRiskCabDecisionManifestRevoking: false,
        changeRiskError: null,
      }))
      return response
    } catch (e) {
      const message = parseCommandError(String(e)).message
      set({
        changeRiskError: message,
        isChangeRiskCabDecisionManifestRevoking: false,
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
