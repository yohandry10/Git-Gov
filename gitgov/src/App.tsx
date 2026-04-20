import { useEffect, useState } from 'react'
import { AppRouter } from './router'
import { useAuthStore } from './store/useAuthStore'
import { useControlPlaneStore } from './store/useControlPlaneStore'
import { useUpdateStore } from './store/useUpdateStore'
import { ToastContainer } from './components/shared/Toast'
import { FolderGit2 } from 'lucide-react'
import { ErrorBoundary } from './components/shared/ErrorBoundary'
import { Button } from './components/shared/Button'
import { detectBrowserTimezone, formatTs } from './lib/timezone'

function SplashScreen() {
  return (
    <div className="min-h-dvh bg-surface-950 flex flex-col items-center justify-center">
      <div className="animate-scale-in flex flex-col items-center">
        <div className="w-12 h-12 rounded-xl bg-brand-600 flex items-center justify-center mb-5">
          <FolderGit2 size={24} className="text-white" />
        </div>
        <h1 className="text-xl font-semibold text-white mb-1 tracking-tight">GitGov</h1>
        <p className="text-xs text-surface-500 mb-8">Governance Platform</p>
        <div className="flex gap-1.5">
          <div className="w-1.5 h-1.5 rounded-full bg-surface-500 animate-pulse" />
          <div className="w-1.5 h-1.5 rounded-full bg-surface-600 animate-pulse [animation-delay:150ms]" />
          <div className="w-1.5 h-1.5 rounded-full bg-surface-700 animate-pulse [animation-delay:300ms]" />
        </div>
      </div>
    </div>
  )
}

interface MandatoryUpdateScreenProps {
  currentVersion?: string
  targetVersion?: string
  minimumSupportedVersion?: string | null
  reason?: string | null
  lastCheckedAt?: number | null
  isChecking: boolean
  isDownloading: boolean
  fallbackDownloadUrl: string
  onRetryCheck: () => void
  onDownloadInstall: () => void
}

function MandatoryUpdateScreen(props: MandatoryUpdateScreenProps) {
  const timezone = detectBrowserTimezone()
  return (
    <div className="min-h-dvh bg-surface-950 flex flex-col items-center justify-center p-6">
      <div className="w-full max-w-xl rounded-2xl border border-warning-500/30 bg-surface-900/70 p-6">
        <div className="mb-3 inline-flex items-center gap-2 rounded-md border border-warning-500/40 bg-warning-500/10 px-2 py-1 text-[10px] uppercase tracking-widest text-warning-300">
          Actualización obligatoria
        </div>
        <h1 className="text-lg font-semibold text-white tracking-tight mb-2">
          Esta versión de GitGov requiere actualización
        </h1>
        <p className="text-xs text-surface-300 leading-relaxed mb-3">
          Versión actual: <span className="font-mono text-surface-100">v{props.currentVersion ?? 'desconocida'}</span>
          {props.targetVersion ? (
            <> · Disponible: <span className="font-mono text-surface-100">v{props.targetVersion}</span></>
          ) : null}
          {props.minimumSupportedVersion ? (
            <> · Mínimo soportado: <span className="font-mono text-surface-100">v{props.minimumSupportedVersion}</span></>
          ) : null}
        </p>
        {props.reason ? (
          <p className="text-xs text-warning-200 mb-3 leading-relaxed">{props.reason}</p>
        ) : null}
        {props.lastCheckedAt ? (
          <p className="text-[10px] text-surface-500 mb-4">
            Última verificación: {formatTs(props.lastCheckedAt, timezone)}
          </p>
        ) : null}

        <div className="flex flex-wrap gap-2">
          <Button
            size="sm"
            onClick={props.onDownloadInstall}
            loading={props.isDownloading}
            disabled={!props.targetVersion}
          >
            Descargar e instalar
          </Button>
          <Button
            size="sm"
            variant="outline"
            onClick={props.onRetryCheck}
            loading={props.isChecking}
            disabled={props.isDownloading}
          >
            Revalidar actualización
          </Button>
          <Button
            size="sm"
            variant="secondary"
            onClick={() => window.open(props.fallbackDownloadUrl, '_blank', 'noopener,noreferrer')}
          >
            Descarga manual
          </Button>
        </div>
      </div>
    </div>
  )
}

function App() {
  const { checkExistingSession, isLoading } = useAuthStore()
  const initFromEnv = useControlPlaneStore((s) => s.initFromEnv)
  const initializeUpdater = useUpdateStore((s) => s.initializeUpdater)
  const isMandatoryUpdateRequired = useUpdateStore((s) => s.isMandatoryUpdateRequired)
  const mandatoryUpdateReason = useUpdateStore((s) => s.mandatoryUpdateReason)
  const minimumSupportedVersion = useUpdateStore((s) => s.minimumSupportedVersion)
  const updateInfo = useUpdateStore((s) => s.updateInfo)
  const lastCheckedAt = useUpdateStore((s) => s.lastCheckedAt)
  const isCheckingUpdater = useUpdateStore((s) => s.isChecking)
  const isDownloadingUpdater = useUpdateStore((s) => s.isDownloading)
  const fallbackDownloadUrl = useUpdateStore((s) => s.fallbackDownloadUrl)
  const checkForUpdates = useUpdateStore((s) => s.checkForUpdates)
  const downloadAndInstall = useUpdateStore((s) => s.downloadAndInstall)
  const [initialized, setInitialized] = useState(false)

  useEffect(() => {
    const init = async () => {
      try {
        // Startup must render fast even if Control Plane is unreachable.
        await checkExistingSession()
      } finally {
        setInitialized(true)
        // Keep Control Plane bootstrap in background; do not block Splash.
        void initFromEnv()
      }
    }
    void init()
  }, [checkExistingSession, initFromEnv])

  useEffect(() => {
    if (!initialized || isLoading) return
    void initializeUpdater()
  }, [initialized, isLoading, initializeUpdater])

  if (!initialized || isLoading) {
    return <SplashScreen />
  }

  if (isMandatoryUpdateRequired) {
    return (
      <MandatoryUpdateScreen
        currentVersion={updateInfo?.currentVersion}
        targetVersion={updateInfo?.version}
        minimumSupportedVersion={minimumSupportedVersion}
        reason={mandatoryUpdateReason}
        lastCheckedAt={lastCheckedAt}
        isChecking={isCheckingUpdater}
        isDownloading={isDownloadingUpdater}
        fallbackDownloadUrl={fallbackDownloadUrl}
        onRetryCheck={() => void checkForUpdates({ manual: true, force: true })}
        onDownloadInstall={() => void downloadAndInstall()}
      />
    )
  }

  return (
    <ErrorBoundary>
      <AppRouter />
      <ToastContainer />
    </ErrorBoundary>
  )
}

export default App
