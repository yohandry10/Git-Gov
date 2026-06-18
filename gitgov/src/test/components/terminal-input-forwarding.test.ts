import { buildNativeTerminalInputForwardingContract } from '@/components/cli/terminalInputForwarding'
import { applyNativeTerminalInputToDraft } from '@/components/cli/terminalSessionHistory'

describe('native terminal input forwarding contract', () => {
  it('forwards manual terminal bytes unchanged while allowing local observation', () => {
    const input = 'status --short\r'

    const observed = applyNativeTerminalInputToDraft('', input)
    const forwarding = buildNativeTerminalInputForwardingContract(input)

    expect(observed).toEqual({
      draft: '',
      submittedCommands: ['status --short'],
    })
    expect(forwarding).toEqual({
      data: input,
      shouldForward: true,
      interception: 'none',
      policyEvaluation: 'not-run',
      mutatesInput: false,
    })
  })

  it('does not rewrite compound-looking or redirected manual input', () => {
    const input = 'opaque-action --change ; follow-up > local-file\r'

    const forwarding = buildNativeTerminalInputForwardingContract(input)

    expect(forwarding.data).toBe(input)
    expect(forwarding.shouldForward).toBe(true)
    expect(forwarding.interception).toBe('none')
    expect(forwarding.policyEvaluation).toBe('not-run')
    expect(forwarding.mutatesInput).toBe(false)
  })

  it('preserves control bytes and pasted multi-line input for the native PTY', () => {
    const input = 'first line\rsecond line\n\u0003'

    const observed = applyNativeTerminalInputToDraft('', input)
    const forwarding = buildNativeTerminalInputForwardingContract(input)

    expect(observed).toEqual({
      draft: '',
      submittedCommands: ['first line', 'second line'],
    })
    expect(forwarding.data).toBe(input)
  })
})
