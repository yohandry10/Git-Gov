import { useCallback, useMemo, useState } from 'react'
import { BarChart3, RefreshCw, ShieldCheck } from 'lucide-react'
import { parseCommandError, tauriInvoke } from '@/lib/tauri'
import type { RepoValidation } from '@/lib/types'
import type {
  ChangeRiskEvaluationListResponse,
  DeploymentGateAuthorizationListResponse,
  MultiRepoExecutiveGovernanceResponse,
  ServerConfig,
} from '@/store/useControlPlaneStore/types'
import type { NativeTerminalGitContext } from './terminalGitContext'
import {
  buildTerminalGovernanceTarget,
  terminalGovernanceEmptyState,
  terminalGovernanceHasEvidence,
  type TerminalGovernanceSnapshot,
} from './terminalGovernanceContext'

interface TerminalGovernanceContextPanelProps {
  context: NativeTerminalGitContext | null
  validation: RepoValidation | null
  currentBranch: string | null
  serverConfig: ServerConfig | null
  selectedOrgName: string
  connectionStatus: 'connected' | 'disconnected' | 'maintenance' | 'checking'
}

function shortValue(value?: string | null, length = 10): string {
  if (!value) return 'none'
  return value.length > length ? value.slice(0, length) : value
}

function classifyError(error: string): string {
  const parsed = parseCommandError(error)
  if (/401|403|denied|unauthorized|forbidden|permission/i.test(`${parsed.code} ${parsed.message}`)) {
    return 'Permission denied for this Control Plane context.'
  }
  return parsed.message || 'Governance context is unavailable.'
}

