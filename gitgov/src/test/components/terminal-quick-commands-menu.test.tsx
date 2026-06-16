import { fireEvent, render, screen } from '@testing-library/react'
import type { NativeTerminalGitContext } from '@/components/cli/terminalGitContext'
import { TerminalQuickCommandsMenu } from '@/components/cli/TerminalQuickCommandsMenu'

const gitContext: NativeTerminalGitContext = {
  cwd: 'C:/work/customer/GitGov',
  is_git_repo: true,
  is_detached: false,
  repo_name: 'GitGov',
  branch: 'main',
  commit_short: 'abc1234',
  detected_at_ms: 1_700_000_000_000,
}

describe('native terminal quick commands menu', () => {
  it('renders Git and provider/tool command groups without exposing the local cwd', () => {
    render(
      <TerminalQuickCommandsMenu
        context={gitContext}
        disabled={false}
        isOpen
        recentCommands={[]}
        onToggle={vi.fn()}
        onInsert={vi.fn()}
      />,
    )

    expect(screen.getByText('Git inspection')).toBeInTheDocument()
    expect(screen.getByText('Provider / Tool context')).toBeInTheDocument()
    expect(screen.getByText('terraform fmt -check -recursive')).toBeInTheDocument()
    expect(screen.getByText('docker compose config --services')).toBeInTheDocument()
    expect(screen.queryByText(/C:\/work/)).not.toBeInTheDocument()
  })

  it('inserts provider/tool commands only through the existing insert callback', () => {
    const onInsert = vi.fn()

    render(
      <TerminalQuickCommandsMenu
        context={gitContext}
        disabled={false}
        isOpen
        recentCommands={[]}
        onToggle={vi.fn()}
        onInsert={onInsert}
      />,
    )

    const commandText = screen.getByText('terraform fmt -check -recursive')
    const commandButton = commandText.closest('button')
    expect(commandButton).not.toBeNull()
    fireEvent.click(commandButton!)

    expect(onInsert).toHaveBeenCalledTimes(1)
    expect(onInsert.mock.calls[0][0]).toMatchObject({
      command: 'terraform fmt -check -recursive',
      group: 'provider-tool',
      enabled: true,
      requiresNetwork: false,
      mayExposeSecrets: false,
    })
  })

  it('shows disabled reasons for all quick commands outside a Git repository', () => {
    render(
      <TerminalQuickCommandsMenu
        context={{ ...gitContext, is_git_repo: false, repo_name: null, branch: null, commit_short: null }}
        disabled={false}
        isOpen
        recentCommands={[]}
        onToggle={vi.fn()}
        onInsert={vi.fn()}
      />,
    )

    expect(screen.getAllByText('Open a Git repository terminal to insert this command')).toHaveLength(12)
    const commandText = screen.getByText('kubectl config current-context')
    const commandButton = commandText.closest('button')
    expect(commandButton).toBeDisabled()
  })
})
