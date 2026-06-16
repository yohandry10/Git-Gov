import { useEffect, useMemo, useState } from 'react'
import { ShieldCheck } from 'lucide-react'
import { parseCommandError, tauriInvoke } from '@/lib/tauri'
import type { RepoValidation } from '@/lib/types'
import type {
  DeploymentGateAuthorizationListResponse,
  ServerConfig,
} from '@/store/useControlPlaneStore/types'
import type { NativeTerminalGitContext } from './terminalGitContext'
import { buildTerminalGovernanceTarget } from './terminalGovernanceContext'
import {
  summarizeTerminalBranchGateStatus,
  terminalBranchGateErrorStatus,
  terminalBranchGateInitialStatus,
  type TerminalBranchGateStatusSummary,
  type TerminalBranchGateStatusTone,
} from './terminalBranchGateStatus'

interface TerminalBranchGateStatusBadgeProps {
  context: NativeTerminalGitContext | null
  validation: RepoValidation | null
  currentBranch: string | null
  serverConfig: ServerConfig | null
  selectedOrgName: string
}

const toneClass: Record<TerminalBranchGateStatusTone, string> = {
  ready: 'border-success-500/25 bg-success-500/10 text-success-200',
  review: 'border-warning-500/25 bg-warning-500/10 text-warning-200',
  muted: 'border-surface-700 bg-surface-900 text-surface-400',
}

export function TerminalBranchGateStatusBadge({
  context,
  validation,
  currentBranch,
  serverConfig,
  selectedOrgName,
}: TerminalBranchGateStatusBadgeProps) {
  const target = useMemo(
    () => buildTerminalGovernanceTarget(context, validation, currentBranch),
    [context, currentBranch, validation],
  )
  const baseSummary = useMemo(
    () => terminalBranchGateInitialStatus(target, serverConfig),
    [serverConfig, target],
  )
  const requestKey = [
    serverConfig?.url ?? 'no-config',
    selectedOrgName.trim(),
    target.status,
    target.repositoryFullName ?? '',
    target.branch ?? '',
  ].join('|')
  const [loadedSummary, setLoadedSummary] = useState<{
    key: string
    summary: TerminalBranchGateStatusSummary
  } | null>(null)
  const summary = loadedSummary?.key === requestKey ? loadedSummary.summary : baseSummary

  useEffect(() => {
    if (!baseSummary.visible || !serverConfig || target.status !== 'ready' || !target.repositoryFullName) {
      return
    }

    let cancelled = false
    const scopedOrgName = selectedOrgName.trim() || null

    async function loadLatestGate() {
      try {
        const response = await tauriInvoke<DeploymentGateAuthorizationListResponse>(
          'cmd_server_list_deployment_gate_authorizations',
          {
            config: serverConfig,
            query: {
              org_name: scopedOrgName,
              repository_full_name: target.repositoryFullName,
              branch: target.branch,
              limit: 1,
              offset: 0,
            },
          },
        )

        if (!cancelled) {
          setLoadedSummary({
            key: requestKey,
            summary: summarizeTerminalBranchGateStatus(response.items[0] ?? null),
          })
        }
      } catch (error) {
        if (!cancelled) {
          setLoadedSummary({
            key: requestKey,
            summary: terminalBranchGateErrorStatus(parseCommandError(String(error)).message),
          })
        }
      }
    }

    void loadLatestGate()

    return () => {
      cancelled = true
    }
  }, [baseSummary.visible, requestKey, selectedOrgName, serverConfig, target])

  if (!summary.visible) return null

  return (
    <span
      className={`inline-flex shrink-0 items-center gap-1 rounded border px-1.5 py-0.5 text-[9px] uppercase tracking-wider ${toneClass[summary.tone]}`}
      title={summary.title}
      aria-label={summary.title}
    >
      <ShieldCheck size={10} />
      {summary.label}
    </span>
  )
}
