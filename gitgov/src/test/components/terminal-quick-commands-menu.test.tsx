import { fireEvent, render, screen } from '@testing-library/react'
import type { NativeTerminalGitContext } from '@/components/cli/terminalGitContext'
import type { NativeTerminalToolContext } from '@/components/cli/terminalToolContext'
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

const terraformToolContext: NativeTerminalToolContext = {
  cwd_kind: 'git_repo',
  tools: [
    {
      tool: 'terraform',
      detected: true,
      confidence: 'high',
      reason: 'terraform_files_present',
      safe_command_ids: ['terraform-fmt-check', 'terraform-validate'],
    },
    {
      tool: 'docker-compose',
      detected: false,
      confidence: 'none',
      reason: 'not_detected',
      safe_command_ids: ['docker-compose-services', 'docker-compose-check'],
    },
    {
      tool: 'helm',
      detected: false,
      confidence: 'none',
      reason: 'not_detected',
      safe_command_ids: ['helm-lint-local'],
    },
    {
      tool: 'kubernetes',
      detected: false,
      confidence: 'none',
      reason: 'not_detected',
      safe_command_ids: ['kubectl-current-context', 'kubectl-list-contexts'],
    },
  ],
  scan_limited: false,
  secrets_read: false,
  network_used: false,
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

  it('surfaces detected local tool commands quietly without exposing cwd', () => {
    const onInsert = vi.fn()

    render(
      <TerminalQuickCommandsMenu
        context={gitContext}
        toolContext={terraformToolContext}
        disabled={false}
        isOpen
        recentCommands={[]}
        onToggle={vi.fn()}
        onInsert={onInsert}
      />,
    )

    expect(screen.getByText('Terraform detected')).toBeInTheDocument()
    expect(screen.getByText('Available in this workspace')).toBeInTheDocument()
    expect(screen.getAllByText('Other safe commands')).toHaveLength(1)
    expect(screen.queryByText(/C:\/work/)).not.toBeInTheDocument()

    const commandButton = screen.getByText('terraform validate -no-color').closest('button')
    expect(commandButton).not.toBeNull()
    fireEvent.click(commandButton!)

    expect(onInsert).toHaveBeenCalledTimes(1)
    expect(onInsert.mock.calls[0][0]).toMatchObject({
      command: 'terraform validate -no-color',
      availableInWorkspace: true,
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
