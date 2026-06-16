import { applyNativeTerminalInputToDraft } from '@/components/cli/terminalSessionHistory'
import type { NativeTerminalGitContext } from '@/components/cli/terminalGitContext'
import {
  SAFE_TERMINAL_QUICK_COMMANDS,
  buildTerminalQuickCommandInsertInput,
  buildTerminalQuickCommandViews,
  isReadOnlyTerminalQuickCommand,
} from '@/components/cli/terminalQuickCommands'

const gitContext: NativeTerminalGitContext = {
  cwd: 'C:/work/GitGov',
  is_git_repo: true,
  is_detached: false,
  repo_name: 'GitGov',
  branch: 'main',
  commit_short: 'abc1234',
  detected_at_ms: 1_700_000_000_000,
}

describe('native terminal quick command helpers', () => {
  it('ships only read-only insertable Git commands in the allowlist', () => {
    expect(SAFE_TERMINAL_QUICK_COMMANDS.map((entry) => entry.command)).toEqual([
      'git status --short',
      'git branch --show-current',
      'git log --oneline -5',
      'git diff --stat',
      'git remote -v',
    ])

    for (const quickCommand of SAFE_TERMINAL_QUICK_COMMANDS) {
      expect(isReadOnlyTerminalQuickCommand(quickCommand.command)).toBe(true)
      expect(quickCommand.command).not.toMatch(/\b(push|pull|merge|rebase|commit|checkout|fetch|deploy|apply)\b/i)
    }
  })

  it('rejects mutating, compound, redirected, and non-git commands', () => {
    const rejected = [
      'git push',
      'git pull',
      'git commit -m test',
      'git checkout main',
      'git fetch --all',
      'git status --short && git push',
      'git status --short; git push',
      'git status --short > out.txt',
      'pnpm test',
      'terraform apply',
    ]

    for (const command of rejected) {
      expect(isReadOnlyTerminalQuickCommand(command)).toBe(false)
      expect(() => buildTerminalQuickCommandInsertInput(command)).toThrow('read-only')
    }
  })

  it('disables git quick commands outside a git repository', () => {
    const nonGitViews = buildTerminalQuickCommandViews({
      ...gitContext,
      is_git_repo: false,
      repo_name: null,
      branch: null,
      commit_short: null,
    })

    expect(nonGitViews).toHaveLength(SAFE_TERMINAL_QUICK_COMMANDS.length)
    expect(nonGitViews.every((entry) => entry.disabled)).toBe(true)
    expect(nonGitViews.every((entry) => entry.disabledReason?.includes('Git repository'))).toBe(true)
  })

  it('enables safe commands in a git repository without exposing cwd in labels', () => {
    const views = buildTerminalQuickCommandViews(gitContext)

    expect(views.every((entry) => !entry.disabled)).toBe(true)
    expect(views.some((entry) => entry.label.includes('C:/work'))).toBe(false)
    expect(views.some((entry) => entry.description.includes('C:/work'))).toBe(false)
  })

  it('builds insert-only text without newline so the command is not auto-run', () => {
    const data = buildTerminalQuickCommandInsertInput(' git status --short ')

    expect(data).toBe('git status --short')
    expect(data).not.toMatch(/[\r\n]/)

    const draft = applyNativeTerminalInputToDraft('', data)
    expect(draft).toEqual({
      draft: 'git status --short',
      submittedCommands: [],
    })

    const submitted = applyNativeTerminalInputToDraft(draft.draft, '\r')
    expect(submitted.submittedCommands).toEqual(['git status --short'])
  })
})
