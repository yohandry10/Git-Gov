import * as vscode from 'vscode'
import { clearApiKey, readApiKey, storeApiKey } from './config'
import { loadGovernanceContext } from './context'
import { detectGitContext } from './git'
import { GovernanceTreeProvider } from './tree'
import type { GitGovConnectionConfig } from './types'

function currentConfig(): GitGovConnectionConfig {
  const config = vscode.workspace.getConfiguration('gitgov')
  return {
    apiUrl: config.get<string>('apiUrl') ?? '',
    orgName: config.get<string>('orgName') ?? '',
  }
}

function currentWorkspacePath(): string | null {
  return vscode.workspace.workspaceFolders?.[0]?.uri.fsPath ?? null
}

async function configureConnection(context: vscode.ExtensionContext): Promise<void> {
  const config = vscode.workspace.getConfiguration('gitgov')
  const current = currentConfig()
  const apiUrl = await vscode.window.showInputBox({
    title: 'GitGov API URL',
    value: current.apiUrl,
    ignoreFocusOut: true,
    prompt: 'Example: https://gitgov-api.onrender.com',
  })
  if (apiUrl === undefined) return

  const orgName = await vscode.window.showInputBox({
    title: 'GitGov Organization',
    value: current.orgName,
    ignoreFocusOut: true,
    prompt: 'Organization scope for read-only governance context.',
  })
  if (orgName === undefined) return

  const apiKey = await vscode.window.showInputBox({
    title: 'GitGov Read-Only API Key',
    password: true,
    ignoreFocusOut: true,
    prompt: 'Stored in VS Code SecretStorage, not in settings.',
  })
  if (apiKey === undefined) return

  await config.update('apiUrl', apiUrl.trim(), vscode.ConfigurationTarget.Global)
  await config.update('orgName', orgName.trim(), vscode.ConfigurationTarget.Global)
  await storeApiKey(context.secrets, apiKey)
  await vscode.window.showInformationMessage('GitGov read-only connection configured.')
}

async function refresh(provider: GovernanceTreeProvider, context: vscode.ExtensionContext): Promise<void> {
  const git = await detectGitContext(currentWorkspacePath())
  const snapshot = await loadGovernanceContext({
    git,
    config: currentConfig(),
    apiKey: await readApiKey(context.secrets),
  })
  provider.setSnapshot(snapshot)

  if (snapshot.error) {
    await vscode.window.showWarningMessage(snapshot.error)
  }
}

export function activate(context: vscode.ExtensionContext): void {
  const provider = new GovernanceTreeProvider()
  context.subscriptions.push(vscode.window.registerTreeDataProvider('gitgovGovernance', provider))
  context.subscriptions.push(vscode.commands.registerCommand('gitgov.refreshGovernanceContext', () => refresh(provider, context)))
  context.subscriptions.push(vscode.commands.registerCommand('gitgov.configureConnection', () => configureConnection(context)))
  context.subscriptions.push(vscode.commands.registerCommand('gitgov.clearConnection', async () => {
    await clearApiKey(context.secrets)
    await vscode.window.showInformationMessage('GitGov read-only API key cleared.')
    await refresh(provider, context)
  }))

  void refresh(provider, context)
}

export function deactivate(): void {
  // No background polling or provider mutation to clean up.
}
