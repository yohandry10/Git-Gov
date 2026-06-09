import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useControlPlaneStore } from '@/store/useControlPlaneStore'
import { Button } from '@/components/shared/Button'
import { Server, Link, Unlink, RefreshCw, Wrench } from 'lucide-react'
import {
  DEFAULT_CONTROL_PLANE_URL,
  resolveControlPlaneUrl,
  validateControlPlaneUrl,
} from '@/lib/controlPlaneConfig'

export function ServerConfigPanel() {
  const { t } = useTranslation()
  const serverConfig = useControlPlaneStore((s) => s.serverConfig)
  const isConnected = useControlPlaneStore((s) => s.isConnected)
  const connectionStatus = useControlPlaneStore((s) => s.connectionStatus)
  const isLoading = useControlPlaneStore((s) => s.isLoading)
  const error = useControlPlaneStore((s) => s.error)
  const userRole = useControlPlaneStore((s) => s.userRole)
  const userClientId = useControlPlaneStore((s) => s.userClientId)
  const setServerConfig = useControlPlaneStore((s) => s.setServerConfig)
  const checkConnection = useControlPlaneStore((s) => s.checkConnection)
  const disconnect = useControlPlaneStore((s) => s.disconnect)
  const [url, setUrl] = useState(resolveControlPlaneUrl({ previousUrl: serverConfig?.url }))
  const [apiKey, setApiKey] = useState(serverConfig?.api_key || '')
  const [localError, setLocalError] = useState<string | null>(null)

  const handleConnect = () => {
    const resolvedUrl = resolveControlPlaneUrl({ inputUrl: url, previousUrl: serverConfig?.url })
    const validationError = validateControlPlaneUrl(resolvedUrl)
    if (validationError) {
      setLocalError(validationError)
      return
    }
    setLocalError(null)
    setServerConfig({
      url: resolvedUrl,
      api_key: apiKey || undefined,
    })
  }

  if (connectionStatus === 'maintenance' && serverConfig) {
    return (
      <div className="card">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-2">
            <Wrench size={20} className="text-warning-400" />
            <span className="text-white font-medium">{t('serverConfig.maintenanceTitle')}</span>
          </div>
          <div className="flex gap-2">
            <Button variant="ghost" size="sm" onClick={() => void checkConnection()}>
              <RefreshCw size={14} className={isLoading ? 'animate-spin' : ''} />
            </Button>
          </div>
        </div>

        <div className="bg-surface-900 rounded-lg p-3">
          <p className="text-xs text-surface-400 mb-1">{t('serverConfig.serverUrl')}</p>
          <p className="text-sm text-white font-mono">{serverConfig.url}</p>
        </div>

        <div className="bg-surface-900 rounded-lg p-3 mt-3">
          <p className="text-xs text-surface-400 mb-1">{t('serverConfig.identity')}</p>
          <p className="text-sm text-white">
            {userRole || t('common.noRole')}{userClientId ? ` · ${userClientId}` : ''}
          </p>
        </div>

        {userRole !== 'Admin' && (
          <div className="mt-3 p-2 bg-warning-500/20 border border-warning-500/50 rounded text-warning-300 text-sm">
            {t('serverConfig.adminRequired', { role: userRole || t('common.noRole') })}
          </div>
        )}

        <div className="mt-3 p-2 bg-warning-500/20 border border-warning-500/50 rounded text-warning-300 text-sm flex items-center gap-2">
          <Wrench size={14} className="shrink-0" />
          {t('serverConfig.maintenanceBody')}
        </div>
      </div>
    )
  }

  if (isConnected && serverConfig) {
    return (
      <div className="card">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-2">
            <Server size={20} className="text-success-500" />
            <span className="text-white font-medium">{t('serverConfig.connectedTitle')}</span>
          </div>
          <div className="flex gap-2">
            <Button variant="ghost" size="sm" onClick={() => void checkConnection()}>
              <RefreshCw size={14} className={isLoading ? 'animate-spin' : ''} />
            </Button>
            <Button variant="danger" size="sm" onClick={disconnect}>
              <Unlink size={14} className="mr-1" />
              {t('serverConfig.disconnect')}
            </Button>
          </div>
        </div>

        <div className="bg-surface-900 rounded-lg p-3">
          <p className="text-xs text-surface-400 mb-1">{t('serverConfig.serverUrl')}</p>
          <p className="text-sm text-white font-mono">{serverConfig.url}</p>
        </div>

        {(error || localError) && (
          <div className="mt-3 p-2 bg-danger-500/20 border border-danger-500/50 rounded text-danger-400 text-sm">
            {error || localError}
          </div>
        )}
      </div>
    )
  }

  return (
    <div className="card">
      <div className="flex items-center gap-2 mb-4">
        <Link size={20} className="text-brand-500" />
        <span className="text-white font-medium">{t('serverConfig.connectTitle')}</span>
      </div>

      <div className="space-y-3">
        <div>
          <label htmlFor="server-url-input" className="block text-sm text-surface-400 mb-1">{t('serverConfig.serverUrl')}</label>
          <input
            id="server-url-input"
            type="text"
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            placeholder={DEFAULT_CONTROL_PLANE_URL}
            className="input"
          />
          <p className="mt-1 text-xs text-surface-500">
            {t('serverConfig.urlHint')}
          </p>
        </div>

        <div>
          <label htmlFor="server-api-key-input" className="block text-sm text-surface-400 mb-1">{t('serverConfig.apiKey')}</label>
          <input
            id="server-api-key-input"
            type="password"
            value={apiKey}
            onChange={(e) => setApiKey(e.target.value)}
            placeholder={t('serverConfig.apiKeyPlaceholder')}
            className="input"
          />
        </div>

        {(error || localError) && (
          <div className="p-2 bg-danger-500/20 border border-danger-500/50 rounded text-danger-400 text-sm">
            {error || localError}
          </div>
        )}

        <Button onClick={handleConnect} loading={isLoading} className="w-full">
          <Link size={16} className="mr-2" />
          {t('serverConfig.connect')}
        </Button>
        {error && (
          <div className="p-2 bg-danger-500/20 border border-danger-500/50 rounded text-danger-400 text-sm">
            {error}
          </div>
        )}
      </div>
    </div>
  )
}
