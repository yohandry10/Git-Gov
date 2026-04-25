import { render, screen } from '@testing-library/react'
import { EventBreakdownGrid } from '@/components/control_plane/EventBreakdownGrid'
import { buildGitHubEvidenceSummary } from '@/components/control_plane/dashboard-helpers'

const baseProps = {
  clientByStatus: {},
  commitsWithoutTicket: [],
  ticketsWithoutCommits: [],
  totalCommitsWithoutTicket: 0,
  totalTicketsWithoutCommits: 0,
}

describe('buildGitHubEvidenceSummary', () => {
  it('classifies complete GitHub evidence coverage', () => {
    const summary = buildGitHubEvidenceSummary({
      pull_request: 4,
      pull_request_review: 2,
      pull_request_review_comment: 1,
      check_run: 3,
    })

    expect(summary.executiveStatus).toBe('Completo')
    expect(summary.activeSignals).toBe(4)
    expect(summary.totalSignals).toBe(4)
    expect(summary.missingSignals).toEqual([])
  })

  it('classifies partial coverage and lists missing executive signals', () => {
    const summary = buildGitHubEvidenceSummary({
      pull_request: 4,
      status: 1,
    })

    expect(summary.executiveStatus).toBe('Parcial')
    expect(summary.activeSignals).toBe(2)
    expect(summary.missingSignals).toEqual(['Reviews', 'Comentarios PR'])
  })

  it('classifies empty coverage as no evidence', () => {
    const summary = buildGitHubEvidenceSummary({})

    expect(summary.executiveStatus).toBe('Sin evidencia')
    expect(summary.activeSignals).toBe(0)
    expect(summary.missingSignals).toEqual([
      'PR lifecycle',
      'Reviews',
      'Comentarios PR',
      'Checks/status',
    ])
  })
})

describe('EventBreakdownGrid', () => {
  it('renders executive GitHub evidence coverage', () => {
    render(
      <EventBreakdownGrid
        {...baseProps}
        githubByType={{
          pull_request: 4,
          pull_request_review: 2,
          issue_comment: 1,
          check_suite: 3,
        }}
      />,
    )

    expect(screen.getByText('Cobertura ejecutiva')).toBeInTheDocument()
    expect(screen.getByText('Completo')).toBeInTheDocument()
    expect(screen.getByText('4/4 señales')).toBeInTheDocument()
    expect(screen.getByText('PR, reviews, comentarios y checks activos')).toBeInTheDocument()
  })

  it('renders missing GitHub evidence signals', () => {
    render(
      <EventBreakdownGrid
        {...baseProps}
        githubByType={{
          pull_request: 1,
          check_run: 1,
        }}
      />,
    )

    expect(screen.getByText('Parcial')).toBeInTheDocument()
    expect(screen.getByText('2/4 señales')).toBeInTheDocument()
    expect(screen.getByText('Falta: Reviews, Comentarios PR')).toBeInTheDocument()
  })
})
