export const controlPlaneStoreRuntime = {
  checkConnectionInFlight: null as Promise<void> | null,
  sseUnlisteners: [] as Array<() => void>,
  sseReconnectTimer: null as ReturnType<typeof setTimeout> | null,
  refreshForCurrentRoleInFlight: null as Promise<void> | null,
  loadLogsIncrementalInFlight: null as Promise<void> | null,
  loadLogsIncrementalInFlightLimit: 0,
  lastHeavyDashboardRefreshAt: 0,
}
