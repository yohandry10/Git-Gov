import {
  DEFAULT_ENTERPRISE_ADOPTION_PROFILE,
  buildEnterpriseAdoptionPack,
  buildEnterpriseOnboardingGuide,
  buildEnterpriseOnboardingReadinessReport,
  buildEnterpriseOnboardingRemediationPlan,
  buildEnterpriseProviderHealth,
  validateEnterpriseAdoptionProfile,
  type EnterpriseAdoptionProfile,
  type EnterpriseProviderHealthCheck,
} from '@/components/control_plane/dashboard-helpers'
import {
  buildActionCenterGuidance,
  type ActionCenterBuildInput,
} from '@/components/action_center/action-center-helpers'

const GENERATED_AT = '2026-06-07T00:00:00.000Z'

function makeInput(overrides: Partial<ActionCenterBuildInput> = {}): ActionCenterBuildInput {
  const profile: EnterpriseAdoptionProfile = overrides.profile ?? DEFAULT_ENTERPRISE_ADOPTION_PROFILE
  const pack = overrides.pack ?? buildEnterpriseAdoptionPack(profile, GENERATED_AT)
  const validation = overrides.validation ?? validateEnterpriseAdoptionProfile(profile)
  const providerHealth = overrides.providerHealth ?? buildEnterpriseProviderHealth(profile, {
    githubEventsTotal: 10,
    jiraCommitsWithTicket: 10,
    jiraCoveragePercentage: 100,
    pipelineRuns7d: 8,
    pipelineSuccess7d: 8,
    sonarRuns: 3,
    sonarSuccessful: 3,
    activeRepos: 1,
  }, pack)
  const readiness = overrides.readiness ?? buildEnterpriseOnboardingReadinessReport(
    profile,
    providerHealth,
    null,
    GENERATED_AT,
  )
  const remediationPlan = overrides.remediationPlan ?? buildEnterpriseOnboardingRemediationPlan(
    readiness,
    pack,
    GENERATED_AT,
  )
  const guide = overrides.guide ?? buildEnterpriseOnboardingGuide(readiness, remediationPlan, GENERATED_AT)

  return {
    goal: 'quick-onboarding',
    lens: 'founder',
    isConnected: true,
    userRole: 'Admin',
    profile,
    pack,
    validation,
    providerHealth,
    readiness,
    remediationPlan,
    guide,
    pipeline: { total_7d: 10, success_7d: 10, failure_7d: 0 },
    ticketCoverage: {
      total_commits: 20,
      commits_with_ticket: 20,
      coverage_percentage: 100,
      commits_without_ticket: [],
      tickets_without_commits: [],
    },
    evidencePacket: {
      subject: 'KAN-69',
      content_hash: 'a'.repeat(64),
      completeness: {
        ticket_found: true,
        commits: 1,
        pull_requests: 1,
        pipelines: 1,
        quality_gates: 1,
        missing: [],
      },
    },
    releaseApprovalsTotal: 1,
    ...overrides,
  }
}

