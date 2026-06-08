import { describe, expect, it } from 'vitest'
import {
  buildGitIdentityEvidenceLines,
  evaluateGitIdentity,
  formatGitIdentityBlockToast,
  formatGitIdentityValue,
} from '@/lib/gitIdentityPolicy'

describe('gitIdentityPolicy', () => {
  it('accepts a Git identity aligned with the authenticated GitHub login', () => {
    const finding = evaluateGitIdentity(
      {
        name: 'yohandry10',
        email: 'yohandry10@users.noreply.github.com',
        name_scope: 'local',
        email_scope: 'local',
      },
      { login: 'yohandry10', name: null },
    )

    expect(finding).toBeNull()
  })

  it('accepts GitHub numbered noreply email as aligned with the authenticated login', () => {
    const finding = evaluateGitIdentity(
      {
        name: 'Yohandry Chirinos',
        email: '123456+yohandry10@users.noreply.github.com',
        name_scope: 'local',
        email_scope: 'local',
      },
      { login: 'yohandry10', name: null },
    )

    expect(finding).toBeNull()
  })

  it('flags an incomplete effective Git identity', () => {
    const finding = evaluateGitIdentity(
      { name: 'yohandry10', email: null, name_scope: 'local', email_scope: null },
      { login: 'yohandry10' },
    )

    expect(finding?.reason).toBe('incomplete')
    expect(finding?.suggestedEmail).toBe('yohandry10@users.noreply.github.com')
  })

  it('does not use placeholder GitHub names as suggested Git identity', () => {
    const finding = evaluateGitIdentity(
      {
        name: 'yohandrychirinos1',
        email: 'yohandrychirinos1@gmail.com',
        name_scope: 'global',
        email_scope: 'global',
      },
      { login: 'yohandry10', name: 'Unknown', email: null },
    )

    expect(finding?.reason).toBe('not_provably_aligned')
    expect(finding?.suggestedName).toBe('yohandry10')
    expect(finding?.suggestedEmail).toBe('yohandry10@users.noreply.github.com')
  })

  it('flags an identity that GitGov cannot prove belongs to the authenticated GitHub user', () => {
    const finding = evaluateGitIdentity(
      {
        name: 'yohandrychirinos1',
        email: 'yohandrychirinos1@gmail.com',
        name_scope: 'global',
        email_scope: 'global',
      },
      { login: 'yohandry10', name: null, email: null },
    )

    expect(finding?.reason).toBe('not_provably_aligned')
    expect(formatGitIdentityBlockToast('Commit', finding!, 'yohandry10')).toContain('@yohandry10')
    expect(formatGitIdentityBlockToast('Commit', finding!, 'yohandry10')).not.toContain('cuenta GitGov')
  })

  it('does not treat partial substring matches as provable identity alignment', () => {
    const finding = evaluateGitIdentity(
      {
        name: 'not-yohandry10-other',
        email: 'audit-yohandry10@example.com',
        name_scope: 'local',
        email_scope: 'local',
      },
      { login: 'yohandry10', name: null, email: null },
    )

    expect(finding?.reason).toBe('not_provably_aligned')
  })

  it('formats read-only evidence lines with git config commands and observed scopes', () => {
    const identity = {
      name: 'yohandrychirinos1',
      email: 'yohandrychirinos1@gmail.com',
      name_scope: 'global' as const,
      email_scope: 'global' as const,
      name_source: 'C:/Users/PC/.gitconfig',
      email_source: 'C:/Users/PC/.gitconfig',
    }
    const user = { login: 'yohandry10' }
    const finding = evaluateGitIdentity(identity, user)
    const lines = buildGitIdentityEvidenceLines(identity, user, finding)

    expect(lines.map((line) => line.text)).toContain('$ git config --get user.name')
    expect(lines.map((line) => line.text)).toContain('$ git config --get user.email')
    expect(lines.every((line) => line.auditable === false)).toBe(true)
    expect(formatGitIdentityValue(identity.email, identity.email_scope, identity.email_source)).toContain('global de Git')
    expect(lines.some((line) => line.text.includes('Authenticated GitHub user: @yohandry10'))).toBe(true)
  })
})
