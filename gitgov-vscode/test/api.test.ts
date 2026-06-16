import { describe, expect, it, vi } from 'vitest'
import { GitGovReadOnlyClient, READ_ONLY_ENDPOINTS } from '../src/api'

function jsonResponse(payload: unknown, status = 200): Response {
  return new Response(JSON.stringify(payload), {
    status,
    headers: { 'content-type': 'application/json' },
  })
}

describe('GitGovReadOnlyClient', () => {
  it('uses only documented GET endpoints for governance context reads', async () => {
    const calls: Array<{ url: string; init?: RequestInit }> = []
    const fetchImpl = vi.fn(async (url: string | URL | Request, init?: RequestInit) => {
      calls.push({ url: String(url), init })
      if (String(url).includes('/executive/repositories')) {
        return jsonResponse({ repositories: [{ repository_full_name: 'yohandry10/Git-Gov', posture: 'review' }] })
      }
      return jsonResponse({ items: [] })
    }) as unknown as typeof fetch
    const client = new GitGovReadOnlyClient({
      config: { apiUrl: 'https://gitgov-api.example', orgName: 'yohandry10' },
      apiKey: 'readonly-secret',
      fetchImpl,
    })

    await client.listDeploymentGates('yohandry10/Git-Gov', 'main')
    await client.listChangeRisks('yohandry10/Git-Gov', 'main')
    await client.getExecutiveRepository('yohandry10/Git-Gov')

    expect(calls.map((call) => new URL(call.url).pathname)).toEqual([...READ_ONLY_ENDPOINTS])
    expect(calls.every((call) => call.init?.method === 'GET')).toBe(true)
    expect(calls.every((call) => String(call.init?.headers).includes('readonly-secret'))).toBe(false)
    expect(calls.map((call) => (call.init?.headers as Record<string, string>).Authorization)).toEqual([
      'Bearer readonly-secret',
      'Bearer readonly-secret',
      'Bearer readonly-secret',
    ])
  })

  it('sanitizes permission errors without leaking the API key', async () => {
    const fetchImpl = vi.fn(async () => jsonResponse({ error: 'token readonly-secret rejected' }, 403)) as unknown as typeof fetch
    const client = new GitGovReadOnlyClient({
      config: { apiUrl: 'https://gitgov-api.example', orgName: 'yohandry10' },
      apiKey: 'readonly-secret',
      fetchImpl,
    })

    await expect(client.listDeploymentGates('yohandry10/Git-Gov', 'main')).rejects.toThrow(
      'GitGov denied this read-only request. Check org scope and key permissions.',
    )
    await expect(client.listDeploymentGates('yohandry10/Git-Gov', 'main')).rejects.not.toThrow('readonly-secret')
  })
})
