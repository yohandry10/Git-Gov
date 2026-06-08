import { useEffect, useState, type ComponentType } from 'react'
import { useTranslation } from 'react-i18next'
import { useAuthStore } from '@/store/useAuthStore'
import { useRepoStore } from '@/store/useRepoStore'
import { useUpdateStore } from '@/store/useUpdateStore'
import { useControlPlaneStore } from '@/store/useControlPlaneStore'
import { Header } from '@/components/layout/Header'
import { Button } from '@/components/shared/Button'
import { Modal } from '@/components/shared/Modal'
import { LanguagePreferenceSelector } from '@/components/shared/LanguagePreferenceSelector'
import { User, FolderOpen, FileCode, LogOut, Shield, Users, Download, RefreshCw, Sparkles, ExternalLink, Globe, Bell, Server, Settings as SettingsIcon } from 'lucide-react'
import { TIMEZONES, detectBrowserTimezone, formatTs } from '@/lib/timezone'
import { AdminOnboardingPanel } from '@/components/control_plane/AdminOnboardingPanel'
import { TeamManagementPanel } from '@/components/control_plane/TeamManagementPanel'
import { ApiKeyManagerWidget } from '@/components/control_plane/ApiKeyManagerWidget'
import { GovernanceRulesPanel } from '@/components/control_plane/GovernanceRulesPanel'
import { ServerConfigPanel } from '@/components/control_plane/ServerConfigPanel'
import { loadNotificationPrefs, saveNotificationPrefs, type NotificationPrefs } from '@/lib/notifications'

type SettingsTab = 'preferences' | 'organization' | 'account' | 'repository' | 'connection'

const SETTINGS_TABS: Array<{
  id: SettingsTab
  labelKey: string
  descriptionKey: string
  icon: ComponentType<{ size?: number; className?: string }>
}> = [
  {
    id: 'preferences',
    labelKey: 'settings.tabs.preferences.label',
    descriptionKey: 'settings.tabs.preferences.description',
    icon: Globe,
  },
  {
    id: 'organization',
    labelKey: 'settings.tabs.organization.label',
    descriptionKey: 'settings.tabs.organization.description',
    icon: Users,
  },
  {
    id: 'account',
    labelKey: 'settings.tabs.account.label',
    descriptionKey: 'settings.tabs.account.description',
    icon: User,
  },
  {
    id: 'repository',
    labelKey: 'settings.tabs.repository.label',
    descriptionKey: 'settings.tabs.repository.description',
    icon: FolderOpen,
  },
  {
    id: 'connection',
    labelKey: 'settings.tabs.connection.label',
    descriptionKey: 'settings.tabs.connection.description',
    icon: Server,
  },
]

