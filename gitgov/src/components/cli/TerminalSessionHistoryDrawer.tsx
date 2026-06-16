import type { TerminalSessionCommand } from './terminalSessionHistory'

interface TerminalSessionHistoryDrawerProps {
  commands: TerminalSessionCommand[]
}

function formatSessionCommandTime(ts: number): string {
  return new Date(ts).toLocaleTimeString([], {
    hour12: false,
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  })
}

function shortRepoPath(repoPath: string): string {
  const normalized = repoPath.replace(/\\/g, '/')
  const parts = normalized.split('/').filter(Boolean)
  return parts[parts.length - 1] ?? repoPath
}

export function TerminalSessionHistoryDrawer({ commands }: TerminalSessionHistoryDrawerProps) {
  return (
    <div className="max-h-36 overflow-auto border-b border-surface-800 bg-surface-900/80 px-3 py-2">
      {commands.length === 0 ? (
        <div className="text-[10px] uppercase tracking-wider text-surface-600">
          No native terminal commands in this session
        </div>
      ) : (
        <div className="space-y-1.5">
          {commands.map((entry) => (
            <div
              key={entry.id}
              className="rounded border border-surface-800 bg-surface-900/80 px-2 py-1.5"
            >
              <div className="flex min-w-0 items-center gap-1.5">
                <span className="rounded border border-surface-700 bg-surface-800 px-1.5 py-0.5 text-[8px] uppercase tracking-wider text-surface-400">
                  {entry.shell}
                </span>
                <span className="truncate text-[9px] text-surface-500">
                  {shortRepoPath(entry.repoPath)} · {entry.branch}
                </span>
                <span className="ml-auto shrink-0 text-[9px] text-surface-500">
                  {formatSessionCommandTime(entry.createdAt)}
                </span>
              </div>
              <p className="mt-1 truncate font-mono text-[10px] text-surface-200">
                {entry.command}
              </p>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