export function TerminalGovernanceContextPanel({
  context,
  validation,
  currentBranch,
  serverConfig,
  selectedOrgName,
  connectionStatus,
}: TerminalGovernanceContextPanelProps) {
  const [isOpen, setIsOpen] = useState(false)
  const [isLoading, setIsLoading] = useState(false)
  const [snapshot, setSnapshot] = useState<TerminalGovernanceSnapshot | null>(null)

  const target = useMemo(
    () => buildTerminalGovernanceTarget(context, validation, currentBranch),
    [context, currentBranch, validation],
  )
  const emptyState = terminalGovernanceEmptyState(target)

  const loadContext = useCallback(async () => {
    if (!serverConfig) {
      setSnapshot({
        target,
        latestGate: null,
        latestRisk: null,
        executiveRepository: null,
        providerHealth: connectionStatus,
        error: 'Control Plane is not configured.',
      })
      return
    }

    if (target.status !== 'ready' || !target.repositoryFullName) {
      setSnapshot({
        target,
        latestGate: null,
        latestRisk: null,
        executiveRepository: null,
        providerHealth: connectionStatus,
        error: emptyState,
      })
      return
    }

    setIsLoading(true)
    const scopedOrgName = selectedOrgName.trim() || null
    try {
      const [gateResponse, riskResponse, executiveResponse] = await Promise.all([
        tauriInvoke<DeploymentGateAuthorizationListResponse>('cmd_server_list_deployment_gate_authorizations', {
          config: serverConfig,
          query: {
            org_name: scopedOrgName,
            repository_full_name: target.repositoryFullName,
            branch: target.branch,
            limit: 1,
            offset: 0,
          },
        }),
        tauriInvoke<ChangeRiskEvaluationListResponse>('cmd_server_list_change_risk_evaluations', {
          config: serverConfig,
          query: {
            org_name: scopedOrgName,
            repository_full_name: target.repositoryFullName,
            branch: target.branch,
            limit: 1,
            offset: 0,
          },
        }),
        tauriInvoke<MultiRepoExecutiveGovernanceResponse>('cmd_server_get_multi_repo_executive_governance', {
          config: serverConfig,
          query: {
            org_name: scopedOrgName,
            repository: target.repositoryFullName,
            limit: 1,
            offset: 0,
          },
        }),
      ])

      setSnapshot({
        target,
        latestGate: gateResponse.items[0] ?? null,
        latestRisk: riskResponse.items[0] ?? null,
        executiveRepository: executiveResponse.repositories[0] ?? null,
        providerHealth: connectionStatus,
        error: null,
      })
    } catch (error) {
      setSnapshot({
        target,
        latestGate: null,
        latestRisk: null,
        executiveRepository: null,
        providerHealth: connectionStatus,
        error: classifyError(String(error)),
      })
    } finally {
      setIsLoading(false)
    }
  }, [connectionStatus, emptyState, selectedOrgName, serverConfig, target])

  const toggle = () => {
    setIsOpen((current) => {
      const next = !current
      if (next) {
        void loadContext()
      }
      return next
    })
  }

  const loadedSnapshot =
    snapshot?.target.repositoryFullName === target.repositoryFullName && snapshot?.target.branch === target.branch
      ? snapshot
      : null
  const hasEvidence = loadedSnapshot ? terminalGovernanceHasEvidence(loadedSnapshot) : false

  return (
    <div className="relative">
      <button
        type="button"
        onClick={toggle}
        className={`inline-flex items-center gap-1 rounded border px-1.5 py-0.5 text-[9px] uppercase tracking-wider transition-colors ${
          isOpen
            ? 'border-brand-500/40 bg-brand-500/15 text-brand-300'
            : 'border-surface-700 bg-surface-900 text-surface-400 hover:text-surface-200'
        }`}
        title="Show read-only governance context for this terminal repository"
      >
        <ShieldCheck size={10} />
        Context
      </button>

      {isOpen && (
        <div className="absolute right-0 top-7 z-20 w-96 rounded border border-surface-700 bg-surface-950 p-2 shadow-xl shadow-black/40">
          <div className="mb-2 flex items-center justify-between gap-2">
            <span className="text-[10px] font-medium uppercase tracking-wider text-surface-300">
              Governance context
            </span>
            <button
              type="button"
              onClick={() => void loadContext()}
              disabled={isLoading}
              className="inline-flex items-center gap-1 rounded border border-surface-700 bg-surface-900 px-1.5 py-0.5 text-[8px] uppercase tracking-wider text-surface-400 hover:text-surface-200 disabled:cursor-not-allowed disabled:opacity-50"
              title="Refresh read-only governance context"
            >
              <RefreshCw size={9} />
              {isLoading ? 'Loading' : 'Refresh'}
            </button>
          </div>

          <div className="rounded border border-warning-500/20 bg-warning-500/8 px-2 py-1 text-[9px] text-warning-100">
            Read-only context. Does not approve, block, certify, deploy, or execute commands.
          </div>

          <div className="mt-2 grid grid-cols-2 gap-1.5 text-[10px]">
            <div className="rounded border border-surface-800 bg-surface-900/80 p-2">
              <div className="text-surface-500">Repository</div>
              <div className="mt-1 truncate text-surface-200" title={target.repositoryFullName ?? target.repoLabel}>
                {target.repositoryFullName ?? target.repoLabel}
              </div>
            </div>
            <div className="rounded border border-surface-800 bg-surface-900/80 p-2">
              <div className="text-surface-500">Branch</div>
              <div className="mt-1 truncate text-surface-200">{target.branch ?? 'unknown'}</div>
            </div>
          </div>

          {(emptyState || loadedSnapshot?.error) && (
            <div className="mt-2 rounded border border-surface-800 bg-surface-900/80 px-2 py-1.5 text-[10px] text-surface-400">
              {loadedSnapshot?.error ?? emptyState}
            </div>
          )}

          {loadedSnapshot && !loadedSnapshot.error && (
            <div className="mt-2 space-y-1.5">
              <div className="grid grid-cols-3 gap-1.5">
                <div className="rounded border border-surface-800 bg-surface-900/80 p-2">
                  <div className="text-[9px] text-surface-500">Latest gate</div>
                  <div className="mt-1 truncate text-[10px] text-surface-200">
                    {loadedSnapshot.latestGate?.decision ?? 'No gate data'}
                  </div>
                  <div className="mt-1 truncate font-mono text-[9px] text-surface-500">
                    {shortValue(loadedSnapshot.latestGate?.authorization_id, 14)}
                  </div>
                </div>
                <div className="rounded border border-surface-800 bg-surface-900/80 p-2">
                  <div className="text-[9px] text-surface-500">Latest risk</div>
                  <div className="mt-1 truncate text-[10px] text-surface-200">
                    {loadedSnapshot.latestRisk?.risk_level ?? 'No risk data'}
                  </div>
                  <div className="mt-1 truncate text-[9px] text-surface-500">
                    {loadedSnapshot.latestRisk?.review_status ?? 'review unknown'}
                  </div>
                </div>
                <div className="rounded border border-surface-800 bg-surface-900/80 p-2">
                  <div className="text-[9px] text-surface-500">Executive posture</div>
                  <div className="mt-1 truncate text-[10px] text-surface-200">
                    {loadedSnapshot.executiveRepository?.posture ?? 'No posture'}
                  </div>
                  <div className="mt-1 truncate text-[9px] text-surface-500">
                    {loadedSnapshot.providerHealth}
                  </div>
                </div>
              </div>

              {!hasEvidence && (
                <div className="rounded border border-surface-800 bg-surface-900/80 px-2 py-1.5 text-[10px] text-surface-400">
                  Repo detected, but no governance evidence is currently available for this filter.
                </div>
              )}

              <div className="flex flex-wrap gap-1 border-t border-surface-800 pt-2">
                <a
                  className="rounded border border-surface-700 bg-surface-900 px-1.5 py-0.5 text-[8px] uppercase tracking-wider text-surface-300 hover:text-surface-100"
                  href="/governance/releases#deployment-gate-history"
                >
                  Open Gate
                </a>
                <a
                  className="rounded border border-surface-700 bg-surface-900 px-1.5 py-0.5 text-[8px] uppercase tracking-wider text-surface-300 hover:text-surface-100"
                  href="/governance/releases#change-risk"
                >
                  Open Risk
                </a>
                <a
                  className="inline-flex items-center gap-1 rounded border border-surface-700 bg-surface-900 px-1.5 py-0.5 text-[8px] uppercase tracking-wider text-surface-300 hover:text-surface-100"
                  href="/governance/releases#multi-repo-executive-governance"
                >
                  <BarChart3 size={9} />
                  Open Executive
                </a>
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  )
}
