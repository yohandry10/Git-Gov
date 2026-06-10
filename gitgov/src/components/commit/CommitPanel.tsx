import { useState, useMemo, useEffect, useCallback } from 'react'
import { useRepoStore } from '@/store/useRepoStore'
import { useAuthStore } from '@/store/useAuthStore'
import { useControlPlaneStore } from '@/store/useControlPlaneStore'
import { Button } from '@/components/shared/Button'
import { COMMIT_TYPES } from '@/lib/constants'
import { AlertTriangle, ArrowDown, ArrowUp, GitCommit, Upload, RotateCcw, TerminalSquare } from 'lucide-react'
import { toast } from '@/components/shared/Toast'
import { tauriInvoke, parseCommandError } from '@/lib/tauri'
import { emitCliLine } from '@/lib/cliEvents'
import {
  buildGitIdentityEvidenceLines,
  buildGitHubCommitAuthorName,
  buildGitHubNoReplyEmail,
  evaluateGitIdentity,
  formatGitIdentityBlockToast,
  formatGitIdentityScope,
  type GitIdentity,
} from '@/lib/gitIdentityPolicy'
import clsx from 'clsx'

interface CliCommandAuditPayload {
  org_name?: string | null
  command: string
  origin: 'button_click' | 'manual_input'
  branch: string
  repo_name?: string
  exit_code?: number
  duration_ms?: number
  metadata?: Record<string, unknown>
}

function inferRepoNameFromPath(path?: string | null): string | undefined {
  if (!path) return undefined
  const parts = path.split(/[\\/]/).filter(Boolean)
  return parts.length > 0 ? parts[parts.length - 1] : undefined
}

function formatPushErrorForUser(rawError: unknown): string {
  const parsed = parseCommandError(String(rawError))
  const msg = parsed.message || String(rawError)

  if (msg.includes('without `workflow` scope') || msg.includes('without workflow scope')) {
    return 'Push rechazado por GitHub: estás modificando .github/workflows/* y tu token no tiene permiso "workflow". Reautentícate en GitHub para conceder ese permiso y vuelve a intentar.'
  }

  if (msg.includes('Invalid username or token') || msg.includes('Authentication failed')) {
    return 'Push rechazado por GitHub: token inválido o expirado. Reautentícate en GitHub y vuelve a intentar.'
  }

  if (
    parsed.code === 'AUTH_ERROR' ||
    msg.includes('No hay token guardado') ||
    msg.includes('Token not found in keyring')
  ) {
    return 'Push no enviado por autenticación local (token no disponible). Tus cambios y commits locales NO se perdieron; siguen en tu repositorio. Reautentica GitHub y vuelve a intentar push.'
  }

  return msg
}

