import { fireEvent, render, screen } from '@testing-library/react'
import { ReleaseGovernanceEnvironmentPolicyPanel } from '@/components/control_plane/ReleaseGovernanceEnvironmentPolicyPanel'
import type { EnterpriseReleaseGovernancePolicy } from '@/components/control_plane/dashboard-helpers'

const policy: EnterpriseReleaseGovernancePolicy = {
  mode: 'record-only',
  environment: 'staging',
  approval_required: false,
  enforcement: 'disabled',
  quorum: { enabled: false, rules: [] },
  environment_overrides: [
    {
      mode: 'approval-required',
      environment: 'production',
      approval_required: true,
      enforcement: 'blocking',
      quorum: { enabled: false, rules: [] },
    },
  ],
}

function renderPanel(overrides = {}) {
  const handlers = {
    onBaseModeChange: vi.fn(),
    onBaseEnvironmentChange: vi.fn(),
    onAddOverride: vi.fn(),
    onOverrideEnvironmentChange: vi.fn(),
    onOverrideModeChange: vi.fn(),
    onRemoveOverride: vi.fn(),
    selectedClass: (selected: boolean) => (selected ? 'selected' : 'unselected'),
    ...overrides,
  }

  render(
    <ReleaseGovernanceEnvironmentPolicyPanel
      policy={policy}
      badgeVariant="neutral"
      {...handlers}
    />,
  )

  return handlers
}

describe('ReleaseGovernanceEnvironmentPolicyPanel', () => {
  it('renders base and production override policies as an environment matrix', () => {
    renderPanel()

    expect(screen.getByText('Environment policy matrix')).toBeInTheDocument()
    expect(screen.getByText('staging')).toBeInTheDocument()
    expect(screen.getByText('Base policy · Record only')).toBeInTheDocument()
    expect(screen.getByText('production')).toBeInTheDocument()
    expect(screen.getByText('Environment override · Approval required')).toBeInTheDocument()
    expect(screen.getAllByText('blocking')).toHaveLength(1)
  })

  it('emits concrete edit intents for environment overrides', () => {
    const handlers = renderPanel()

    fireEvent.change(screen.getByLabelText('Release governance override environment 1'), {
      target: { value: 'production-us' },
    })
    fireEvent.click(screen.getAllByRole('button', { name: 'Quorum required' })[1])
    fireEvent.click(screen.getByTitle('Remove environment override'))

    expect(handlers.onOverrideEnvironmentChange).toHaveBeenCalledWith(0, 'production-us')
    expect(handlers.onOverrideModeChange).toHaveBeenCalledWith(0, 'quorum-required')
    expect(handlers.onRemoveOverride).toHaveBeenCalledWith(0)
  })
})
