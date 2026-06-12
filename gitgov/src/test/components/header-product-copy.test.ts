import headerSource from '../../components/layout/Header.tsx?raw'

describe('Header product behavior', () => {
  it('does not expose a global refresh that can force the Control Plane auth screen', () => {
    expect(headerSource).not.toContain('Actualizar')
    expect(headerSource).not.toContain('handleRefresh')
    expect(headerSource).not.toContain('RefreshCw')
  })

  it('keeps Control Plane connection checks in background mode from the header', () => {
    expect(headerSource).toContain('checkConnection({ background: true })')
    expect(headerSource).not.toContain('checkConnection())')
  })
})
