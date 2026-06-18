export interface NativeTerminalInputForwardingContract {
  data: string
  shouldForward: true
  interception: 'none'
  policyEvaluation: 'not-run'
  mutatesInput: false
}

export function buildNativeTerminalInputForwardingContract(
  data: string,
): NativeTerminalInputForwardingContract {
  return {
    data,
    shouldForward: true,
    interception: 'none',
    policyEvaluation: 'not-run',
    mutatesInput: false,
  }
}
