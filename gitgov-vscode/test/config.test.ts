import { describe, expect, it } from 'vitest'
import {
  GITGOV_API_KEY_SECRET,
  buildStoredConnection,
  clearApiKey,
  normalizeApiUrl,
  readApiKey,
  storeApiKey,
} from '../src/config'
import type { GitGovSecretStore } from '../src/types'

class MemorySecrets implements GitGovSecretStore {
  readonly values = new Map<string, string>()

  async get(key: string): Promise<string | undefined> {
    return this.values.get(key)
  }

  async store(key: string, value: string): Promise<void> {
    this.values.set(key, value)
  }

  async delete(key: string): Promise<void> {
    this.values.delete(key)
  }
}

describe('GitGov connection configuration', () => {
  it('normalizes API URL and never stores API key in plain config', () => {
    const stored = buildStoredConnection({ apiUrl: ' https://gitgov-api.onrender.com/// ', orgName: ' yohandry10 ' }, 'secret-key')

    expect(stored).toEqual({
      apiUrl: 'https://gitgov-api.onrender.com',
      orgName: 'yohandry10',
      hasApiKey: true,
    })
    expect(Object.values(stored)).not.toContain('secret-key')
    expect(normalizeApiUrl('https://example.com///')).toBe('https://example.com')
  })

  it('stores and clears the API key only through SecretStorage-compatible API', async () => {
    const secrets = new MemorySecrets()

    await storeApiKey(secrets, '  readonly-key  ')
    expect(secrets.values.get(GITGOV_API_KEY_SECRET)).toBe('readonly-key')
    expect(await readApiKey(secrets)).toBe('readonly-key')

    await clearApiKey(secrets)
    expect(await readApiKey(secrets)).toBeUndefined()
  })

  it('rejects empty API keys before writing to secrets', async () => {
    const secrets = new MemorySecrets()

    await expect(storeApiKey(secrets, '   ')).rejects.toThrow('GitGov API key cannot be empty.')
    expect(secrets.values.size).toBe(0)
  })
})