export function CommitPanel() {
  const {
    repoPath,
    stagedFiles,
    fileChanges,
    currentBranch,
    branchSync,
    commit,
    push,
    unstageAll,
    refreshStatus,
    refreshBranchSync,
  } = useRepoStore()
  const { user } = useAuthStore()
  const controlPlaneConfig = useControlPlaneStore((s) => s.serverConfig)
  const selectedOrgName = useControlPlaneStore((s) => s.selectedOrgName)
  const [message, setMessage] = useState('')
  const [commitType, setCommitType] = useState('feat')
  const [isCommitting, setIsCommitting] = useState(false)
  const [isPushing, setIsPushing] = useState(false)
  const [isRefreshingGitIdentity, setIsRefreshingGitIdentity] = useState(false)
  const [lastCommitHash, setLastCommitHash] = useState<string | null>(null)
  const [gitIdentity, setGitIdentity] = useState<GitIdentity | null>(null)

  const refreshGitIdentity = useCallback(async (notify = false) => {
    if (!repoPath) {
      setGitIdentity(null)
      return null
    }

    setIsRefreshingGitIdentity(true)
    try {
      const nextIdentity = await tauriInvoke<GitIdentity>('cmd_get_git_identity', { repoPath })
      setGitIdentity(nextIdentity)
      if (notify) {
        toast('success', 'Identidad Git revalidada.')
      }
      return nextIdentity
    } catch {
      setGitIdentity(null)
      if (notify) {
        toast('error', 'No se pudo leer la identidad Git efectiva para este repo.')
      }
      return null
    } finally {
      setIsRefreshingGitIdentity(false)
    }
  }, [repoPath])

  useEffect(() => {
    void refreshGitIdentity()
  }, [refreshGitIdentity])

  const identityFinding = evaluateGitIdentity(gitIdentity, user)

  const fullMessage = useMemo(() => {
    if (!message.trim()) return ''
    if (message.includes(':')) return message
    return `${commitType}: ${message}`
  }, [commitType, message])

  const isValidMessage = useMemo(() => {
    if (!fullMessage) return false
    return /^(feat|fix|docs|style|refactor|test|chore|hotfix):/.test(fullMessage)
  }, [fullMessage])

  const ahead = branchSync?.ahead ?? 0
  const behind = branchSync?.behind ?? 0
  const pendingLocalCommits = branchSync?.pending_local_commits ?? ahead
  const hasUpstream = branchSync?.has_upstream ?? false
  const hasLocalCommits = pendingLocalCommits > 0

  const hasStagedFiles = stagedFiles.size > 0
  const hasUncommittedChanges = fileChanges.some((f) => f.staged) || stagedFiles.size > 0
  const canPush = Boolean(currentBranch) && (hasLocalCommits || lastCommitHash !== null || hasUncommittedChanges)
  const isIdentityBlocked = Boolean(identityFinding)
  const hasIdentityOrigin = Boolean(identityFinding?.nameScope || identityFinding?.emailScope)

  const handleShowIdentityProof = async () => {
    if (!user) return
    const currentIdentity = await refreshGitIdentity()
    if (!currentIdentity) return

    const currentFinding = evaluateGitIdentity(currentIdentity, user)
    buildGitIdentityEvidenceLines(currentIdentity, user, currentFinding).forEach((line) => {
      emitCliLine(line)
    })
    toast('info', 'Prueba de identidad Git escrita en el panel CLI.')
  }

  const handleCommit = async () => {
    if (!user || !isValidMessage) return
    if (identityFinding) {
      toast('error', formatGitIdentityBlockToast('Commit', identityFinding, user.login))
      return
    }
    emitCliLine({
      lineType: 'command',
      text: `$ git commit -m "${fullMessage.replaceAll('"', '\\"')}"`,
    })
    const commitAuditStart = Date.now()
    setIsCommitting(true)
    try {
      const hash = await commit(
        fullMessage,
        buildGitHubCommitAuthorName(user),
        buildGitHubNoReplyEmail(user),
        user.login
      )
      setLastCommitHash(hash)
      setMessage('')
      toast('success', `Commit creado: ${hash.substring(0, 7)}`)
      emitCliLine({
        lineType: 'gitgov',
        text: `✓ Commit auditado en GitGov (${hash.substring(0, 7)})`,
      })
      if (controlPlaneConfig?.url && controlPlaneConfig.api_key) {
        const payload: CliCommandAuditPayload = {
          command: `git commit -m "${fullMessage}"`,
          org_name: selectedOrgName.trim() || null,
          origin: 'button_click',
          branch: currentBranch ?? 'unknown',
          repo_name: inferRepoNameFromPath(repoPath),
          exit_code: 0,
          duration_ms: Date.now() - commitAuditStart,
          metadata: { source: 'commit_panel' },
        }
        void tauriInvoke('cmd_server_ingest_cli_command', {
          config: controlPlaneConfig,
          payload,
        }).catch(() => {})
      }
      const sync = await refreshBranchSync(currentBranch ?? undefined)
      const pendingAfterCommit = sync?.pending_local_commits ?? sync?.ahead ?? 0
      if (pendingAfterCommit > 0) {
        toast(
          'warning',
          `Tienes ${pendingAfterCommit} commit(s) local(es) sin push en ${sync?.branch ?? currentBranch ?? 'la rama actual'}.`
        )
      }
    } catch (e) {
      const parsed = parseCommandError(String(e))
      toast('error', parsed.message)
      emitCliLine({
        lineType: 'stderr',
        text: `✗ ${parsed.message}`,
      })
      if (controlPlaneConfig?.url && controlPlaneConfig.api_key) {
        const payload: CliCommandAuditPayload = {
          command: `git commit -m "${fullMessage}"`,
          org_name: selectedOrgName.trim() || null,
          origin: 'button_click',
          branch: currentBranch ?? 'unknown',
          repo_name: inferRepoNameFromPath(repoPath),
          exit_code: 1,
          duration_ms: Date.now() - commitAuditStart,
          metadata: { source: 'commit_panel', error: parsed.message },
        }
        void tauriInvoke('cmd_server_ingest_cli_command', {
          config: controlPlaneConfig,
          payload,
        }).catch(() => {})
      }
    } finally {
      setIsCommitting(false)
    }
  }

  const handlePush = async () => {
    if (!user || !currentBranch) return
    if (identityFinding) {
      toast('error', formatGitIdentityBlockToast('Push', identityFinding, user.login))
      return
    }
    emitCliLine({
      lineType: 'command',
      text: `$ git push origin ${currentBranch}`,
    })
    const pushAuditStart = Date.now()
    setIsPushing(true)
    try {
      await push(currentBranch, user.login)
      const syncAfterPush = await refreshBranchSync(currentBranch)
      const pendingAfterPush = syncAfterPush?.pending_local_commits ?? syncAfterPush?.ahead ?? 0
      if (pendingAfterPush > 0) {
        toast(
          'warning',
          `Push ejecutado pero aún quedan ${pendingAfterPush} commit(s) sin sincronizar en ${syncAfterPush?.branch ?? currentBranch}.`
        )
        emitCliLine({
          lineType: 'system',
          text: `! Push parcial: quedan ${pendingAfterPush} commit(s) pendientes`,
        })
      } else {
        toast('success', `Push exitoso a ${currentBranch}`)
        emitCliLine({
          lineType: 'gitgov',
          text: `✓ Push auditado en GitGov (${currentBranch})`,
        })
      }
      if (controlPlaneConfig?.url && controlPlaneConfig.api_key) {
        const payload: CliCommandAuditPayload = {
          command: `git push origin ${currentBranch}`,
          org_name: selectedOrgName.trim() || null,
          origin: 'button_click',
          branch: currentBranch,
          repo_name: inferRepoNameFromPath(repoPath),
          exit_code: 0,
          duration_ms: Date.now() - pushAuditStart,
          metadata: { source: 'commit_panel' },
        }
        void tauriInvoke('cmd_server_ingest_cli_command', {
          config: controlPlaneConfig,
          payload,
        }).catch(() => {})
      }
      setLastCommitHash(null)
      await refreshStatus()
    } catch (e) {
      const userMessage = formatPushErrorForUser(e)
      toast('error', userMessage)
      emitCliLine({
        lineType: 'stderr',
        text: `✗ ${userMessage}`,
      })
      if (controlPlaneConfig?.url && controlPlaneConfig.api_key) {
        const payload: CliCommandAuditPayload = {
          command: `git push origin ${currentBranch}`,
          org_name: selectedOrgName.trim() || null,
          origin: 'button_click',
          branch: currentBranch,
          repo_name: inferRepoNameFromPath(repoPath),
          exit_code: 1,
          duration_ms: Date.now() - pushAuditStart,
          metadata: { source: 'commit_panel', error: userMessage },
        }
        void tauriInvoke('cmd_server_ingest_cli_command', {
          config: controlPlaneConfig,
          payload,
        }).catch(() => {})
      }
      const syncAfterError = await refreshBranchSync(currentBranch)
      const pendingAfterError = syncAfterError?.pending_local_commits ?? syncAfterError?.ahead ?? 0
      if (pendingAfterError > 0) {
        toast(
          'warning',
          `Alerta: tienes ${pendingAfterError} commit(s) local(es) sin push en ${syncAfterError?.branch ?? currentBranch}.`
        )
      } else {
        toast('info', 'Tus cambios locales no se perdieron. Verifica el estado local y reintenta push.')
      }
      await refreshStatus()
    } finally {
      setIsPushing(false)
    }
  }

  const handleUnstageAll = async () => {
    emitCliLine({
      lineType: 'command',
      text: '$ git restore --staged .',
    })
    const unstageAuditStart = Date.now()
    await unstageAll()
    toast('info', 'Staging area limpiado')
    emitCliLine({
      lineType: 'gitgov',
      text: '✓ Staging limpiado',
    })
    if (controlPlaneConfig?.url && controlPlaneConfig.api_key) {
      const payload: CliCommandAuditPayload = {
        command: 'git restore --staged .',
        org_name: selectedOrgName.trim() || null,
        origin: 'button_click',
        branch: currentBranch ?? 'unknown',
        repo_name: inferRepoNameFromPath(repoPath),
        exit_code: 0,
        duration_ms: Date.now() - unstageAuditStart,
        metadata: { source: 'commit_panel' },
      }
      void tauriInvoke('cmd_server_ingest_cli_command', {
        config: controlPlaneConfig,
        payload,
      }).catch(() => {})
    }
  }

  return (
    <div className="shrink-0 min-w-0 border-t border-surface-700/30 bg-surface-900/50 px-5 py-4">
      {identityFinding && (
        <div className="mb-3 flex items-start gap-2 rounded-lg border border-warning-500/30 bg-warning-500/10 px-3 py-2">
          <AlertTriangle size={13} strokeWidth={1.75} className="mt-0.5 shrink-0 text-warning-400" />
          <div className="min-w-0 flex-1">
            <div className="flex flex-wrap items-center justify-between gap-2">
              <p className="text-[11px] font-medium text-warning-300">
                {identityFinding.reason === 'incomplete'
                  ? 'Identidad Git efectiva incompleta'
                  : 'Identidad Git no alineada de forma verificable'}
              </p>
              <Button
                size="sm"
                variant="ghost"
                onClick={() => void handleShowIdentityProof()}
                loading={isRefreshingGitIdentity}
                className="h-6 px-2 text-[10px] text-warning-200 hover:bg-warning-500/10 hover:text-warning-100"
                title="Revalidar y mostrar prueba de git config en el panel CLI"
              >
                <TerminalSquare size={12} strokeWidth={1.75} />
                Ver prueba
              </Button>
              <Button
                size="sm"
                variant="ghost"
                onClick={() => void refreshGitIdentity(true)}
                loading={isRefreshingGitIdentity}
                className="h-6 px-2 text-[10px] text-warning-200 hover:bg-warning-500/10 hover:text-warning-100"
                title="Revalidar identidad Git efectiva"
              >
                <RotateCcw size={12} strokeWidth={1.75} />
                Revalidar
              </Button>
            </div>
            <p className="mt-0.5 text-[10px] text-surface-400">
              {identityFinding.reason === 'incomplete'
                ? 'Git CLI no tiene autor efectivo completo para commits manuales en este repo. '
                : `Git CLI resolverá "${identityFinding.effectiveName} <${identityFinding.effectiveEmail}>" para commits manuales, mientras GitGov Desktop está autenticado como @${user?.login}. `}
              Ejecuta{' '}
              <code className="rounded bg-surface-800 px-1 text-warning-300">
                git config --local user.name "{identityFinding.suggestedName}"
              </code>{' '}
              y{' '}
              <code className="rounded bg-surface-800 px-1 text-warning-300">
                git config --local user.email "{identityFinding.suggestedEmail}"
              </code>
            </p>
            {hasIdentityOrigin && (
              <p className="mt-1 text-[10px] text-surface-500">
                Origen observado: user.name {formatGitIdentityScope(identityFinding.nameScope)}; user.email {formatGitIdentityScope(identityFinding.emailScope)}.
              </p>
            )}
            <p className="mt-1 text-[10px] text-warning-200">
              Bloqueo de política: Commit y Push quedan deshabilitados hasta que la identidad Git efectiva sea completa y verificable frente al usuario GitHub autenticado.
            </p>
          </div>
        </div>
      )}
      <div
        className="grid items-start gap-4"
        style={{ gridTemplateColumns: 'minmax(0, 1fr) 184px' }}
      >
        <div className="min-w-0 space-y-2">
          <div
            className="grid items-center gap-2"
            style={{ gridTemplateColumns: 'auto minmax(0, 1fr)' }}
          >
            <select
              value={commitType}
              onChange={(e) => setCommitType(e.target.value)}
              className="px-2.5 py-2 bg-surface-800 border border-surface-700/50 rounded-lg text-white text-xs focus:outline-none focus:border-brand-500/50 transition-colors"
            >
              {COMMIT_TYPES.map((type) => (
                <option key={type.value} value={type.value}>
                  {type.label}
                </option>
              ))}
            </select>
            <input
              type="text"
              value={message}
              onChange={(e) => setMessage(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && hasStagedFiles && isValidMessage && !isIdentityBlocked) {
                  handleCommit()
                }
              }}
              placeholder="descripción del cambio"
              className="w-full min-w-0 px-3 py-2 bg-surface-800 border border-surface-700/50 rounded-lg text-white text-xs placeholder-surface-600 focus:outline-none focus:border-brand-500/50 transition-colors"
            />
          </div>

          {branchSync && currentBranch && (
            <div className="flex flex-wrap items-center gap-1.5 text-[11px]">
              {!hasUpstream && (
                <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded border border-warning-500/30 bg-warning-500/10 text-warning-300">
                  <AlertTriangle size={11} strokeWidth={1.75} />
                  La rama no tiene upstream remoto configurado
                </span>
              )}

              {pendingLocalCommits > 0 && (
                <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded border border-danger-500/30 bg-danger-500/10 text-danger-300">
                  <ArrowUp size={11} strokeWidth={1.75} />
                  {pendingLocalCommits} commit(s) local(es) sin push
                </span>
              )}

              {hasUpstream && behind > 0 && (
                <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded border border-warning-500/30 bg-warning-500/10 text-warning-300">
                  <ArrowDown size={11} strokeWidth={1.75} />
                  {behind} commit(s) pendientes de pull
                </span>
              )}
            </div>
          )}

          <div className="flex items-center gap-2 text-[11px] px-0.5">
            <span className="text-surface-600">Preview:</span>
            <code
              className={clsx(
                'px-1.5 py-0.5 rounded mono-data text-[11px] transition-colors',
                isValidMessage ? 'bg-success-500/10 text-success-400' : 'bg-surface-800/50 text-surface-600',
              )}
            >
              {fullMessage || 'mensaje vacío'}
            </code>
          </div>
        </div>

        <div className="shrink-0 flex flex-col gap-2 justify-end" style={{ width: 184 }}>
          <div className="flex gap-2">
            <Button
              size="sm"
              variant="ghost"
              onClick={handleUnstageAll}
              disabled={!hasStagedFiles}
              title="Limpiar staging"
            >
              <RotateCcw size={13} strokeWidth={1.5} />
            </Button>

            <Button
              size="sm"
              onClick={handleCommit}
              loading={isCommitting}
              disabled={!hasStagedFiles || !isValidMessage || isIdentityBlocked}
              className="flex-1"
            >
              <GitCommit size={13} strokeWidth={1.5} />
              Commit ({stagedFiles.size})
            </Button>
          </div>

          <Button
            size="sm"
            variant="outline"
            onClick={handlePush}
            loading={isPushing}
            disabled={!canPush || isIdentityBlocked}
            className="w-full"
          >
            <Upload size={13} strokeWidth={1.5} />
            Push
          </Button>
        </div>
      </div>
    </div>
  )
}
