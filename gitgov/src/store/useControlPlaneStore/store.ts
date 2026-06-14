import { create } from 'zustand'
import { createChatActions } from './actions/chat'
import { createComplianceActions } from './actions/compliance'
import { createConnectionActions } from './actions/connection'
import { createDashboardActions } from './actions/dashboard'
import { createEnterpriseActions } from './actions/enterprise'
import { createOrganizationActions } from './actions/organization'
import { createPolicySseActions } from './actions/policy-sse'
import { createInitialControlPlaneState } from './state'
import type { ControlPlaneStore } from './store-types'

export const useControlPlaneStore = create<ControlPlaneStore>((set, get) => ({
  ...createInitialControlPlaneState(),
  ...createConnectionActions(set, get),
  ...createDashboardActions(set, get),
  ...createEnterpriseActions(set, get),
  ...createComplianceActions(set, get),
  ...createOrganizationActions(set, get),
  ...createChatActions(set, get),
  ...createPolicySseActions(set, get),
}))
