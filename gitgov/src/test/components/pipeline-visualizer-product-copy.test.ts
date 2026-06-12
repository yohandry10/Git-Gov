import pipelineVisualizerSource from '../../components/cli/PipelineVisualizer.tsx?raw'

describe('PipelineVisualizer product copy', () => {
  it('keeps Workspace focused on the local execution step', () => {
    expect(pipelineVisualizerSource).toContain('Next local step')
  })

  it('does not duplicate the global Next Action inside Policy Signals', () => {
    const gateItemsStart = pipelineVisualizerSource.indexOf('const gateItems')
    const gateItemsEnd = pipelineVisualizerSource.indexOf('return (', gateItemsStart)
    const gateItemsSource = pipelineVisualizerSource.slice(gateItemsStart, gateItemsEnd)

    expect(gateItemsSource).toContain("label: 'Traceability'")
    expect(gateItemsSource).toContain("label: 'Review Gate'")
    expect(gateItemsSource).toContain("label: 'CI Gate'")
    expect(gateItemsSource).not.toContain("label: 'Next Action'")
  })

  it('labels policy signal impact without implying terminal blocking', () => {
    expect(pipelineVisualizerSource).toContain('Policy Signals')
    expect(pipelineVisualizerSource).toContain('Local terminal')
    expect(pipelineVisualizerSource).toContain('Blocks merge')
    expect(pipelineVisualizerSource).toContain('Advisory')
    expect(pipelineVisualizerSource).toContain('Waiting')
    expect(pipelineVisualizerSource).not.toContain('Gates / Blockers')
  })
})
