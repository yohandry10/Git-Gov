import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useAuthStore } from '@/store/useAuthStore'
import { Button } from '@/components/shared/Button'
import { Spinner } from '@/components/shared/Spinner'
import { Github, ExternalLink, Download, Copy, Check, ShieldCheck } from 'lucide-react'
import { isTauriDesktop, tauriInvoke } from '@/lib/tauri'

const GITHUB_DEVICE_URL = 'https://github.com/login/device'
const PUBLIC_REPO_URL =
  (import.meta.env.VITE_PUBLIC_REPO_URL as string | undefined)?.trim() ||
  'https://github.com'

export function LoginScreen() {
  const { authStep, deviceFlowInfo, error, startAuth, pollAuth, cancelAuth, clearError } = useAuthStore()
  const { t } = useTranslation()
  const [copied, setCopied] = useState(false)
  const isDesktop = isTauriDesktop()

  const handleCopyCode = () => {
    if (deviceFlowInfo) {
      navigator.clipboard.writeText(deviceFlowInfo.user_code)
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    }
  }

  const handleOpenGitHub = async () => {
    const deviceUrl =
      deviceFlowInfo?.verification_uri?.includes('github.com/login/device')
        ? deviceFlowInfo.verification_uri
        : GITHUB_DEVICE_URL

    if (isDesktop) {
      try {
        await tauriInvoke('cmd_open_external_url', { url: deviceUrl })
        return
      } catch {
        // Fallback to browser open if native command fails.
      }
    }
    window.open(deviceUrl, '_blank', 'noopener,noreferrer')
  }

  if (!isDesktop) {
    return (
      <div className="min-h-dvh bg-surface-950 flex items-center justify-center p-4">
        <div className="max-w-sm w-full animate-fade-in">
          <div className="text-center mb-8">
            <div className="inline-flex items-center justify-center w-12 h-12 rounded-xl bg-brand-600 mb-5">
              <Github size={24} className="text-white" />
            </div>
            <h1 className="text-2xl font-semibold text-white mb-2 tracking-tight">GitGov</h1>
            <p className="text-sm text-surface-500">{t('login.tagline')}</p>
          </div>

          <div className="glass-card p-6">
            <div className="text-center mb-4">
              <Download size={36} strokeWidth={1.5} className="mx-auto text-surface-400 mb-4" />
              <h2 className="text-lg font-semibold text-white mb-2">
                {t('login.desktopRequiredTitle')}
              </h2>
              <p className="text-sm text-surface-400">
                {t('login.desktopRequiredBody')}
              </p>
            </div>

            <div className="bg-surface-900/50 rounded-xl p-4 mb-4 border border-surface-700/30">
              <p className="text-surface-400 text-xs mb-2">
                {t('login.desktopStepsLabel')}
              </p>
              <ol className="text-surface-500 text-xs list-decimal list-inside space-y-1">
                <li>{t('login.desktopStepDownload')}</li>
                <li>{t('login.desktopStepInstall')}</li>
                <li>{t('login.desktopStepOpen')}</li>
              </ol>
            </div>

            <Button
              onClick={() => window.open(PUBLIC_REPO_URL, '_blank')}
              className="w-full"
              size="lg"
            >
              <ExternalLink size={16} />
              {t('login.downloadDesktop')}
            </Button>
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-dvh bg-surface-950 flex items-center justify-center p-4">
      <div className="relative max-w-sm w-full animate-fade-in">
        <div className="text-center mb-8">
          <div className="inline-flex items-center justify-center w-12 h-12 rounded-xl bg-brand-600 mb-5">
            <Github size={24} className="text-white" />
          </div>
          <h1 className="text-2xl font-semibold text-white mb-2 tracking-tight">GitGov</h1>
          <p className="text-sm text-surface-500">{t('login.tagline')}</p>
        </div>

        {authStep === 'idle' && (
          <div className="glass-card p-6 animate-slide-up">
            {error && (
              <div className="mb-4 p-3 bg-danger-500/10 border border-danger-500/20 rounded-xl text-danger-400 text-xs flex items-center justify-between">
                <span>{error}</span>
                <button onClick={clearError} className="ml-2 text-danger-400 hover:text-danger-300 underline text-[11px]">
                  {t('common.close')}
                </button>
              </div>
            )}
            <p className="text-sm text-surface-400 text-center mb-5">
              {t('login.connectPrompt')}
            </p>
            <div className="mb-5 rounded-lg border border-white/8 bg-white/[0.03] p-3 text-xs text-surface-400">
              <div className="flex items-start gap-2">
                <ShieldCheck size={14} className="mt-0.5 shrink-0 text-brand-300" />
                <span>
                  {t('login.sessionReuse')}
                </span>
              </div>
            </div>
            <Button onClick={startAuth} className="w-full" size="lg">
              <Github size={18} />
              {t('login.connectGitHub')}
            </Button>
          </div>
        )}

        {authStep === 'waiting_device' && deviceFlowInfo && (
          <div className="glass-card p-6 animate-slide-up">
            {error && (
              <div className="mb-4 p-3 bg-warning-500/10 border border-warning-500/30 rounded-xl text-warning-300 text-xs flex items-center justify-between gap-3">
                <span>{error}</span>
                <button onClick={clearError} className="shrink-0 text-warning-200 hover:text-warning-100 underline text-[11px]">
                  {t('common.close')}
                </button>
              </div>
            )}
            <p className="text-sm text-surface-400 text-center mb-5">
              {t('login.deviceInstruction')}
            </p>

            <button
              onClick={handleCopyCode}
              className="w-full bg-surface-900/60 rounded-xl p-5 mb-5 text-center border border-surface-700/30 hover:border-surface-600/50 transition-colors group cursor-pointer"
            >
              <code className="text-3xl mono-data text-white tracking-[0.2em] font-semibold">
                {deviceFlowInfo.user_code}
              </code>
              <span className="flex items-center justify-center gap-1.5 mt-3 text-xs text-surface-500 group-hover:text-surface-400 transition-colors">
                {copied ? (
                  <>
                    <Check size={13} className="text-success-400" />
                    <span className="text-success-400">{t('login.copied')}</span>
                  </>
                ) : (
                  <>
                    <Copy size={13} />
                    <span>{t('login.copyCode')}</span>
                  </>
                )}
              </span>
            </button>

            <div className="flex gap-3">
              <Button onClick={handleOpenGitHub} variant="secondary" className="flex-1">
                <ExternalLink size={14} />
                {t('login.openGitHub')}
              </Button>
              <Button onClick={pollAuth} className="flex-1">
                {t('common.continue')}
              </Button>
            </div>
            <Button onClick={cancelAuth} variant="ghost" className="w-full mt-3">
              {t('common.cancel')}
            </Button>
          </div>
        )}

        {authStep === 'polling' && (
          <div className="glass-card p-6 text-center animate-slide-up">
            <Spinner size="lg" className="mx-auto mb-4" />
            <p className="text-white font-medium mb-1 text-sm">{t('login.connectingTitle')}</p>
            <p className="text-surface-500 text-xs">
              {t('login.connectingBody')}
            </p>
            <Button onClick={cancelAuth} variant="ghost" className="w-full mt-4">
              {t('common.cancel')}
            </Button>
          </div>
        )}
      </div>
    </div>
  )
}

