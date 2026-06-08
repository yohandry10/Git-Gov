import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import clsx from 'clsx'
import { Globe2 } from 'lucide-react'
import {
  LANGUAGE_OPTIONS,
  getAppLanguage,
  normalizeAppLanguage,
  setAppLanguage,
  type AppLanguage,
} from '@/lib/i18n'

interface LanguagePreferenceSelectorProps {
  compact?: boolean
  className?: string
}

export function LanguagePreferenceSelector({
  compact = false,
  className,
}: LanguagePreferenceSelectorProps) {
  const { t, i18n } = useTranslation()
  const [pendingLanguage, setPendingLanguage] = useState<AppLanguage | null>(null)
  const currentLanguage = normalizeAppLanguage(i18n.resolvedLanguage || i18n.language || getAppLanguage())
  const activeLabel =
    LANGUAGE_OPTIONS.find((option) => option.value === currentLanguage)?.nativeLabel ?? 'English'

  const handleLanguageChange = async (language: AppLanguage) => {
    if (language === currentLanguage || pendingLanguage) return
    setPendingLanguage(language)
    try {
      await setAppLanguage(language)
    } finally {
      setPendingLanguage(null)
    }
  }

  return (
    <div
      className={clsx(
        'rounded-lg border border-white/8 bg-white/[0.03] p-3',
        compact ? 'space-y-2' : 'space-y-3',
        className,
      )}
    >
      <div className="flex items-start gap-2">
        <Globe2 size={14} className="mt-0.5 shrink-0 text-brand-300" />
        <div className="min-w-0">
          <p className="text-xs font-semibold text-surface-100">{t('language.title')}</p>
          <p className="text-[11px] leading-relaxed text-surface-400">{t('language.prompt')}</p>
        </div>
      </div>

      <div
        role="radiogroup"
        aria-label={t('language.title')}
        className="grid grid-cols-2 gap-2"
      >
        {LANGUAGE_OPTIONS.map((option) => {
          const isActive = option.value === currentLanguage
          const isPending = pendingLanguage === option.value
          return (
            <button
              key={option.value}
              type="button"
              role="radio"
              aria-checked={isActive}
              onClick={() => void handleLanguageChange(option.value)}
              disabled={Boolean(pendingLanguage)}
              className={clsx(
                'min-h-9 rounded-md border px-3 py-2 text-left transition-colors',
                'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-500/50',
                'disabled:cursor-not-allowed disabled:opacity-70',
                isActive
                  ? 'border-brand-500/60 bg-brand-500/15 text-white'
                  : 'border-surface-700/40 bg-surface-900/40 text-surface-300 hover:border-surface-500/60 hover:bg-surface-800/70',
              )}
            >
              <span className="block text-xs font-semibold leading-tight">
                {option.nativeLabel}
              </span>
              <span className="block text-[10px] leading-tight text-surface-500">
                {isPending ? t('language.pending') : t(`language.optionSubtitles.${option.value}`)}
              </span>
            </button>
          )
        })}
      </div>

      <p className="text-[10px] leading-relaxed text-surface-500">
        {t('language.active', { language: activeLabel })} {t('language.helper')}
      </p>
    </div>
  )
}
