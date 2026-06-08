import { render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { Sidebar } from '@/components/layout/Sidebar'
import { useAuthStore } from '@/store/useAuthStore'
import { setAppLanguage } from '@/lib/i18n'
import routerSource from '../../router.tsx?raw'

describe('Governance navigation', () => {
  beforeEach(async () => {
    await setAppLanguage('en')
    useAuthStore.setState({
      user: {
        login: 'operator',
        name: 'Operator',
        avatar_url: '/logo.png',
        is_admin: true,
      },
      authStep: 'authenticated',
      isLoading: false,
      isPinEnabled: false,
      pinUnlocked: true,
    })
  })

  it('renders Governance as a primary sidebar destination', () => {
    render(
      <MemoryRouter>
        <Sidebar />
      </MemoryRouter>,
    )

    expect(screen.getByLabelText('Governance')).toHaveAttribute('href', '/governance')
    expect(screen.queryByLabelText('Control Plane')).not.toBeInTheDocument()
  })

  it('updates primary navigation labels when Spanish is active', async () => {
    await setAppLanguage('es')

    render(
      <MemoryRouter>
        <Sidebar />
      </MemoryRouter>,
    )

    expect(screen.getByLabelText('Gobernanza')).toHaveAttribute('href', '/governance')
    expect(screen.getByLabelText('Configuración')).toHaveAttribute('href', '/settings')
  })

  it('registers Governance routes for every domain section', () => {
    expect(routerSource).toContain("path: '/governance'")
    expect(routerSource).toContain("path: '/governance/:section'")
    expect(routerSource).toContain('<GovernancePage />')
    expect(routerSource).toContain('to="/settings#control-plane"')
  })

  it('keeps Evidence as the first Governance section without a generic Dashboard tab', async () => {
    const governanceSource = await import('../../pages/GovernancePage.tsx?raw')
    const source = governanceSource.default as string
    const evidenceIndex = source.indexOf("id: 'evidence'")
    const policyIndex = source.indexOf("id: 'policy'")

    expect(source).not.toContain("id: 'dashboard'")
    expect(evidenceIndex).toBeGreaterThanOrEqual(0)
    expect(policyIndex).toBeGreaterThan(evidenceIndex)
    expect(source).toContain("labelKey: 'governance.sections.evidence.label'")
    expect(source).toContain('{t(item.labelKey)}')
  })
})
