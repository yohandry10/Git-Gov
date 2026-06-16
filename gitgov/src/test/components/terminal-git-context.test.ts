import {
  formatTerminalGitContextLabel,
  shouldRefreshTerminalGitContext,
  terminalGitContextTitle,
  type NativeTerminalGitContext,
} from '@/components/cli/terminalGitContext'

describe('native terminal git context helpers', () => {
  it('refreshes context only for simple directory-change commands', () => {
    expect(shouldRefreshTerminalGitContext('cd ..')).toBe(true)
    expect(shouldRefreshTerminalGitContext('chdir src')).toBe(true)
    expect(shouldRefreshTerminalGitContext('Set-Location "C:/work/GitGov"')).toBe(true)
    expect(shouldRefreshTerminalGitContext('sl ./gitgov')).toBe(true)

    expect(shouldRefreshTerminalGitContext('git status')).toBe(false)
    expect(shouldRefreshTerminalGitContext('cd repo && git status')).toBe(false)
    expect(shouldRefreshTerminalGitContext('cd repo; git status')).toBe(false)
    expect(shouldRefreshTerminalGitContext('cd repo | echo ok')).toBe(false)
  })

  it('formats pending and non-git labels without leaking cwd', () => {
    expect(formatTerminalGitContextLabel(null)).toBe('context pending')
    expect(terminalGitContextTitle(null)).toContain('not been detected')

    const context: NativeTerminalGitContext = {
      cwd: 'C:/Users/PC/Desktop/secret-path',
      is_git_repo: false,
      is_detached: false,
      detected_at_ms: 1_700_000_000_000,
    }

    expect(formatTerminalGitContextLabel(context)).toBe('No git repo')
    expect(formatTerminalGitContextLabel(context)).not.toContain('secret-path')
    expect(terminalGitContextTitle(context)).toContain('not inside a Git repository')
  })

  it('formats branch and detached context labels', () => {
    expect(
      formatTerminalGitContextLabel({
        cwd: 'C:/work/GitGov',
        is_git_repo: true,
        is_detached: false,
        repo_name: 'GitGov',
        branch: 'feature/KAN-133-weird_branch',
        commit_short: 'abc1234',
        detected_at_ms: 1_700_000_000_000,
      }),
    ).toBe('GitGov:feature/KAN-133-weird_branch')

    expect(
      formatTerminalGitContextLabel({
        cwd: 'C:/work/GitGov',
        is_git_repo: true,
        is_detached: true,
        repo_name: 'GitGov',
        branch: null,
        commit_short: 'def5678',
        detected_at_ms: 1_700_000_000_000,
      }),
    ).toBe('GitGov:detached@def5678')
  })
})
