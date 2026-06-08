import helpSource from '../../pages/HelpPage.tsx?raw'

describe('Help page layout', () => {
  it('uses the full app workspace instead of a narrow centered document column', () => {
    expect(helpSource).not.toContain('max-w-2xl mx-auto')
    expect(helpSource).toContain("xl:grid-cols-[280px_minmax(0,1fr)]")
    expect(helpSource).toContain('2xl:grid-cols-6')
    expect(helpSource).toContain("index < 2 ? '2xl:col-span-3' : '2xl:col-span-2'")
  })

  it('keeps FAQ content organized by category instead of removing sections', () => {
    expect(helpSource).toContain('Qué GitGov NO hace')
    expect(helpSource).toContain('Datos y seguridad')
    expect(helpSource).toContain('App de escritorio')
    expect(helpSource).toContain('Control Plane')
    expect(helpSource).toContain('Cumplimiento')
  })

  it('uses the canonical GitGov Cloud URL instead of the old Vercel app URL', () => {
    expect(helpSource).toContain('https://gitgov.cloud/docs/faq')
    expect(helpSource).toContain('https://gitgov.cloud/contact')
    expect(helpSource).not.toContain('git-gov.vercel.app')
  })
})
