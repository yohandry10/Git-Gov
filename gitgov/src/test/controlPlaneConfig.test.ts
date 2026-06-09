import { describe, expect, it, vi, beforeEach } from 'vitest'
import {
  DEFAULT_CONTROL_PLANE_URL,
  formatControlPlaneConnectionError,
  isLocalControlPlaneUrl,
  normalizeControlPlaneUrl,
  resolveControlPlaneUrl,
  validateControlPlaneUrl,
} from '@/lib/controlPlaneConfig'

describe('controlPlaneConfig', () => {
  beforeEach(() => {
    vi.unstubAllEnvs()
  })

  it('normalizes localhost and strips route paths from Control Plane URLs', () => {
    expect(normalizeControlPlaneUrl('http://localhost:3000/health?x=1')).toBe(DEFAULT_CONTROL_PLANE_URL)
    expect(normalizeControlPlaneUrl('https://gitgov-api.onrender.com/docs')).toBe('https://gitgov-api.onrender.com')
  })

  it('prefers configured env URL over stale forced localhost defaults', () => {
    expect(resolveControlPlaneUrl({
      previousUrl: DEFAULT_CONTROL_PLANE_URL,
      storedUrl: DEFAULT_CONTROL_PLANE_URL,
      envUrl: 'https://gitgov-api.onrender.com',
    })).toBe('https://gitgov-api.onrender.com')
  })

  it('keeps an explicit user-entered URL even when env exists', () => {
    expect(resolveControlPlaneUrl({
      inputUrl: 'http://127.0.0.1:3001',
      envUrl: 'https://gitgov-api.onrender.com',
    })).toBe('http://127.0.0.1:3001')
  })

  it('recognizes IPv4 and IPv6 loopback URLs as local Control Plane targets', () => {
    expect(isLocalControlPlaneUrl('http://127.0.0.1:3000')).toBe(true)
    expect(isLocalControlPlaneUrl('http://localhost:3000')).toBe(true)
    expect(isLocalControlPlaneUrl('http://[::1]:3000')).toBe(true)
    expect(isLocalControlPlaneUrl('https://gitgov-api.onrender.com')).toBe(false)
  })

  it('rejects unsafe Control Plane URLs before persistence', () => {
    expect(validateControlPlaneUrl('ftp://gitgov.cloud')).toContain('http or https')
    expect(validateControlPlaneUrl('https://user:pass@gitgov.cloud')).toContain('must not contain embedded credentials')
    expect(validateControlPlaneUrl('http://gitgov.cloud')).toContain('must use https')
    expect(validateControlPlaneUrl('http://127.0.0.1:3000')).toBeNull()
    expect(validateControlPlaneUrl('https://gitgov.cloud')).toBeNull()
  })

  it('formats localhost connection failures as actionable product messages', () => {
    expect(formatControlPlaneConnectionError(
      'Network error: error sending request for url (http://127.0.0.1:3000/health)',
      DEFAULT_CONTROL_PLANE_URL,
    )).toContain('No hay un Control Plane local escuchando')
  })

  it('formats IPv6 localhost connection failures as local Control Plane messages', () => {
    expect(formatControlPlaneConnectionError(
      'Network error: error sending request for url (http://[::1]:3000/health)',
      'http://[::1]:3000',
    )).toContain('No hay un Control Plane local escuchando')
  })
})
