import {
  isNativeTerminalDisabledError,
  nativeTerminalDisabledEnv,
  nativeTerminalDisabledMessage,
} from '@/components/cli/terminalStatus'

describe('terminal native status helpers', () => {
  it('detects explicit native terminal opt-out errors', () => {
    expect(
      isNativeTerminalDisabledError(
        `Native terminal is disabled by ${nativeTerminalDisabledEnv}=false`,
      ),
    ).toBe(true)
  })

  it('does not treat unrelated terminal failures as opt-out', () => {
    expect(isNativeTerminalDisabledError('Failed to spawn native terminal shell')).toBe(false)
  })

  it('tells the operator how to restore the native terminal', () => {
    expect(nativeTerminalDisabledMessage).toContain(nativeTerminalDisabledEnv)
    expect(nativeTerminalDisabledMessage).toContain('restart GitGov Desktop')
  })
})
