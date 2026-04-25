import { render, screen } from '@testing-library/react'
import { EventBreakdownGrid } from '@/components/control_plane/EventBreakdownGrid'
import { GitHubEvidenceTrendWidget } from '@/components/control_plane/GitHubEvidenceTrendWidget'
import {
  appendGitHubEvidenceTrendPoint,
  buildAuditExportPackage,
  buildGitHubEvidenceSummary,
  buildGitHubEvidenceTrendPoint,
} from '@/components/control_plane/dashboard-helpers'

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

describe('buildAuditExportPackage', () => {
  it('packages raw export data with executive GitHub evidence context', () => {
    const pkg = buildAuditExportPackage(
      {
        id: 'export-1',
        export_type: 'events',
        record_count: 2,
        content_hash: 'sha256:abc',
        created_at: 123,
        data: { events: [{ event_type: 'commit' }] },
      },
      {
        pull_request: 1,
        pull_request_review: 1,
        issue_comment: 1,
        status: 1,
      },
      '2026-04-25T00:00:00.000Z',
    )

    expect(pkg.export_id).toBe('export-1')
    expect(pkg.source_content_hash).toBe('sha256:abc')
    expect(pkg.packaged_at).toBe('2026-04-25T00:00:00.000Z')
    expect(pkg.executive_summary.github_evidence.executiveStatus).toBe('Completo')
    expect(pkg.executive_summary.github_evidence.activeSignals).toBe(4)
    expect(pkg.data).toEqual({ events: [{ event_type: 'commit' }] })
  })
})

describe('GitHub evidence trend helpers', () => {
  it('builds and appends trend points without duplicating unchanged adjacent status', () => {
    const partial = buildGitHubEvidenceTrendPoint(
      buildGitHubEvidenceSummary({ pull_request: 1, check_run: 1 }),
      '2026-04-25T20:00:00.000Z',
    )
    const repeatedPartial = buildGitHubEvidenceTrendPoint(
      buildGitHubEvidenceSummary({ pull_request: 2, check_suite: 1 }),
      '2026-04-25T20:05:00.000Z',
    )
    const complete = buildGitHubEvidenceTrendPoint(
      buildGitHubEvidenceSummary({
        pull_request: 2,
        pull_request_review: 1,
        issue_comment: 1,
        status: 1,
      }),
      '2026-04-25T20:10:00.000Z',
    )

    const trend = appendGitHubEvidenceTrendPoint(
      appendGitHubEvidenceTrendPoint([partial], repeatedPartial),
      complete,
    )

    expect(trend).toHaveLength(2)
    expect(trend[0].capturedAt).toBe('2026-04-25T20:05:00.000Z')
    expect(trend[1].executiveStatus).toBe('Completo')
    expect(trend[1].activeSignals).toBe(4)
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

describe('GitHubEvidenceTrendWidget', () => {
  it('renders local trend summary and missing signals', () => {
    render(
      <GitHubEvidenceTrendWidget
        onCapture={() => undefined}
        points={[
          {
            capturedAt: '2026-04-25T20:00:00.000Z',
            activeSignals: 1,
            totalSignals: 4,
            executiveStatus: 'Parcial',
            missingSignals: ['Reviews', 'Comentarios PR', 'Checks/status'],
          },
          {
            capturedAt: '2026-04-25T20:10:00.000Z',
            activeSignals: 2,
            totalSignals: 4,
            executiveStatus: 'Parcial',
            missingSignals: ['Reviews', 'Comentarios PR'],
          },
        ]}
      />,
    )

    expect(screen.getByText('Trend evidencia GitHub')).toBeInTheDocument()
    expect(screen.getByText('2/4')).toBeInTheDocument()
    expect(screen.getByText('+1')).toBeInTheDocument()
    expect(screen.getByText('Faltan: Reviews, Comentarios PR')).toBeInTheDocument()
  })
})
