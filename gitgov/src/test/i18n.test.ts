import { beforeEach, describe, expect, it } from 'vitest'
import i18n, {
  APP_LANGUAGE_STORAGE_KEY,
  DEFAULT_APP_LANGUAGE,
  getAppLanguage,
  readStoredLanguage,
  setAppLanguage,
} from '@/lib/i18n'

describe('i18n language preference', () => {
  beforeEach(async () => {
    localStorage.clear()
    await setAppLanguage(DEFAULT_APP_LANGUAGE)
  })

  it('defaults content language to English', () => {
    localStorage.clear()

    expect(readStoredLanguage()).toBe('en')
    expect(getAppLanguage()).toBe('en')
    expect(i18n.t('login.connectGitHub')).toBe('Connect with GitHub')
  })

  it('persists Spanish and updates translated content', async () => {
    await setAppLanguage('es')

    expect(localStorage.getItem(APP_LANGUAGE_STORAGE_KEY)).toBe('es')
    expect(getAppLanguage()).toBe('es')
    expect(i18n.t('login.connectGitHub')).toBe('Conectar con GitHub')
    expect(i18n.t('navigation.settings')).toBe('Configuración')
    expect(i18n.t('settings.tabs.preferences.label')).toBe('Preferencias')
    expect(i18n.t('settings.notifications.enable')).toBe('Activar notificaciones')
    expect(i18n.t('governance.sections.evidence.label')).toBe('Evidencia')
  })

  it('keeps first-class module chrome translatable in both languages', async () => {
    const requiredKeys = [
      'navigation.home',
      'navigation.governance',
      'settings.title',
      'settings.tabs.preferences.label',
      'settings.tabs.connection.label',
      'settings.tabs.organization.label',
      'settings.tabs.account.label',
      'settings.tabs.repository.label',
      'governance.title',
      'governance.sections.evidence.label',
      'governance.sections.policy.label',
      'governance.sections.adoption.label',
      'governance.sections.releases.label',
      'governance.sections.copilot.label',
    ]

    for (const language of ['en', 'es'] as const) {
      await setAppLanguage(language)
      for (const key of requiredKeys) {
        expect(i18n.t(key), `${language}:${key}`).not.toBe(key)
      }
    }
  })

  it('falls back to English for unsupported stored values', () => {
    localStorage.setItem(APP_LANGUAGE_STORAGE_KEY, 'fr')

    expect(readStoredLanguage()).toBe('en')
  })
})
