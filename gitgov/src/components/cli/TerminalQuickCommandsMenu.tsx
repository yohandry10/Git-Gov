import { Keyboard } from 'lucide-react'
import type { NativeTerminalGitContext } from './terminalGitContext'
import {
  terminalToolContextDetectedLabels,
  terminalToolContextSafetyLabel,
  type NativeTerminalToolContext,
} from './terminalToolContext'
import {
  buildTerminalQuickCommandViews,
  terminalQuickCommandGroupLabel,
  type TerminalQuickCommand,
  type TerminalQuickCommandView,
} from './terminalQuickCommands'

interface TerminalQuickCommandsMenuProps {
  context: NativeTerminalGitContext | null
  toolContext?: NativeTerminalToolContext | null
  disabled: boolean
  isOpen: boolean
  recentCommands: string[]
  onToggle: () => void
  onInsert: (command: TerminalQuickCommand) => void
}

export function TerminalQuickCommandsMenu({
  context,
  toolContext = null,
  disabled,
  isOpen,
  recentCommands,
  onToggle,
  onInsert,
}: TerminalQuickCommandsMenuProps) {
  const quickCommands = buildTerminalQuickCommandViews(context, toolContext)
  const detectedToolLabels = terminalToolContextDetectedLabels(toolContext)
  const hasToolContext = toolContext !== null
  const workspaceCommands = quickCommands.filter((quickCommand) => quickCommand.availableInWorkspace)
  const otherCommands = quickCommands.filter((quickCommand) => !quickCommand.availableInWorkspace)
  const groupedQuickCommands = hasToolContext
    ? [{ key: 'other-safe', label: 'Other safe commands', commands: otherCommands }]
    : otherCommands.reduce<Array<{
    key: TerminalQuickCommand['group']
    label: string
    commands: TerminalQuickCommandView[]
  }>>((groups, quickCommand) => {
    const existing = groups.find((group) => group.key === quickCommand.group)
    if (existing) {
      existing.commands.push(quickCommand)
    } else {
      groups.push({
        key: quickCommand.group,
        label: terminalQuickCommandGroupLabel(quickCommand.group),
        commands: [quickCommand],
      })
    }
    return groups
  }, [])

  return (
    <div className="relative">
      <button
        type="button"
        onClick={onToggle}
        disabled={disabled}
        className={`inline-flex items-center gap-1 rounded border px-1.5 py-0.5 text-[9px] uppercase tracking-wider transition-colors ${
          isOpen
            ? 'border-brand-500/40 bg-brand-500/15 text-brand-300'
            : 'border-surface-700 bg-surface-900 text-surface-400 hover:text-surface-200 disabled:cursor-not-allowed disabled:opacity-50'
        }`}
        title="Insert a safe read-only command into the native terminal without running it"
      >
        <Keyboard size={10} />
        Quick
      </button>

      {isOpen && (
        <div className="absolute right-0 top-7 z-20 w-80 rounded border border-surface-700 bg-surface-950 p-2 shadow-xl shadow-black/40">
          <div className="mb-2 flex items-center justify-between gap-2">
            <span className="text-[10px] font-medium uppercase tracking-wider text-surface-300">
              Safe quick commands
            </span>
            <span className="rounded border border-success-500/30 bg-success-500/10 px-1.5 py-0.5 text-[8px] uppercase tracking-wider text-success-300">
              insert only
            </span>
          </div>

          {hasToolContext && (
            <div className="mb-2 flex flex-wrap gap-1">
              {(detectedToolLabels.length > 0 ? detectedToolLabels : [terminalToolContextSafetyLabel(toolContext)])
                .map((label) => (
                  <span
                    key={label}
                    className="rounded border border-surface-800 bg-surface-900 px-1.5 py-0.5 text-[8px] uppercase tracking-wider text-surface-500"
                  >
                    {label}
                  </span>
                ))}
            </div>
          )}

          <div className="space-y-2">
            {workspaceCommands.length > 0 && (
              <section>
                <div className="mb-1 text-[8px] uppercase tracking-wider text-success-300">
                  Available in this workspace
                </div>
                <div className="space-y-1.5">
                  {workspaceCommands.map((quickCommand) => (
                    <button
                      key={quickCommand.id}
                      type="button"
                      disabled={quickCommand.disabled}
                      onClick={() => onInsert(quickCommand)}
                      className="block w-full rounded border border-success-500/20 bg-success-500/5 px-2 py-1.5 text-left transition-colors hover:border-success-500/40 disabled:cursor-not-allowed disabled:opacity-50"
                      title={quickCommand.disabledReason ?? `Insert ${quickCommand.command}`}
                    >
                      <div className="flex min-w-0 items-center gap-2">
                        <span className="text-[10px] font-medium text-surface-200">
                          {quickCommand.label}
                        </span>
                        <code className="ml-auto max-w-[190px] truncate font-mono text-[9px] text-brand-200">
                          {quickCommand.command}
                        </code>
                      </div>
                      <p className="mt-1 text-[9px] leading-snug text-surface-500">
                        {quickCommand.disabledReason ?? quickCommand.description}
                      </p>
                    </button>
                  ))}
                </div>
              </section>
            )}

            {groupedQuickCommands.map((group) => (
              <section key={group.key}>
                <div className="mb-1 text-[8px] uppercase tracking-wider text-surface-600">
                  {group.label}
                </div>
                <div className="space-y-1.5">
                  {group.commands.map((quickCommand) => (
                    <button
                      key={quickCommand.id}
                      type="button"
                      disabled={quickCommand.disabled}
                      onClick={() => onInsert(quickCommand)}
                      className="block w-full rounded border border-surface-800 bg-surface-900/80 px-2 py-1.5 text-left transition-colors hover:border-surface-600 disabled:cursor-not-allowed disabled:opacity-50"
                      title={quickCommand.disabledReason ?? `Insert ${quickCommand.command}`}
                    >
                      <div className="flex min-w-0 items-center gap-2">
                        <span className="text-[10px] font-medium text-surface-200">
                          {quickCommand.label}
                        </span>
                        <code className="ml-auto max-w-[190px] truncate font-mono text-[9px] text-brand-200">
                          {quickCommand.command}
                        </code>
                      </div>
                      <p className="mt-1 text-[9px] leading-snug text-surface-500">
                        {quickCommand.disabledReason ?? quickCommand.description}
                      </p>
                    </button>
                  ))}
                </div>
              </section>
            ))}
          </div>

          {recentCommands.length > 0 && (
            <div className="mt-2 border-t border-surface-800 pt-2">
              <div className="mb-1 text-[8px] uppercase tracking-wider text-surface-600">
                Used in this session
              </div>
              <div className="flex flex-wrap gap-1">
                {recentCommands.map((command) => (
                  <span
                    key={command}
                    className="max-w-[140px] truncate rounded border border-surface-800 bg-surface-900 px-1.5 py-0.5 font-mono text-[8px] text-surface-400"
                    title={command}
                  >
                    {command}
                  </span>
                ))}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  )
}