describe('action-center-helpers', () => {
  it('prioritizes the adoption profile when profile validation fails', () => {
    const profile: EnterpriseAdoptionProfile = {
      ...DEFAULT_ENTERPRISE_ADOPTION_PROFILE,
      repository_full_name: '',
    }

    const guidance = buildActionCenterGuidance(makeInput({ profile }))

    expect(guidance.primary.id).toBe('complete-profile')
    expect(guidance.primary.status).toBe('blocked')
    expect(guidance.primary.permission.canAct).toBe(true)
  })

  it('surfaces provider configuration before evidence collection', () => {
    const providerHealth: EnterpriseProviderHealthCheck[] = [
      {
        provider: 'github',
        label: 'GitHub',
        status: 'needs-config',
        evidence: 'No telemetry variables are present',
        next_step: 'Add required GitGov variable and secret names.',
      },
      {
        provider: 'jira',
        label: 'Jira',
        status: 'needs-evidence',
        evidence: 'No ticket correlation observed',
        next_step: 'Run Jira correlation.',
      },
    ]

    const guidance = buildActionCenterGuidance(makeInput({ providerHealth }))

    expect(guidance.primary.id).toBe('complete-provider-config')
    expect(guidance.primary.title).toContain('provider configuration')
  })

  it('keeps admin-only actions advisory for non-admin users', () => {
    const providerHealth: EnterpriseProviderHealthCheck[] = [
      {
        provider: 'github',
        label: 'GitHub',
        status: 'needs-config',
        evidence: 'No telemetry variables are present',
        next_step: 'Add required GitGov variable and secret names.',
      },
    ]

    const guidance = buildActionCenterGuidance(makeInput({
      providerHealth,
      userRole: 'Developer',
    }))

    expect(guidance.primary.id).toBe('complete-provider-config')
    expect(guidance.primary.permission.canAct).toBe(false)
    expect(guidance.primary.permission.requiredRole).toBe('Admin')
  })

  it('prioritizes pipeline health for release prep when recent CI is weak', () => {
    const guidance = buildActionCenterGuidance(makeInput({
      goal: 'prepare-release',
      pipeline: { total_7d: 6, success_7d: 4, failure_7d: 2 },
    }))

    expect(guidance.primary.id).toBe('review-pipeline-health')
    expect(guidance.primary.evidence.map((line) => line.label)).toContain('Success rate')
  })

  it('prioritizes Jira traceability when pipeline is healthy but coverage is low', () => {
    const guidance = buildActionCenterGuidance(makeInput({
      goal: 'prepare-release',
      pipeline: { total_7d: 10, success_7d: 10, failure_7d: 0 },
      ticketCoverage: {
        total_commits: 10,
        commits_with_ticket: 7,
        coverage_percentage: 70,
        commits_without_ticket: [{ sha: 'abc1234' }],
        tickets_without_commits: [],
      },
    }))

    expect(guidance.primary.id).toBe('repair-traceability-coverage')
    expect(guidance.primary.confidence).toBe('high')
  })

  it('keeps release prep conservative when Jira traceability is not loaded', () => {
    const guidance = buildActionCenterGuidance(makeInput({
      goal: 'prepare-release',
      pipeline: { total_7d: 10, success_7d: 10, failure_7d: 0 },
      ticketCoverage: null,
      evidencePacket: {
        subject: 'KAN-69',
        content_hash: 'c'.repeat(64),
        completeness: {
          ticket_found: true,
          commits: 2,
          pull_requests: 1,
          pipelines: 1,
          quality_gates: 1,
          missing: [],
        },
      },
    }))

    expect(guidance.primary.id).toBe('repair-traceability-coverage')
    expect(guidance.primary.status).toBe('needs-action')
    expect(guidance.primary.confidence).toBe('low')
  })

  it('does not treat an empty traceability window as release-ready', () => {
    const guidance = buildActionCenterGuidance(makeInput({
      goal: 'prepare-release',
      pipeline: { total_7d: 10, success_7d: 10, failure_7d: 0 },
      ticketCoverage: {
        total_commits: 0,
        commits_with_ticket: 0,
        coverage_percentage: 100,
        commits_without_ticket: [],
        tickets_without_commits: [],
      },
    }))

    expect(guidance.primary.id).toBe('repair-traceability-coverage')
    expect(guidance.primary.status).toBe('needs-action')
  })

  it('marks a complete current Evidence Packet as ready for export review', () => {
    const guidance = buildActionCenterGuidance(makeInput({
      goal: 'export-evidence',
      evidencePacket: {
        subject: 'KAN-69',
        content_hash: 'b'.repeat(64),
        completeness: {
          ticket_found: true,
          commits: 2,
          pull_requests: 1,
          pipelines: 1,
          quality_gates: 1,
          missing: [],
        },
      },
    }))

    expect(guidance.primary.id).toBe('review-current-evidence-packet')
    expect(guidance.primary.status).toBe('ready')
  })
})
