import type { GitGovConnectionConfig, GitGovSecretStore, StoredGitGovConnection } from './types'

export const GITGOV_API_KEY_SECRET = 'gitgov.apiKey'

export function normalizeApiUrl(value: string): string {
  return value.trim().replace(/\/+$/, '')
}

export function normalizeOrgName(value: string): string {
  return value.trim()
}

export function buildStoredConnection(
  config: GitGovConnectionConfig,
  apiKey: string | undefined,
): StoredGitGovConnection {
  return {
    apiUrl: normalizeApiUrl(config.apiUrl),
    orgName: normalizeOrgName(config.orgName),
    hasApiKey: Boolean(apiKey?.trim()),
  }
}

export async function storeApiKey(secretStore: GitGovSecretStore, apiKey: string): Promise<void> {
  const trimmed = apiKey.trim()
  if (!trimmed) {
    throw new Error('GitGov API key cannot be empty.')
  }
  await secretStore.store(GITGOV_API_KEY_SECRET, trimmed)
}

export async function readApiKey(secretStore: GitGovSecretStore): Promise<string | undefined> {
  const apiKey = await secretStore.get(GITGOV_API_KEY_SECRET)
  return apiKey?.trim() || undefined
}

export async function clearApiKey(secretStore: GitGovSecretStore): Promise<void> {
  await secretStore.delete(GITGOV_API_KEY_SECRET)
}
