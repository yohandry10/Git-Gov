import pipelineVisualizerSource from '../../components/cli/PipelineVisualizer.tsx?raw'

describe('PipelineVisualizer product copy', () => {
  it('keeps Workspace focused on the local execution step', () => {
    expect(pipelineVisualizerSource).toContain('Next local step')
  })

  it('does not duplicate the global Next Action inside Gates / Blockers', () => {
    const gateItemsStart = pipelineVisualizerSource.indexOf('const gateItems')
    const gateItemsEnd = pipelineVisualizerSource.indexOf('return (', gateItemsStart)
    const gateItemsSource = pipelineVisualizerSource.slice(gateItemsStart, gateItemsEnd)

    expect(gateItemsSource).toContain("label: 'Traceability'")
    expect(gateItemsSource).toContain("label: 'Review Gate'")
    expect(gateItemsSource).toContain("label: 'CI Gate'")
    expect(gateItemsSource).not.toContain("label: 'Next Action'")
  })
})
