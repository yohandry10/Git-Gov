export interface TerminalCommandDraftResult {
  draft: string
  submittedCommands: string[]
}

export interface TerminalSessionCommand {
  id: string
  command: string
  repoPath: string
  branch: string
  shell: string
  createdAt: number
}

export interface TerminalSessionCommandInput {
  command: string
  repoPath?: string | null
  branch?: string | null
  shell?: string | null
  createdAt?: number
}

export const MAX_TERMINAL_SESSION_COMMANDS = 50

function isPrintableInput(char: string): boolean {
  if (char.length !== 1) return false
  const code = char.charCodeAt(0)
  return code >= 32 && code !== 127
}

export function applyNativeTerminalInputToDraft(
  draft: string,
  input: string,
): TerminalCommandDraftResult {
  let nextDraft = draft
  const submittedCommands: string[] = []

  for (let index = 0; index < input.length; index += 1) {
    const char = input[index]

    if (char === '\r' || char === '\n') {
      const command = nextDraft.trim()
      if (command) {
        submittedCommands.push(command)
      }
      nextDraft = ''
      continue
    }

    if (char === '\u0003') {
      nextDraft = ''
      continue
    }

    if (char === '\b' || char === '\u007f') {
      nextDraft = nextDraft.slice(0, -1)
      continue
    }

    if (char === '\u001b') {
      if (input[index + 1] === '[') {
        index += 2
        while (index < input.length && !/[A-Za-z~]/.test(input[index])) {
          index += 1
        }
      }
      continue
    }

    if (isPrintableInput(char)) {
      nextDraft += char
    }
  }

  return {
    draft: nextDraft,
    submittedCommands,
  }
}

export function appendTerminalSessionCommand(
  history: TerminalSessionCommand[],
  input: TerminalSessionCommandInput,
): TerminalSessionCommand[] {
  const command = input.command.trim()
  if (!command) return history

  const createdAt = input.createdAt ?? Date.now()
  const nextEntry: TerminalSessionCommand = {
    id: `${createdAt}-${Math.random().toString(36).slice(2, 10)}`,
    command,
    repoPath: input.repoPath?.trim() || 'No repository selected',
    branch: input.branch?.trim() || 'unknown',
    shell: input.shell?.trim() || 'shell',
    createdAt,
  }

  const nextHistory = [nextEntry, ...history]
  return nextHistory.length > MAX_TERMINAL_SESSION_COMMANDS
    ? nextHistory.slice(0, MAX_TERMINAL_SESSION_COMMANDS)
    : nextHistory
}
