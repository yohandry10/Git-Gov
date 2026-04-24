import { describe, it, expect, vi } from 'vitest'

// Mock tauri helper
vi.mock('@/lib/tauri', () => ({
  isTauriDesktop: vi.fn().mockReturnValue(false),
}))

import {
  compareAppVersions,
  evaluateDesktopUpdateEnforcement,
  getDesktopUpdateFallbackUrl,
  canUseDesktopUpdater,
  isUpdaterNotConfiguredError,
  normalizeUpdaterErrorMessage,
} from '@/lib/updater'

describe('updater utility', () => {
  describe('getDesktopUpdateFallbackUrl', () => {
    it('returns generic GitHub URL by default when no env is configured', () => {
      const url = getDesktopUpdateFallbackUrl()
      expect(url).toBe('https://github.com')
    })

    it('returns same URL for stable channel with generic default URL', () => {
      const url = getDesktopUpdateFallbackUrl('stable')
      expect(url).toBe('https://github.com')
    })

    it('returns same URL for beta channel with generic default URL', () => {
      const url = getDesktopUpdateFallbackUrl('beta')
      expect(url).toBe('https://github.com')
    })
  })

  describe('canUseDesktopUpdater', () => {
    it('returns false when not in Tauri', () => {
      expect(canUseDesktopUpdater()).toBe(false)
    })
  })

  describe('isUpdaterNotConfiguredError', () => {
    it('detects updater config errors', () => {
      expect(isUpdaterNotConfiguredError('Updater endpoint not configured')).toBe(true)
      expect(isUpdaterNotConfiguredError('updater pubkey missing')).toBe(true)
      expect(isUpdaterNotConfiguredError('Updater config error')).toBe(true)
    })

    it('returns false for unrelated errors', () => {
      expect(isUpdaterNotConfiguredError('Network timeout')).toBe(false)
      expect(isUpdaterNotConfiguredError('Connection refused')).toBe(false)
      expect(isUpdaterNotConfiguredError(null)).toBe(false)
      expect(isUpdaterNotConfiguredError(undefined)).toBe(false)
    })

    it('handles non-string errors', () => {
      expect(isUpdaterNotConfiguredError(new Error('Updater endpoint missing'))).toBe(true)
      expect(isUpdaterNotConfiguredError(42)).toBe(false)
    })
  })

  describe('normalizeUpdaterErrorMessage', () => {
    it('returns user-friendly message for decoding error', () => {
      const msg = normalizeUpdaterErrorMessage('error decoding response body from server')
      expect(msg).toContain('latest.json')
      expect(msg).toContain('Descarga manual')
    })

    it('returns user-friendly message for 404 latest.json', () => {
      const msg = normalizeUpdaterErrorMessage('404 latest.json not found')
      expect(msg).toContain('latest.json')
    })

    it('returns user-friendly message for TLS errors', () => {
      const msg = normalizeUpdaterErrorMessage('TLS handshake failed')
      expect(msg).toContain('TLS')
    })

    it('returns user-friendly message for certificate errors', () => {
      const msg = normalizeUpdaterErrorMessage('certificate verification failed')
      expect(msg).toContain('certificado')
    })

    it('returns raw message for unknown errors', () => {
      const msg = normalizeUpdaterErrorMessage('Something unexpected happened')
      expect(msg).toBe('Something unexpected happened')
    })

    it('handles null/undefined', () => {
      // The mock returns String(e), so String(null) = '', String(undefined) = ''
      const resultNull = normalizeUpdaterErrorMessage(null)
      const resultUndefined = normalizeUpdaterErrorMessage(undefined)
      expect(typeof resultNull).toBe('string')
      expect(typeof resultUndefined).toBe('string')
    })
  })

  describe('compareAppVersions', () => {
    it('compares semantic versions correctly', () => {
      expect(compareAppVersions('1.2.3', '1.2.3')).toBe(0)
      expect(compareAppVersions('1.2.3', '1.2.4')).toBeLessThan(0)
      expect(compareAppVersions('1.3.0', '1.2.9')).toBeGreaterThan(0)
    })

    it('treats prerelease as lower than stable', () => {
      expect(compareAppVersions('1.0.0-beta.1', '1.0.0')).toBeLessThan(0)
      expect(compareAppVersions('1.0.0', '1.0.0-rc.1')).toBeGreaterThan(0)
    })
  })

  describe('evaluateDesktopUpdateEnforcement', () => {
    it('requires update when current version is below minimum supported', () => {
      const enforcement = evaluateDesktopUpdateEnforcement({
        currentVersion: '1.4.0',
        version: '1.5.0',
        rawJson: {
          min_supported_version: '1.5.0',
        },
      })
      expect(enforcement.required).toBe(true)
      expect(enforcement.reason).toBe('min-supported-version')
      expect(enforcement.currentBelowMinSupported).toBe(true)
      expect(enforcement.minSupportedVersion).toBe('1.5.0')
    })

    it('requires update when force flag is set', () => {
      const enforcement = evaluateDesktopUpdateEnforcement({
        currentVersion: '1.5.0',
        version: '1.5.1',
        rawJson: {
          force_update: true,
          force_update_reason: 'Security hotfix',
        },
      })
      expect(enforcement.required).toBe(true)
      expect(enforcement.reason).toBe('force-update')
      expect(enforcement.note).toBe('Security hotfix')
    })

    it('does not require update by default', () => {
      const enforcement = evaluateDesktopUpdateEnforcement({
        currentVersion: '1.5.0',
        version: '1.5.1',
        rawJson: {},
      })
      expect(enforcement.required).toBe(false)
      expect(enforcement.reason).toBe('none')
    })
  })
})

