import {
  MAX_TERMINAL_SESSION_COMMANDS,
  appendTerminalSessionCommand,
  applyNativeTerminalInputToDraft,
  type TerminalSessionCommand,
} from '@/components/cli/terminalSessionHistory'

describe('native terminal session history helpers', () => {
  it('captures submitted commands while preserving in-progress draft input', () => {
    const first = applyNativeTerminalInputToDraft('', 'git sta')
    expect(first).toEqual({
      draft: 'git sta',
      submittedCommands: [],
    })

    const second = applyNativeTerminalInputToDraft(first.draft, 'tus\r')
    expect(second).toEqual({
      draft: '',
      submittedCommands: ['git status'],
    })
  })

  it('handles pasted multi-command input without keeping blank submissions', () => {
    const result = applyNativeTerminalInputToDraft('', 'git status\r\rpnpm test\n')

    expect(result.draft).toBe('')
    expect(result.submittedCommands).toEqual(['git status', 'pnpm test'])
  })

  it('handles correction and cancellation controls before submission', () => {
    const corrected = applyNativeTerminalInputToDraft('', 'git stats\u007fus\r')
    expect(corrected.submittedCommands).toEqual(['git status'])

    const cancelled = applyNativeTerminalInputToDraft('dangerous command', '\u0003git status\r')
    expect(cancelled.submittedCommands).toEqual(['git status'])
  })

  it('ignores terminal escape bytes instead of recording navigation as commands', () => {
    const result = applyNativeTerminalInputToDraft('', '\u001b[Agit status\r')

    expect(result.submittedCommands).toEqual(['git status'])
  })

  it('stores newest commands first with repo, branch, shell, and session limit metadata', () => {
    let history: TerminalSessionCommand[] = []

    for (let index = 0; index < MAX_TERMINAL_SESSION_COMMANDS + 5; index += 1) {
      history = appendTerminalSessionCommand(history, {
        command: `git status ${index}`,
        repoPath: 'C:/work/GitGov',
        branch: 'main',
        shell: 'powershell',
        createdAt: 1_700_000_000_000 + index,
      })
    }

    expect(history).toHaveLength(MAX_TERMINAL_SESSION_COMMANDS)
    expect(history[0]).toMatchObject({
      command: `git status ${MAX_TERMINAL_SESSION_COMMANDS + 4}`,
      repoPath: 'C:/work/GitGov',
      branch: 'main',
      shell: 'powershell',
    })
    expect(history[history.length - 1]?.command).toBe('git status 5')
  })

  it('does not store empty commands and fills safe labels for missing metadata', () => {
    const unchanged = appendTerminalSessionCommand([], {
      command: '   ',
      createdAt: 1_700_000_000_000,
    })
    expect(unchanged).toEqual([])

    const history = appendTerminalSessionCommand([], {
      command: 'git log',
      createdAt: 1_700_000_000_000,
    })
    expect(history[0]).toMatchObject({
      command: 'git log',
      repoPath: 'No repository selected',
      branch: 'unknown',
      shell: 'shell',
    })
  })
})
