import { useEffect, useState } from 'react'
import { useAuthStore } from '@/store/useAuthStore'
import { useControlPlaneStore } from '@/store/useControlPlaneStore'
import { Button } from '@/components/shared/Button'
import { KeyRound, ShieldCheck } from 'lucide-react'
import {
  DEFAULT_CONTROL_PLANE_URL,
  resolveControlPlaneUrl,
} from '@/lib/controlPlaneConfig'

export function ControlPlaneAuthScreen() {
  const user = useAuthStore((s) => s.user)
  const logout = useAuthStore((s) => s.logout)
  const disconnect = useControlPlaneStore((s) => s.disconnect)
  const serverConfig = useControlPlaneStore((s) => s.serverConfig)
  const error = useControlPlaneStore((s) => s.error)
  const clearError = useControlPlaneStore((s) => s.clearError)
  const isLoading = useControlPlaneStore((s) => s.isLoading)
  const applyApiKey = useControlPlaneStore((s) => s.applyApiKey)
  const userRole = useControlPlaneStore((s) => s.userRole)
  const userClientId = useControlPlaneStore((s) => s.userClientId)
  const userOrgId = useControlPlaneStore((s) => s.userOrgId)
  const selectedOrgName = useControlPlaneStore((s) => s.selectedOrgName)
  const setSelectedOrgName = useControlPlaneStore((s) => s.setSelectedOrgName)
  const connectionStatus = useControlPlaneStore((s) => s.connectionStatus)
  const initialUrl = resolveControlPlaneUrl({ previousUrl: serverConfig?.url })
  const [apiKey, setApiKey] = useState(serverConfig?.api_key ?? '')
  const [url, setUrl] = useState(initialUrl)
  const [githubLogin, setGithubLogin] = useState(user?.login ?? '')
  const [activeOrgName, setActiveOrgName] = useState(selectedOrgName || '')
  const [localError, setLocalError] = useState<string | null>(null)
  const [hasSubmitted, setHasSubmitted] = useState(false)
  const [isSubmitting, setIsSubmitting] = useState(false)

  const resolvedUrl = resolveControlPlaneUrl({
    inputUrl: url,
    previousUrl: serverConfig?.url,
  })

  useEffect(() => {
    if (user?.login) {
      setGithubLogin(user.login)
    }
  }, [user?.login])

  useEffect(() => {
    if (serverConfig?.api_key) {
      setApiKey(serverConfig.api_key)
    }
    if (serverConfig?.url) {
      setUrl(resolveControlPlaneUrl({ previousUrl: serverConfig.url }))
    }
  }, [serverConfig?.api_key, serverConfig?.url])

  const handleContinue = async () => {
    setHasSubmitted(true)
    setLocalError(null)
    clearError()
    const currentGitHubLogin = user?.login?.trim()
    const enteredGitHubLogin = githubLogin.trim()
    if (!currentGitHubLogin) {
      setLocalError('No hay sesión GitHub activa. Vuelve a autenticar con Device Flow.')
      return
    }
    if (!enteredGitHubLogin) {
      setLocalError('Ingresa tu usuario GitHub.')
      return
    }
    if (enteredGitHubLogin.toLowerCase() !== currentGitHubLogin.toLowerCase()) {
      setLocalError(`El usuario debe coincidir con tu sesión Device Flow: @${currentGitHubLogin}.`)
      return
    }
    if (!apiKey.trim()) {
      setLocalError('Ingresa tu API key de GitGov.')
      return
    }

    setIsSubmitting(true)
    try {
      const ok = await applyApiKey(apiKey, resolvedUrl)
      if (!ok) {
        const state = useControlPlaneStore.getState()
        setLocalError(state.error || 'No se pudo validar la API key en Control Plane.')
        return
      }
      const state = useControlPlaneStore.getState()
      if (state.userRole === 'Admin' && !activeOrgName.trim()) {
        setLocalError('Esta API key es Admin. Completa org_name activo para continuar.')
        return
      }
      if (activeOrgName.trim()) {
        setSelectedOrgName(activeOrgName.trim())
      }
    } finally {
      setIsSubmitting(false)
    }
  }

  const handleUrlChange = (nextUrl: string) => {
    setUrl(nextUrl)
    setLocalError(null)
    clearError()
  }

  const handleApiKeyChange = (nextApiKey: string) => {
    setApiKey(nextApiKey)
    setLocalError(null)
    clearError()
  }

  const visibleError = localError || (hasSubmitted ? error : null)

  return (
    <div className="min-h-dvh bg-surface-950 flex items-center justify-center p-4">
      <div className="max-w-md w-full animate-fade-in">
        <div className="text-center mb-6">
          <div className="inline-flex items-center justify-center w-12 h-12 rounded-xl bg-brand-600 mb-4">
            <ShieldCheck size={22} className="text-white" />
          </div>
          <h1 className="text-xl font-semibold text-white mb-1 tracking-tight">Paso 2 de 2</h1>
          <p className="text-xs text-surface-500">Validar acceso al Control Plane</p>
        </div>

        <div className="glass-card p-6 space-y-3">
          <div className="text-xs text-surface-400">
            GitHub autenticado como <span className="text-surface-200 font-medium">@{user?.login ?? 'desconocido'}</span>.
            Completa acceso de Control Plane con tu API key de GitGov.
          </div>

          <div className="rounded-lg border border-white/8 bg-white/[0.03] p-3 text-xs text-surface-400 space-y-1">
            <div>
              <span className="text-surface-200 font-medium">GitHub</span> identifica a la persona que opera Desktop.
            </div>
            <div>
              <span className="text-surface-200 font-medium">GitGov API key</span> autoriza rol, org y evidencia del Control Plane.
            </div>
            <div className="text-surface-500">
              Ambas credenciales se guardan en almacenamiento seguro local; solo se piden de nuevo si faltan, expiran o cambias de usuario.
            </div>
          </div>

          <div>
            <label htmlFor="cp-github-login-auth" className="block text-xs text-surface-500 mb-1">
              Usuario GitHub
            </label>
            <input
              id="cp-github-login-auth"
              type="text"
              value={githubLogin}
              onChange={(e) => setGithubLogin(e.target.value)}
              className="input"
              placeholder="ej: octocat"
            />
          </div>

          <div>
            <label htmlFor="cp-url-auth" className="block text-xs text-surface-500 mb-1">URL Control Plane</label>
            <input
              id="cp-url-auth"
              type="text"
              value={url}
              onChange={(e) => handleUrlChange(e.target.value)}
              className="input"
              placeholder={DEFAULT_CONTROL_PLANE_URL}
            />
            <p className="mt-1 text-[10px] text-surface-500">
              Usa localhost solo si el Control Plane local esta levantado. La URL configurada en el entorno se usa como predeterminada.
            </p>
          </div>

          <div>
            <label htmlFor="cp-key-auth" className="block text-xs text-surface-500 mb-1">API key</label>
            <input
              id="cp-key-auth"
              type="password"
              value={apiKey}
              onChange={(e) => handleApiKeyChange(e.target.value)}
              className="input"
              placeholder="Pega tu API key de GitGov"
            />
          </div>

          <div>
            <label htmlFor="cp-org-name-auth" className="block text-xs text-surface-500 mb-1">
              org_name activo (requerido para Admin)
            </label>
            <input
              id="cp-org-name-auth"
              type="text"
              value={activeOrgName}
              onChange={(e) => setActiveOrgName(e.target.value)}
              className="input"
              placeholder="ej: mi-organizacion"
            />
          </div>

          <Button onClick={handleContinue} loading={isSubmitting || isLoading} className="w-full">
            <KeyRound size={14} />
            Entrar al Control Plane
          </Button>

          <Button
            onClick={async () => {
              disconnect()
              await logout()
            }}
            variant="ghost"
            className="w-full"
          >
            Cambiar usuario GitHub
          </Button>

          {userRole && (
            <div className="rounded border border-white/10 bg-surface-900/70 p-3 text-xs text-surface-300 space-y-1">
              <div className="text-surface-500 uppercase tracking-widest text-[10px]">Sesión actual</div>
              <div>client_id: <span className="mono-data text-surface-100">{userClientId ?? 'n/a'}</span></div>
              <div>role: <span className="mono-data text-surface-100">{userRole}</span></div>
              <div>org_id: <span className="mono-data text-surface-100">{userOrgId ?? 'null'}</span></div>
            </div>
          )}

          {visibleError && (
            <div className="p-2 bg-danger-500/20 border border-danger-500/50 rounded text-danger-400 text-xs">
              {visibleError}
            </div>
          )}

          {connectionStatus === 'maintenance' && (
            <div className="p-2 bg-warning-500/20 border border-warning-500/50 rounded text-warning-300 text-xs">
              El servidor está en mantenimiento/reinicio. Reintenta en unos segundos.
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

