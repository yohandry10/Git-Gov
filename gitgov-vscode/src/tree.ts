import * as vscode from 'vscode'
import type { GovernanceSnapshot } from './types'

export class GovernanceTreeItem extends vscode.TreeItem {
  constructor(label: string, description?: string, collapsibleState = vscode.TreeItemCollapsibleState.None) {
    super(label, collapsibleState)
    this.description = description
  }
}

export class GovernanceTreeProvider implements vscode.TreeDataProvider<GovernanceTreeItem> {
  private snapshot: GovernanceSnapshot | null = null
  private readonly changed = new vscode.EventEmitter<GovernanceTreeItem | undefined | null | void>()
  readonly onDidChangeTreeData = this.changed.event

  setSnapshot(snapshot: GovernanceSnapshot): void {
    this.snapshot = snapshot
    this.changed.fire()
  }

  getTreeItem(element: GovernanceTreeItem): vscode.TreeItem {
    return element
  }

  getChildren(element?: GovernanceTreeItem): GovernanceTreeItem[] {
    if (element) return []
    if (!this.snapshot) {
      return [new GovernanceTreeItem('GitGov Governance', 'Run refresh to load read-only context')]
    }

    const { git, latestGate, latestRisk, executiveRepository, error } = this.snapshot
    const items = [
      new GovernanceTreeItem('Repository', git.repositoryFullName ?? (git.isGitRepository ? 'GitHub remote missing' : 'No Git repository')),
      new GovernanceTreeItem('Branch', git.branch ?? 'unknown'),
    ]

    if (error) {
      items.push(new GovernanceTreeItem('Status', error))
      return items
    }

    items.push(
      new GovernanceTreeItem('Latest Deployment Gate', latestGate?.decision ?? 'No gate data'),
      new GovernanceTreeItem('Latest Change Risk', latestRisk?.risk_level ?? 'No risk data'),
      new GovernanceTreeItem('Review Status', latestRisk?.review_status ?? executiveRepository?.latest_review_status ?? 'unknown'),
      new GovernanceTreeItem('Executive Posture', executiveRepository?.posture ?? 'No posture'),
    )

    return items
  }
}
