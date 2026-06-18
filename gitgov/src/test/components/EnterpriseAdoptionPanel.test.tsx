import { render, screen, waitFor, within } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { EnterpriseAdoptionPanel } from '@/components/control_plane/EnterpriseAdoptionPanel'
import { DEFAULT_ENTERPRISE_ADOPTION_PROFILE, type EnterpriseAdoptionProfile } from '@/components/control_plane/dashboard-helpers'

const storeMock = vi.hoisted(() => ({
  state: {} as Record<string, unknown>,
}))

vi.mock('@/store/useControlPlaneStore', () => ({
  useControlPlaneStore: (selector: (state: Record<string, unknown>) => unknown) => selector(storeMock.state),
}))

function panelState(profile: EnterpriseAdoptionProfile) {
  return {
    selectedOrgName: 'exampleco',
    serverStats: {
      github_events: { total: 0, by_type: {} },
      pipeline: { total_7d: 0, success_7d: 0 },
      active_repos: 0,
    },
    ticketCoverage: {
      commits_with_ticket: 0,
      coverage_percentage: 0,
    },
    jenkinsCorrelations: [],
    enterpriseAdoptionProfile: profile,
    enterpriseAdoptionProfileUpdatedAt: null,
    isEnterpriseAdoptionProfileLoading: false,
    isEnterpriseAdoptionProfileSaving: false,
    enterpriseAdoptionProfileError: null,
    enterpriseOnboardingChecklistTracking: null,
    enterpriseOnboardingChecklistTrackingUpdatedAt: null,
    isEnterpriseOnboardingChecklistTrackingLoading: false,
    isEnterpriseOnboardingChecklistTrackingSaving: false,
    enterpriseOnboardingChecklistTrackingError: null,
    loadEnterpriseAdoptionProfile: vi.fn(),
    saveEnterpriseAdoptionProfile: vi.fn(),
    loadEnterpriseOnboardingChecklistTracking: vi.fn(),
    saveEnterpriseOnboardingChecklistTracking: vi.fn(),
  }
}

describe('EnterpriseAdoptionPanel provider setup guidance', () => {
  it('renders manual connect, retry, and skipped guidance without provider mutation controls', async () => {
    storeMock.state = panelState({
      ...DEFAULT_ENTERPRISE_ADOPTION_PROFILE,
      jira_project_key: '',
      providers: ['jira', 'vercel'],
    })

    render(
      <MemoryRouter>
        <EnterpriseAdoptionPanel />
      </MemoryRouter>,
    )

    const guidance = await screen.findByRole('region', { name: 'Provider setup guidance' })
    await waitFor(() => {
      expect(within(guidance).getByText('Next: Connect')).toBeInTheDocument()
    })

    expect(within(guidance).getByText('0/2 selected ready, 4 skipped')).toBeInTheDocument()
    expect(within(guidance).getByText('Jira')).toBeInTheDocument()
    expect(within(guidance).getAllByText('Set the Jira project key for traceability validation.')).toHaveLength(2)
    expect(within(guidance).getByText('Vercel')).toBeInTheDocument()
    expect(within(guidance).getAllByText('Retry').length).toBeGreaterThanOrEqual(1)
    expect(within(guidance).getAllByText('Skipped')).toHaveLength(4)
    expect(within(guidance).getByRole('link', { name: 'Open Settings for Jira' })).toHaveAttribute('href', '/settings#control-plane')
    expect(within(guidance).getByRole('link', { name: 'Open Evidence for Vercel' })).toHaveAttribute('href', '/governance/evidence')
    expect(within(guidance).getAllByRole('link', { name: /Review profile for/i })).toHaveLength(4)
    expect(within(guidance).queryByRole('button', { name: /connect|retry|skipped/i })).not.toBeInTheDocument()
  })
})
