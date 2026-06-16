import { execFileSync } from 'node:child_process'
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { describe, expect, it } from 'vitest'
import { detectGitContext, parseGitHubRemote } from '../src/git'

function createTempDir(prefix: string): string {
  return mkdtempSync(join(tmpdir(), prefix))
}

function git(cwd: string, args: string[]): void {
  execFileSync('git', args, { cwd, stdio: 'ignore', windowsHide: true })
}

describe('parseGitHubRemote', () => {
  it('parses common GitHub remote URL formats and rejects non-GitHub remotes', () => {
    expect(parseGitHubRemote('git@github.com:yohandry10/Git-Gov.git')).toBe('yohandry10/Git-Gov')
    expect(parseGitHubRemote('https://github.com/yohandry10/Git-Gov.git')).toBe('yohandry10/Git-Gov')
    expect(parseGitHubRemote('ssh://git@github.com/yohandry10/Git-Gov.git')).toBe('yohandry10/Git-Gov')
    expect(parseGitHubRemote('https://gitlab.com/yohandry10/Git-Gov.git')).toBeNull()
    expect(parseGitHubRemote('')).toBeNull()
  })
})

describe('detectGitContext', () => {
  it('detects repo and branch from a real local Git checkout without mutating it', async () => {
    const dir = createTempDir('gitgov-vscode-git-')
    try {
      git(dir, ['init', '-b', 'feature/KAN-136-test'])
      git(dir, ['remote', 'add', 'origin', 'git@github.com:yohandry10/Git-Gov.git'])
      writeFileSync(join(dir, 'README.md'), 'test\n')

      const context = await detectGitContext(dir)

      expect(context.isGitRepository).toBe(true)
      expect(context.repositoryFullName).toBe('yohandry10/Git-Gov')
      expect(context.branch).toBe('feature/KAN-136-test')
      expect(context.rootPath).toBe(dir.replace(/\\/g, '/'))
      expect(context.error).toBeNull()
    } finally {
      rmSync(dir, { recursive: true, force: true })
    }
  })

  it('returns a safe non-git state for ordinary folders', async () => {
    const dir = createTempDir('gitgov-vscode-non-git-')
    try {
      const context = await detectGitContext(dir)

      expect(context.isGitRepository).toBe(false)
      expect(context.repositoryFullName).toBeNull()
      expect(context.branch).toBeNull()
      expect(context.error).toBe('No Git repository detected.')
    } finally {
      rmSync(dir, { recursive: true, force: true })
    }
  })
})
