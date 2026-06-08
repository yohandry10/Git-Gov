import settingsSource from '../../pages/SettingsPage.tsx?raw'

describe('Settings navigation', () => {
  it('organizes Settings into stable domain tabs', () => {
    expect(settingsSource).toContain("id: 'preferences'")
    expect(settingsSource).toContain("id: 'organization'")
    expect(settingsSource).toContain("id: 'account'")
    expect(settingsSource).toContain("id: 'repository'")
    expect(settingsSource).toContain("id: 'connection'")
    expect(settingsSource).not.toContain("id: 'updates'")
  })

  it('orders Account next to Organization and keeps System last after Repository', () => {
    const tabsExpression = settingsSource.slice(
      settingsSource.indexOf('const SETTINGS_TABS'),
      settingsSource.indexOf('function readSettingsTabFromHash'),
    )

    expect(tabsExpression.indexOf("id: 'organization'")).toBeLessThan(tabsExpression.indexOf("id: 'account'"))
    expect(tabsExpression.indexOf("id: 'account'")).toBeLessThan(tabsExpression.indexOf("id: 'repository'"))
    expect(tabsExpression.indexOf("id: 'repository'")).toBeLessThan(tabsExpression.indexOf("id: 'connection'"))
  })

  it('preserves Control Plane deep-link compatibility inside Settings', () => {
    expect(settingsSource).toContain("if (hash === 'control-plane') return 'connection'")
    expect(settingsSource).toContain("if (hash === 'updates') return 'connection'")
    expect(settingsSource).toContain("const hash = tab === 'connection' ? 'control-plane' : tab")
  })

  it('keeps existing Settings surfaces assigned to tabs instead of deleting them', () => {
    expect(settingsSource).toContain('<LanguagePreferenceSelector />')
    expect(settingsSource).toContain('<ServerConfigPanel />')
    expect(settingsSource).toContain("activeTab === 'connection' ? '' : 'hidden'} rounded-2xl border border-surface-700/30 bg-surface-800/40 p-6`")
    expect(settingsSource).toContain('<AdminOnboardingPanel />')
    expect(settingsSource).toContain('<TeamManagementPanel />')
    expect(settingsSource).toContain('<ApiKeyManagerWidget />')
    expect(settingsSource).toContain('<GovernanceRulesPanel repoFullName={repoFullName} />')
  })

  it('uses i18n keys for Settings tab labels and descriptions', () => {
    expect(settingsSource).toContain("labelKey: 'settings.tabs.preferences.label'")
    expect(settingsSource).toContain("descriptionKey: 'settings.tabs.connection.description'")
    expect(settingsSource).toContain('{t(item.labelKey)}')
    expect(settingsSource).not.toContain("label: 'Preferences'")
    expect(settingsSource).not.toContain("label: 'Connection'")
  })

  it('keeps Organization as a full-width flow to avoid an empty right column', () => {
    const layoutExpression = settingsSource.slice(
      settingsSource.indexOf('const settingsContentClass ='),
      settingsSource.indexOf('const controlPlaneEndpoint ='),
    )

    expect(layoutExpression).not.toContain("activeTab === 'organization'")
    expect(layoutExpression).toContain("activeTab === 'repository' && config")
  })
})
