import type { StoreApi } from 'zustand'
import type { ControlPlaneActions, ControlPlaneState } from './types'

export type ControlPlaneStore = ControlPlaneState & ControlPlaneActions
export type ControlPlaneSet = StoreApi<ControlPlaneStore>['setState']
export type ControlPlaneGet = StoreApi<ControlPlaneStore>['getState']
