import { applyNativeTerminalInputToDraft } from '@/components/cli/terminalSessionHistory'
import type { NativeTerminalGitContext } from '@/components/cli/terminalGitContext'
import type { NativeTerminalToolContext } from '@/components/cli/terminalToolContext'
import {
  SAFE_TERMINAL_QUICK_COMMANDS,
  buildTerminalDisabledActionPreviews,
  buildTerminalQuickCommandInsertInput,
  buildTerminalQuickCommandViews,
  isReadOnlyTerminalQuickCommand,
  terminalQuickCommandGroupLabel,
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

describe('native terminal quick command helpers', () => {
  it('ships only read-only insertable Git commands in the allowlist', () => {
    expect(SAFE_TERMINAL_QUICK_COMMANDS.map((entry) => entry.command)).toEqual([
      'git status --short',
      'git branch --show-current',
      'git log --oneline -5',
      'git diff --stat',
      'git remote',
      'terraform fmt -check -recursive',
      'terraform validate -no-color',
      'kubectl config current-context',
      'kubectl config get-contexts',
      'docker compose config --services',
      'docker compose config --quiet',
      'helm lint .',
    ])

    for (const quickCommand of SAFE_TERMINAL_QUICK_COMMANDS) {
      expect(isReadOnlyTerminalQuickCommand(quickCommand.command)).toBe(true)
      expect(quickCommand.enabled).toBe(true)
      expect(quickCommand.safetyLevel).toBe('local-read-only')
      expect(quickCommand.requiresNetwork).toBe(false)
      expect(quickCommand.mayExposeSecrets).toBe(false)
      expect(quickCommand.command).not.toMatch(
        /\b(push|pull|merge|rebase|commit|checkout|fetch|deploy|apply|delete|destroy|install|upgrade|uninstall|up|down)\b/i,
      )
    }
  })

  it('labels Git and provider/tool command groups without path leakage', () => {
    expect(terminalQuickCommandGroupLabel('git')).toBe('Git inspection')
    expect(terminalQuickCommandGroupLabel('provider-tool')).toBe('Provider / Tool context')

    const views = buildTerminalQuickCommandViews(gitContext)
    expect(views.some((entry) => entry.group === 'git')).toBe(true)
    expect(views.some((entry) => entry.group === 'provider-tool')).toBe(true)
    expect(views.some((entry) => entry.label.includes('C:/work'))).toBe(false)
    expect(views.some((entry) => entry.description.includes('C:/work'))).toBe(false)
  })

  it('rejects mutating, compound, redirected, networked, secret-exposing, and non-registry commands', () => {
    const rejected = [
      'git push',
      'git pull',
      'git commit -m test',
      'git checkout main',
      'git fetch --all',
      'git status',
      'git remote -v',
      'git status --short && git push',
      'git status --short; git push',
      'git status --short > out.txt',
      'terraform plan',
      'terraform apply',
      'terraform destroy',
      'terraform output -json',
      'kubectl get pods',
      'kubectl apply -f deployment.yml',
      'kubectl delete pod api',
      'helm install app .',
      'helm upgrade app .',
      'helm uninstall app',
      'docker compose up',
      'docker compose down',
      'docker compose logs',
      'aws sts get-caller-identity',
      'az account show',
      'gcloud config list',
      'vercel deploy',
      'render services list',
      'env',
      'printenv',
      'cat .env',
      'pnpm test',
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
    expect(views.every((entry) => !entry.availableInWorkspace)).toBe(true)
    expect(views.filter((entry) => entry.group === 'provider-tool')).toHaveLength(7)
    expect(views.find((entry) => entry.command === 'terraform fmt -check -recursive')).toMatchObject({
      tool: 'terraform',
      requiresNetwork: false,
      mayExposeSecrets: false,
      disabled: false,
    })
    expect(views.find((entry) => entry.command === 'kubectl config current-context')).toMatchObject({
      tool: 'kubernetes',
      requiresNetwork: false,
      mayExposeSecrets: false,
      disabled: false,
    })
  })

  it('marks detected provider/tool commands as available without disabling other safe commands', () => {
    const views = buildTerminalQuickCommandViews(gitContext, terraformToolContext)
    const terraformViews = views.filter((entry) => entry.tool === 'terraform')
    const dockerViews = views.filter((entry) => entry.tool === 'docker-compose')

    expect(terraformViews).toHaveLength(2)
    expect(terraformViews.every((entry) => entry.availableInWorkspace)).toBe(true)
    expect(terraformViews.every((entry) => !entry.disabled)).toBe(true)
    expect(dockerViews.every((entry) => !entry.availableInWorkspace)).toBe(true)
    expect(dockerViews.every((entry) => !entry.disabled)).toBe(true)
  })

  it('does not trust tool context that reports secret or network usage', () => {
    const unsafeContext: NativeTerminalToolContext = {
      ...terraformToolContext,
      secrets_read: true,
      network_used: true,
    }
    const views = buildTerminalQuickCommandViews(gitContext, unsafeContext)

    expect(views.every((entry) => !entry.availableInWorkspace)).toBe(true)
    expect(views.every((entry) => !entry.disabled)).toBe(true)
  })

  it('shows quiet disabled action previews only when local tools are detected', () => {
    expect(buildTerminalDisabledActionPreviews(null)).toEqual([])
    expect(buildTerminalDisabledActionPreviews({
      ...terraformToolContext,
      tools: terraformToolContext.tools.map((tool) => ({ ...tool, detected: false })),
    })).toEqual([])

    const previews = buildTerminalDisabledActionPreviews(terraformToolContext)

    expect(previews).toHaveLength(4)
    expect(previews.map((preview) => preview.id)).toEqual([
      'state-changing-tool-actions',
      'cloud-provider-api-actions',
      'secret-or-value-inspection',
      'repository-write-actions',
    ])

    for (const preview of previews) {
      expect('command' in preview).toBe(false)
      expect(`${preview.label} ${preview.reason} ${preview.guardrail}`).not.toMatch(
        /\b(git push|terraform apply|terraform destroy|kubectl apply|helm install|docker compose up|cat \.env|printenv|aws |az |gcloud |vercel deploy|render services)\b/i,
      )
    }
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

  it('builds provider/tool insert-only text without newline so it is not auto-run', () => {
    const data = buildTerminalQuickCommandInsertInput(' terraform fmt -check -recursive ')

    expect(data).toBe('terraform fmt -check -recursive')
    expect(data).not.toMatch(/[\r\n]/)

    const draft = applyNativeTerminalInputToDraft('', data)
    expect(draft).toEqual({
      draft: 'terraform fmt -check -recursive',
      submittedCommands: [],
    })

    const dockerData = buildTerminalQuickCommandInsertInput('docker compose config --services')
    expect(dockerData).toBe('docker compose config --services')
    expect(dockerData).not.toMatch(/[\r\n]/)
  })
})