function readSettingsTabFromHash(): SettingsTab {
  if (typeof window === 'undefined') return 'preferences'
  const hash = window.location.hash.replace(/^#/, '')
  if (hash === 'control-plane') return 'connection'
  if (hash === 'updates') return 'connection'
  if (
    hash === 'preferences' ||
    hash === 'connection' ||
    hash === 'organization' ||
    hash === 'account' ||
    hash === 'repository'
  ) {
    return hash
  }
  return 'preferences'
}

export function SettingsPage() {
  const { t } = useTranslation()
  const { user, logout, isPinEnabled, setLocalPin, clearLocalPin, lockSession, pinError } = useAuthStore()
  const { repoPath, config, validation } = useRepoStore()
  const displayTimezone = useControlPlaneStore((s) => s.displayTimezone)
  const setDisplayTimezone = useControlPlaneStore((s) => s.setDisplayTimezone)
  const serverConfig = useControlPlaneStore((s) => s.serverConfig)
  const isConnected = useControlPlaneStore((s) => s.isConnected)
  const connectionStatus = useControlPlaneStore((s) => s.connectionStatus)
  const selectedOrgName = useControlPlaneStore((s) => s.selectedOrgName)
  const userRole = useControlPlaneStore((s) => s.userRole)
  const userClientId = useControlPlaneStore((s) => s.userClientId)
  const sseConnected = useControlPlaneStore((s) => s.sseConnected)
  const disconnect = useControlPlaneStore((s) => s.disconnect)
  const {
    status: updaterStatus,
    isChecking,
    isDownloading,
    isUpdaterSupported,
    isUpdaterConfigured,
    updateInfo,
    progress,
    lastCheckedAt,
    error: updaterError,
    isMandatoryUpdateRequired,
    mandatoryUpdateReason,
    minimumSupportedVersion,
    channel: updateChannel,
    fallbackDownloadUrl,
    changelogExpanded,
    telemetry: updaterTelemetry,
    checkForUpdates,
    downloadAndInstall,
    retryDownload,
    setChannel,
    setChangelogExpanded,
  } = useUpdateStore()
  const [showRepoSelector, setShowRepoSelector] = useState(false)
  const [pinInput, setPinInput] = useState('')
  const [notifPrefs, setNotifPrefs] = useState<NotificationPrefs>(loadNotificationPrefs)
  const [activeTab, setActiveTab] = useState<SettingsTab>(() => readSettingsTabFromHash())
  const isControlPlaneAdmin = userRole === 'Admin'
  const canManageOrgSettings = Boolean(user?.is_admin) || isControlPlaneAdmin
  const remoteUrl = validation?.remote_url ?? ''
  const repoFullName = remoteUrl.match(/[/:]([^/]+\/[^/.]+?)(?:\.git)?$/)?.[1] ?? ''
  const settingsContentClass =
    activeTab === 'preferences'
      ? 'grid grid-cols-1 gap-4 xl:grid-cols-3'
      : activeTab === 'repository' && config
        ? 'grid grid-cols-1 gap-4 xl:grid-cols-2'
        : 'space-y-4'
  const controlPlaneEndpoint = serverConfig?.url || t('common.notConfigured')
  const controlPlaneScope = selectedOrgName || userClientId || t('common.notSelected')
  const controlPlaneTransport = sseConnected
    ? t('settings.connection.liveStream')
    : connectionStatus === 'connected'
      ? t('settings.connection.httpConnected')
      : connectionStatus === 'checking'
        ? t('common.checking')
        : connectionStatus === 'maintenance'
          ? t('common.maintenance')
          : t('common.disconnected')
  const updaterStatusText =
    !isUpdaterSupported
      ? t('settings.updates.unsupported')
      : updaterStatus === 'not-configured'
        ? t('settings.updates.notConfigured')
        : updaterStatus === 'mandatory-update'
          ? t('settings.updates.mandatory')
          : updaterStatus === 'update-available'
            ? t('settings.updates.updateAvailable', { version: updateInfo?.version ?? 'unknown' })
            : updaterStatus === 'installed'
              ? t('settings.updates.installed')
              : updaterStatus === 'downloading'
                ? t('settings.updates.downloading')
                : updaterStatus === 'checking'
                  ? t('settings.updates.checking')
                  : updaterStatus === 'no-update'
                    ? t('settings.updates.noUpdate')
                    : t('settings.updates.idle')

  useEffect(() => {
    if (typeof window === 'undefined') return
    const onHashChange = () => setActiveTab(readSettingsTabFromHash())
    window.addEventListener('hashchange', onHashChange)
    onHashChange()
    return () => window.removeEventListener('hashchange', onHashChange)
  }, [])

  const selectTab = (tab: SettingsTab) => {
    setActiveTab(tab)
    if (typeof window === 'undefined') return
    const hash = tab === 'connection' ? 'control-plane' : tab
    window.history.replaceState(null, '', `/settings#${hash}`)
  }

  const updateNotifPrefs = (patch: Partial<NotificationPrefs>) => {
    const next = { ...notifPrefs, ...patch }
    setNotifPrefs(next)
    saveNotificationPrefs(next)
  }

  return (
    <div className="h-full flex flex-col bg-surface-950">
      <Header />

      <div className="flex-1 overflow-auto bg-surface-950">
        <div className="p-5 animate-fade-in">
          <div className="mb-4">
            <div className="flex items-center gap-2">
              <SettingsIcon size={16} className="text-brand-400" />
              <h1 className="text-sm font-semibold text-white">{t('settings.title')}</h1>
            </div>
            <p className="mt-1 max-w-3xl text-xs text-surface-500">
              {t('settings.body')}
            </p>
          </div>

          <nav className="mb-4 grid grid-cols-1 gap-2 md:grid-cols-3 xl:grid-cols-5">
            {SETTINGS_TABS.map((item) => (
              <button
                key={item.id}
                type="button"
                onClick={() => selectTab(item.id)}
                className={`rounded-lg border p-3 text-left transition-colors ${
                  activeTab === item.id
                    ? 'border-brand-500/45 bg-brand-500/10 text-surface-100'
                    : 'border-white/8 bg-white/[0.02] text-surface-300 hover:border-white/20 hover:bg-white/[0.04]'
                }`}
              >
                <span className="flex items-center gap-2 text-xs font-semibold">
                  <item.icon size={14} className="text-brand-300" />
                  {t(item.labelKey)}
                </span>
                <span className="mt-1 block text-[11px] leading-4 text-surface-500">
                  {t(item.descriptionKey)}
                </span>
              </button>
            ))}
          </nav>

          <div className={settingsContentClass}>
          <section className={`${activeTab === 'preferences' ? '' : 'hidden'} rounded-2xl border border-surface-700/30 bg-surface-800/40 p-6`}>
            <div className="card-header mb-2">
              <Globe size={12} strokeWidth={1.5} />
              {t('settings.languageSectionTitle')}
            </div>
            <p className="text-xs text-surface-400 mb-4">
              {t('settings.languageSectionBody')}
            </p>
            <LanguagePreferenceSelector />
          </section>

          <section className={`${activeTab === 'preferences' ? '' : 'hidden'} rounded-2xl border border-surface-700/30 bg-surface-800/40 p-6`}>
            <div className="card-header mb-2">
              <Globe size={12} strokeWidth={1.5} />
              {t('settings.timezone.title')}
            </div>
            <p className="text-xs text-surface-400 mb-4">
              {t('settings.timezone.body')}
            </p>
            <div className="space-y-3">
              <div className="rounded-lg border border-surface-700/30 bg-surface-900/50 p-3 space-y-2">
                <p className="text-[10px] text-surface-500 uppercase tracking-widest font-medium">{t('settings.timezone.field')}</p>
                <select
                  value={displayTimezone}
                  onChange={(e) => setDisplayTimezone(e.target.value)}
                  className="w-full bg-surface-900 border border-surface-700/50 rounded-lg px-3 py-2 text-xs text-surface-100 focus:outline-none focus:border-brand-500/60"
                >
                  {TIMEZONES.map((tz) => (
                    <option key={tz.value} value={tz.value}>{tz.label}</option>
                  ))}
                  {!TIMEZONES.some((tz) => tz.value === displayTimezone) && (
                    <option value={displayTimezone}>{displayTimezone}</option>
                  )}
                </select>
                <div className="flex items-center gap-2 flex-wrap">
                  <Button
                    variant="secondary"
                    size="sm"
                    onClick={() => setDisplayTimezone(detectBrowserTimezone())}
                  >
                    {t('settings.timezone.autoDetect')}
                  </Button>
                  <span className="text-[10px] text-surface-500">
                    {t('settings.timezone.active')} <span className="text-surface-300 font-medium font-mono">{displayTimezone}</span>
                  </span>
                </div>
              </div>
            </div>
          </section>

          <section id="control-plane" className={`${activeTab === 'connection' ? '' : 'hidden'} rounded-2xl border border-surface-700/30 bg-surface-800/40 p-6`}>
            <div className="card-header mb-2">
              <Server size={12} strokeWidth={1.5} />
              {t('settings.connection.title')}
            </div>
            <p className="text-xs text-surface-400 mb-4">
              {t('settings.connection.body')}
            </p>

            <div className="grid grid-cols-1 gap-2 md:grid-cols-2 xl:grid-cols-4 mb-4">
              <div className="rounded-lg border border-white/8 bg-surface-900/50 p-3">
                <p className="text-[10px] uppercase tracking-widest text-surface-500">{t('settings.connection.endpoint')}</p>
                <p className="mt-2 mono-data text-xs font-semibold text-surface-100 break-all">{controlPlaneEndpoint}</p>
              </div>
              <div className="rounded-lg border border-white/8 bg-surface-900/50 p-3">
                <p className="text-[10px] uppercase tracking-widest text-surface-500">{t('settings.connection.role')}</p>
                <p className="mt-2 text-xs font-semibold text-surface-100">{userRole || t('common.noRole')}</p>
              </div>
              <div className="rounded-lg border border-white/8 bg-surface-900/50 p-3">
                <p className="text-[10px] uppercase tracking-widest text-surface-500">{t('settings.connection.scope')}</p>
                <p className="mt-2 text-xs font-semibold text-surface-100 break-all">{controlPlaneScope}</p>
              </div>
              <div className="rounded-lg border border-white/8 bg-surface-900/50 p-3">
                <p className="text-[10px] uppercase tracking-widest text-surface-500">{t('settings.connection.transport')}</p>
                <p className="mt-2 text-xs font-semibold text-surface-100">{controlPlaneTransport}</p>
              </div>
            </div>

            <ServerConfigPanel />
          </section>

          <section className={`${activeTab === 'preferences' ? '' : 'hidden'} rounded-2xl border border-surface-700/30 bg-surface-800/40 p-6`}>
            <div className="card-header mb-2">
              <Bell size={12} strokeWidth={1.5} />
              {t('settings.notifications.title')}
            </div>
            <p className="text-xs text-surface-400 mb-4">
              {t('settings.notifications.body')}
            </p>
            <div className="space-y-2">
              <label className="flex items-center gap-3 rounded-lg border border-surface-700/30 bg-surface-900/50 px-3 py-2.5 cursor-pointer" aria-label={t('settings.notifications.enable')}>
                <input
                  type="checkbox"
                  checked={notifPrefs.enabled}
                  onChange={(e) => updateNotifPrefs({ enabled: e.target.checked })}
                  className="accent-brand-500"
                />
                <div>
                  <span className="text-xs text-surface-100 font-medium">{t('settings.notifications.enable')}</span>
                  <p className="text-[10px] text-surface-500">{t('settings.notifications.master')}</p>
                </div>
              </label>
              {notifPrefs.enabled && (
                <div className="ml-4 space-y-1.5">
                  <label className="flex items-center gap-3 rounded-lg border border-surface-700/20 bg-surface-900/30 px-3 py-2 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={notifPrefs.onNewEvents}
                      onChange={(e) => updateNotifPrefs({ onNewEvents: e.target.checked })}
                      className="accent-brand-500"
                    />
                    <span className="text-xs text-surface-200">{t('settings.notifications.newEvents')}</span>
                  </label>
                  <label className="flex items-center gap-3 rounded-lg border border-surface-700/20 bg-surface-900/30 px-3 py-2 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={notifPrefs.onBlockedPush}
                      onChange={(e) => updateNotifPrefs({ onBlockedPush: e.target.checked })}
                      className="accent-brand-500"
                    />
                    <span className="text-xs text-surface-200">{t('settings.notifications.blockedPush')}</span>
                  </label>
                  <label className="flex items-center gap-3 rounded-lg border border-surface-700/20 bg-surface-900/30 px-3 py-2 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={notifPrefs.onGovernanceWarn}
                      onChange={(e) => updateNotifPrefs({ onGovernanceWarn: e.target.checked })}
                      className="accent-brand-500"
                    />
                    <span className="text-xs text-surface-200">{t('settings.notifications.governanceWarn')}</span>
                  </label>
                </div>
              )}
            </div>
          </section>

          {activeTab === 'organization' && canManageOrgSettings && (
            <section className="rounded-2xl border border-surface-700/30 bg-surface-800/40 p-6">
              <div className="card-header mb-2">{t('settings.organization.title')}</div>
              <p className="text-xs text-surface-400 mb-4">
                {t('settings.organization.body')}
              </p>

              {!isConnected ? (
                <div className="rounded-lg border border-white/8 bg-surface-900/50 p-4 text-xs text-surface-300">
                  {t('settings.organization.connectFirst')}
                  <div className="mt-3">
                    <Button
                      size="sm"
                      variant="secondary"
                      onClick={() => selectTab('connection')}
                    >
                      {t('settings.organization.configureControlPlane')}
                    </Button>
                  </div>
                </div>
              ) : (
                <div className="space-y-4">
                  <AdminOnboardingPanel />
                  <TeamManagementPanel />
                  <ApiKeyManagerWidget />
                </div>
              )}
            </section>
          )}

          {activeTab === 'organization' && canManageOrgSettings && isConnected && repoFullName && (
            <section className="rounded-2xl border border-surface-700/30 bg-surface-800/40 p-6">
              <GovernanceRulesPanel repoFullName={repoFullName} />
            </section>
          )}

          {activeTab === 'organization' && !canManageOrgSettings && (
            <section className="rounded-2xl border border-surface-700/30 bg-surface-800/40 p-6">
              <div className="card-header mb-2">
                <Users size={12} strokeWidth={1.5} />
                {t('settings.organization.title')}
              </div>
              <p className="text-xs text-surface-400">
                {t('settings.organization.adminRequired')}
              </p>
            </section>
          )}

          <section className={`${activeTab === 'connection' ? '' : 'hidden'} rounded-2xl border border-surface-700/30 bg-surface-800/40 p-6`}>
            <div className="card-header mb-5">
              <Sparkles size={12} strokeWidth={1.5} />
              {t('settings.updates.title')}
            </div>

            <div className="space-y-3">
              <div className="rounded-lg border border-surface-700/30 bg-surface-900/50 p-3">
                <p className="text-[10px] text-surface-500 uppercase tracking-widest font-medium mb-2">
                  {t('settings.updates.channel')}
                </p>
                <div className="flex flex-wrap gap-2">
                  <Button
                    size="sm"
                    variant={updateChannel === 'stable' ? 'primary' : 'secondary'}
                    onClick={() => setChannel('stable')}
                    disabled={isChecking || isDownloading}
                    title={t('settings.updates.stableTitle')}
                  >
                    Stable
                  </Button>
                  <Button
                    size="sm"
                    variant={updateChannel === 'beta' ? 'primary' : 'secondary'}
                    onClick={() => setChannel('beta')}
                    disabled={isChecking || isDownloading}
                    title={t('settings.updates.betaTitle')}
                  >
                    Beta
                  </Button>
                </div>
                <p className="text-[10px] text-surface-500 mt-2">
                  {t('settings.updates.activeChannel')} <span className="text-surface-300 font-medium">{updateChannel}</span>
                </p>
              </div>

              <div className="rounded-lg border border-surface-700/30 bg-surface-900/50 p-3">
                <p className="text-[10px] text-surface-500 uppercase tracking-widest font-medium mb-1">
                  {t('settings.updates.status')}
                </p>
                <p className="text-xs text-surface-200">
                  {updaterStatusText}
                </p>
                {lastCheckedAt && (
                  <p className="text-[10px] text-surface-500 mt-1">
                    {t('settings.updates.lastChecked')} {formatTs(lastCheckedAt, displayTimezone)}
                  </p>
                )}
                <p className="text-[10px] text-surface-500 mt-1">
                  {t('settings.updates.telemetry', {
                    checks: updaterTelemetry.checks,
                    withUpdate: updaterTelemetry.updateChecksWithUpdate,
                    downloads: updaterTelemetry.downloadAttempts,
                    installed: updaterTelemetry.installSuccesses,
                    failed: updaterTelemetry.installFailures,
                  })}
                </p>
                {updaterTelemetry.lastEventAt && (
                  <p className="text-[10px] text-surface-500 mt-1">
                    {t('settings.updates.lastOutcome')} <span className="text-surface-300">{updaterTelemetry.lastOutcome}</span> · {formatTs(updaterTelemetry.lastEventAt, displayTimezone)}
                  </p>
                )}
                {updaterError && (
                  <p className="text-[10px] text-danger-400 mt-1 wrap-break-word">{updaterError}</p>
                )}
                {isMandatoryUpdateRequired && (
                  <p className="text-[10px] text-warning-300 mt-1 wrap-break-word">
                    {t('settings.updates.mandatoryActive')}
                    {minimumSupportedVersion ? ` ${t('settings.updates.minimumSupported', { version: minimumSupportedVersion })}` : ''}
                    {mandatoryUpdateReason ? ` ${mandatoryUpdateReason}` : ''}
                  </p>
                )}
                {!isUpdaterConfigured && isUpdaterSupported && (
                  <p className="text-[10px] text-warning-300 mt-1">
                    {t('settings.updates.configHint')}
                  </p>
                )}
              </div>

              {updateInfo && (
                <div className="rounded-lg border border-brand-500/20 bg-brand-500/5 p-3">
                  <div className="flex flex-wrap items-center justify-between gap-2">
                    <div>
                      <p className="text-sm font-semibold text-white tracking-tight">
                        v{updateInfo.version}
                      </p>
                      <p className="text-[10px] text-surface-500">
                        {t('settings.updates.current')} v{updateInfo.currentVersion}
                        {updateInfo.date ? ` · ${formatTs(Date.parse(updateInfo.date), displayTimezone)}` : ''}
                      </p>
                    </div>
                    <div className="flex flex-wrap gap-2">
                      <Button
                        variant="secondary"
                        size="sm"
                        onClick={() => setChangelogExpanded(!changelogExpanded)}
                      >
                        {changelogExpanded ? t('settings.updates.hideChangelog') : t('settings.updates.showChangelog')}
                      </Button>
                      <Button
                        size="sm"
                        onClick={() => void downloadAndInstall()}
                        loading={isDownloading}
                      >
                        <Download size={13} strokeWidth={1.5} />
                        {t('settings.updates.downloadInstall')}
                      </Button>
                      {updaterStatus === 'error' && (
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => void retryDownload()}
                          disabled={isDownloading}
                        >
                          <RefreshCw size={13} strokeWidth={1.5} />
                          {t('settings.updates.retryDownload')}
                        </Button>
                      )}
                    </div>
                  </div>

                  {isDownloading && (
                    <div className="mt-2">
                      <div className="h-1.5 rounded bg-surface-800 overflow-hidden">
                        <div
                          className="h-full bg-brand-500 transition-all duration-200"
                          style={{
                            width: progress?.totalBytes && progress.totalBytes > 0
                              ? `${Math.min(100, (progress.downloadedBytes / progress.totalBytes) * 100)}%`
                              : '20%',
                          }}
                        />
                      </div>
                      <p className="text-[10px] text-surface-500 mt-1">
                        {progress?.downloadedBytes
                          ? t('settings.updates.downloadedKb', { kb: Math.round(progress.downloadedBytes / 1024) })
                          : t('settings.updates.preparingDownload')}
                      </p>
                    </div>
                  )}

                  {changelogExpanded && (
                    <div className="mt-2 rounded border border-white/6 bg-surface-950/50 p-2">
                      <p className="text-[10px] text-surface-500 mb-1">{t('settings.updates.changelog')}</p>
                      <pre className="text-[11px] whitespace-pre-wrap text-surface-300 leading-relaxed">
                        {updateInfo.body?.trim() || t('settings.updates.noChangelog')}
                      </pre>
                    </div>
                  )}
                </div>
              )}

              <div className="flex flex-wrap gap-2">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => void checkForUpdates({ manual: true, force: true })}
                  loading={isChecking}
                >
                  <RefreshCw size={13} strokeWidth={1.5} />
                  {t('settings.updates.check')}
                </Button>
                <Button
                  variant="secondary"
                  size="sm"
                  onClick={() => window.open(fallbackDownloadUrl, '_blank', 'noopener,noreferrer')}
                  title={t('settings.updates.fallbackTitle')}
                >
                  <ExternalLink size={13} strokeWidth={1.5} />
                  {t('settings.updates.manualDownload')}
                </Button>
              </div>
            </div>
          </section>

          <section className={`${activeTab === 'account' ? '' : 'hidden'} rounded-2xl border border-surface-700/30 bg-surface-800/40 p-6`}>
            <div className="card-header mb-5">
              <User size={12} strokeWidth={1.5} />
              {t('settings.account.title')}
            </div>
            {user && (
              <div className="space-y-4">
                <div className="flex items-center gap-4">
                  <img
                    src={user.avatar_url}
                    alt={user.login}
                    className="w-12 h-12 rounded-full ring-2 ring-surface-700 ring-offset-2 ring-offset-surface-800"
                  />
                  <div>
                    <p className="text-white font-semibold tracking-tight">{user.name}</p>
                    <p className="text-surface-500 text-xs">@{user.login}</p>
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  {user.is_admin && (
                    <span className="text-[10px] font-medium bg-brand-500/10 text-brand-400 px-2 py-0.5 rounded inline-flex items-center gap-1">
                      <Shield size={9} />
                      {t('settings.account.admin')}
                    </span>
                  )}
                  {user.group && (
                    <span className="text-[10px] font-medium bg-surface-700/40 text-surface-400 px-2 py-0.5 rounded inline-flex items-center gap-1">
                      <Users size={9} />
                      {user.group}
                    </span>
                  )}
                </div>

                <div className="rounded-lg border border-surface-700/30 bg-surface-900/50 p-3">
                  <p className="text-[10px] text-surface-500 uppercase tracking-widest mb-1 font-medium">{t('settings.account.controlPlane')}</p>
                  <p className="text-xs text-surface-300">
                    {t('settings.account.roleSeparation')}
                  </p>
                  <p className="text-xs text-surface-200 mt-1">
                    {t('settings.account.currentRole')} <span className="font-medium">{userRole || t('common.noRole')}</span>
                    {userClientId ? <span className="text-surface-500"> · {userClientId}</span> : null}
                  </p>
                </div>

                <Button
                  variant="danger"
                  size="sm"
                  onClick={async () => {
                    disconnect()
                    await logout()
                  }}
                >
                  <LogOut size={13} strokeWidth={1.5} />
                  {t('settings.account.signOut')}
                </Button>
                <Button
                  variant="secondary"
                  size="sm"
                  onClick={async () => {
                    disconnect()
                    await logout()
                  }}
                >
                  {t('settings.account.switchUser')}
                </Button>

                <div className="mt-3 pt-3 border-t border-surface-700/30 space-y-2">
                  <p className="text-[10px] text-surface-500 uppercase tracking-widest font-medium">{t('settings.account.pinTitle')}</p>
                  <p className="text-[11px] text-surface-500">
                    {t('settings.account.pinBody')}
                  </p>
                  <div className="flex items-center gap-2">
                    <input
                      type="password"
                      inputMode="numeric"
                      pattern="[0-9]*"
                      value={pinInput}
                      onChange={(e) => setPinInput(e.target.value)}
                      placeholder={isPinEnabled ? t('settings.account.newPin') : t('settings.account.pinPlaceholder')}
                      className="bg-surface-900/60 rounded-lg border border-surface-700/30 px-3 py-1.5 text-xs text-white outline-none focus:border-brand-500/60"
                    />
                    <Button
                      size="sm"
                      onClick={async () => {
                        await setLocalPin(pinInput)
                        setPinInput('')
                      }}
                    >
                      {isPinEnabled ? t('settings.account.updatePin') : t('settings.account.enablePin')}
                    </Button>
                    {isPinEnabled && (
                      <Button variant="outline" size="sm" onClick={() => { void clearLocalPin() }}>
                        {t('settings.account.disablePin')}
                      </Button>
                    )}
                    {isPinEnabled && (
                      <Button variant="secondary" size="sm" onClick={lockSession}>
                        {t('settings.account.lockNow')}
                      </Button>
                    )}
                  </div>
                  {pinError && <p className="text-[11px] text-danger-400">{pinError}</p>}
                </div>
              </div>
            )}
          </section>

          <section className={`${activeTab === 'repository' ? '' : 'hidden'} rounded-2xl border border-surface-700/30 bg-surface-800/40 p-6`}>
            <div className="card-header mb-5">
              <FolderOpen size={12} strokeWidth={1.5} />
              {t('settings.repository.title')}
            </div>
            <div className="space-y-3">
              <div>
                <p className="text-[10px] text-surface-500 uppercase tracking-widest mb-1.5 font-medium">{t('settings.repository.currentPath')}</p>
                <p className="text-white mono-data text-xs bg-surface-900/60 p-3 rounded-lg border border-surface-700/30">
                  {repoPath || t('settings.repository.noneSelected')}
                </p>
              </div>
              <Button variant="secondary" onClick={() => setShowRepoSelector(true)}>
                <FolderOpen size={13} strokeWidth={1.5} />
                {t('settings.repository.change')}
              </Button>
            </div>
          </section>

          {activeTab === 'repository' && config && (
            <section className="rounded-2xl border border-surface-700/30 bg-surface-800/40 p-6">
              <div className="card-header mb-5">
                <FileCode size={12} strokeWidth={1.5} />
                {t('settings.repository.configTitle')}
              </div>
              <div className="bg-surface-900/60 rounded-lg p-4 border border-surface-700/30">
                <pre className="text-[11px] mono-data overflow-auto whitespace-pre-wrap leading-relaxed">
                  {JSON.stringify(config, null, 2).split('\n').map((line, i) => {
                    const keyMatch = line.match(/^(\s*)"([^"]+)"(:)/)
                    if (keyMatch) {
                      return (
                        <span key={i}>
                          {keyMatch[1]}<span className="text-brand-400">"{keyMatch[2]}"</span>{keyMatch[3]}
                          <span className="text-surface-400">{line.slice(keyMatch[0].length)}</span>{'\n'}
                        </span>
                      )
                    }
                    return <span key={i} className="text-surface-400">{line}{'\n'}</span>
                  })}
                </pre>
              </div>
            </section>
          )}
        </div>
      </div>
      </div>

      <Modal
        isOpen={showRepoSelector}
        onClose={() => setShowRepoSelector(false)}
        title={t('settings.repository.modalTitle')}
        size="lg"
      >
        <div className="text-center py-4">
          <p className="text-surface-400 text-sm mb-4">
            {t('settings.repository.modalBody')}
          </p>
          <Button onClick={() => setShowRepoSelector(false)}>
            {t('settings.repository.modalAction')}
          </Button>
        </div>
      </Modal>
    </div>
  )
}
