import { execFile } from 'node:child_process'
import { promisify } from 'node:util'
import type { GitContext } from './types'

const execFileAsync = promisify(execFile)

export function parseGitHubRemote(remoteUrl?: string | null): string | null {
  const trimmed = remoteUrl?.trim()
  if (!trimmed) return null

  const match = trimmed.match(/github\.com[:/](?<owner>[^/\s:]+)\/(?<repo>[^/\s]+?)(?:\.git)?(?:[#?].*)?$/i)
  if (!match?.groups) return null

  const owner = match.groups.owner.trim()
  const repo = match.groups.repo.replace(/\.git$/i, '').trim()
  if (!owner || !repo) return null
  return `${owner}/${repo}`
}

async function git(args: string[], cwd: string): Promise<string> {
  const { stdout } = await execFileAsync('git', args, {
    cwd,
    windowsHide: true,
    timeout: 5000,
    maxBuffer: 1024 * 1024,
  })
  return stdout.trim()
}

export async function detectGitContext(workspacePath: string | null | undefined): Promise<GitContext> {
  const cwd = workspacePath?.trim()
  if (!cwd) {
    return {
      isGitRepository: false,
      repositoryFullName: null,
      branch: null,
      rootPath: null,
      error: 'No workspace folder is open.',
    }
  }

  try {
    const rootPath = await git(['rev-parse', '--show-toplevel'], cwd)
    const branch = await git(['branch', '--show-current'], rootPath)
    let remoteUrl: string | null = null
    try {
      remoteUrl = await git(['remote', 'get-url', 'origin'], rootPath)
    } catch {
      remoteUrl = null
    }

    return {
      isGitRepository: true,
      repositoryFullName: parseGitHubRemote(remoteUrl),
      branch: branch || null,
      rootPath,
      error: null,
    }
  } catch {
    return {
      isGitRepository: false,
      repositoryFullName: null,
      branch: null,
      rootPath: null,
      error: 'No Git repository detected.',
    }
  }
}
