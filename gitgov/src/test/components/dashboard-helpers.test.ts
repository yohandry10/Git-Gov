import {
  DEFAULT_ENTERPRISE_ADOPTION_PROFILE,
  buildEnterpriseAdoptionPack,
  buildEnterpriseWorkflowTemplatePack,
  buildEnterpriseWorkflowTemplatePackFilename,
  buildEnterpriseProviderHealth,
  buildOperationalEvidenceMetrics,
  formatOperationalMetricDuration,
  validateEnterpriseAdoptionProfile,
  type EnterpriseAdoptionProfile,
  type OperationalPipelineEvidence,
} from '@/components/control_plane/dashboard-helpers'

describe('dashboard-helpers operational evidence metrics', () => {
  it('computes average time-to-evidence from commit to pipeline ingestion', () => {
    const correlations: OperationalPipelineEvidence[] = [
      {
        commit_created_at: 1_000,
        pipeline: {
          pipeline_event_id: 'pipe-1',
          pipeline_id: 'jenkins-1',
          job_name: 'gitgov-demo-pipeline',
          status: 'success',
          ingested_at: 11_000,
        },
      },
      {
        commit_created_at: 2_000,
        pipeline: {
          pipeline_event_id: 'pipe-2',
          pipeline_id: 'jenkins-2',
          job_name: 'gitgov-demo-pipeline',
          status: 'success',
          ingested_at: 22_000,
        },
      },
      {
        commit_created_at: 30_000,
        pipeline: {
          pipeline_event_id: 'pipe-3',
          pipeline_id: 'jenkins-3',
          job_name: 'gitgov-demo-pipeline',
          status: 'success',
          ingested_at: 20_000,
        },
      },
    ]

    const metrics = buildOperationalEvidenceMetrics(correlations)

    expect(metrics.timeToEvidenceSamples).toBe(2)
    expect(metrics.timeToEvidenceMs).toBe(15_000)
  })

  it('deduplicates pipeline evidence before calculating samples', () => {
    const correlations: OperationalPipelineEvidence[] = [
      {
        commit_created_at: 1_000,
        pipeline: {
          pipeline_event_id: 'pipe-1',
          pipeline_id: 'jenkins-1',
          job_name: 'gitgov-demo-pipeline',
          status: 'success',
          ingested_at: 11_000,
        },
      },
      {
        commit_created_at: 1_000,
        pipeline: {
          pipeline_event_id: 'pipe-1',
          pipeline_id: 'jenkins-1',
          job_name: 'gitgov-demo-pipeline',
          status: 'success',
          ingested_at: 11_000,
        },
      },
    ]

    const metrics = buildOperationalEvidenceMetrics(correlations)

    expect(metrics.timeToEvidenceSamples).toBe(1)
    expect(metrics.timeToEvidenceMs).toBe(10_000)
  })

  it('computes MTTR from recoverable pipeline failure to next success for the same job', () => {
    const correlations: OperationalPipelineEvidence[] = [
      {
        commit_created_at: 1_000,
        pipeline: {
          pipeline_event_id: 'pipe-1',
          pipeline_id: 'jenkins-1',
          job_name: 'gitgov-demo-pipeline',
          status: 'failure',
          ingested_at: 10_000,
        },
      },
      {
        commit_created_at: 1_000,
        pipeline: {
          pipeline_event_id: 'pipe-2',
          pipeline_id: 'jenkins-2',
          job_name: 'gitgov-demo-pipeline',
          status: 'success',
          ingested_at: 70_000,
        },
      },
      {
        commit_created_at: 1_000,
        pipeline: {
          pipeline_event_id: 'pipe-3',
          pipeline_id: 'jenkins-3',
          job_name: 'sonar-governance',
          status: 'error',
          ingested_at: 100_000,
        },
      },
      {
        commit_created_at: 1_000,
        pipeline: {
          pipeline_event_id: 'pipe-4',
          pipeline_id: 'jenkins-4',
          job_name: 'sonar-governance',
          status: 'passed',
          ingested_at: 220_000,
        },
      },
    ]

    const metrics = buildOperationalEvidenceMetrics(correlations)

    expect(metrics.mttrSamples).toBe(2)
    expect(metrics.mttrMs).toBe(90_000)
  })

  it('returns empty operational metrics when evidence is insufficient', () => {
    const metrics = buildOperationalEvidenceMetrics([
      {
        commit_created_at: 10_000,
        pipeline: null,
      },
    ])

    expect(metrics).toEqual({
      timeToEvidenceMs: null,
      timeToEvidenceSamples: 0,
      mttrMs: null,
      mttrSamples: 0,
    })
    expect(formatOperationalMetricDuration(metrics.mttrMs, metrics.mttrSamples)).toBe('N/A')
  })
})

describe('dashboard-helpers enterprise adoption pack', () => {
  it('builds the default moderate adoption pack without secret values', () => {
    const pack = buildEnterpriseAdoptionPack(
      DEFAULT_ENTERPRISE_ADOPTION_PROFILE,
      '2026-04-30T00:00:00.000Z',
    )

    expect(pack.workflow_plan).toHaveLength(13)
    expect(pack.variables.map((variable) => variable.name)).toEqual([
      'GITGOV_URL',
      'SONAR_HOST_URL',
      'SONAR_PROJECT_KEY',
    ])
    expect(pack.secrets.map((secret) => secret.name)).toEqual([
      'GITGOV_API_KEY',
      'SONAR_TOKEN',
    ])
    expect(pack.secrets.every((secret) => !Object.prototype.hasOwnProperty.call(secret, 'value'))).toBe(true)
    expect(pack.policy_rules).toContainEqual({
      rule: 'Release readiness target',
      setting: '75',
    })
    expect(validateEnterpriseAdoptionProfile(DEFAULT_ENTERPRISE_ADOPTION_PROFILE).valid).toBe(true)
  })

  it('turns strict preset into PR review and trend enforcement requirements', () => {
    const profile: EnterpriseAdoptionProfile = {
      ...DEFAULT_ENTERPRISE_ADOPTION_PROFILE,
      policy_preset: 'strict',
      modules: DEFAULT_ENTERPRISE_ADOPTION_PROFILE.modules.filter((module) => module !== 'trend-enforcement'),
    }

    const pack = buildEnterpriseAdoptionPack(profile, '2026-04-30T00:00:00.000Z')

    expect(pack.workflow_plan.map((workflow) => workflow.file)).toContain(
      '.github/workflows/product-vulnerability-review-trend-enforcement.yml',
    )
    expect(pack.policy_rules).toContainEqual({
      rule: 'PR review evidence',
      setting: 'required',
    })
    expect(pack.policy_rules).toContainEqual({
      rule: 'Vulnerability trend enforcement',
      setting: 'enabled',
    })
  })

  it('keeps formal release approval visible as an open product gap', () => {
    const profile: EnterpriseAdoptionProfile = {
      ...DEFAULT_ENTERPRISE_ADOPTION_PROFILE,
      modules: [...DEFAULT_ENTERPRISE_ADOPTION_PROFILE.modules, 'formal-approval'],
    }

    const pack = buildEnterpriseAdoptionPack(profile, '2026-04-30T00:00:00.000Z')

    expect(pack.open_product_gaps).toEqual([
      {
        gap: 'Formal release approval',
        detail: 'GitGov has PR review evidence and policy decisions, but a full enterprise release approval model still needs approvers, expiration, risk acceptance, and evidence binding.',
      },
    ])
  })

  it('validates customer adoption profile inputs', () => {
    const profile: EnterpriseAdoptionProfile = {
      ...DEFAULT_ENTERPRISE_ADOPTION_PROFILE,
      repository_full_name: 'missing-owner',
      jira_project_key: 'kan',
    }

    const validation = validateEnterpriseAdoptionProfile(profile)

    expect(validation.valid).toBe(false)
    expect(validation.errors).toContain('Repository must look like owner/repo.')
    expect(validation.errors).toContain('Jira project key should be uppercase letters/numbers, like KAN.')
  })

  it('builds a dashboard workflow template pack without secret values or unresolved tokens', () => {
    const pack = buildEnterpriseWorkflowTemplatePack(
      DEFAULT_ENTERPRISE_ADOPTION_PROFILE,
      '2026-04-30T00:00:00.000Z',
    )

    expect(pack.files).toHaveLength(13)
    expect(pack.manifest.workflow_templates).toHaveLength(13)
    expect(pack.manifest.safety).toEqual({
      contains_secret_values: false,
      mutates_customer_repository: false,
      requires_manual_install_review: true,
    })
    expect(pack.files.map((file) => file.file)).toContain('.github/workflows/release-readiness-gate.yml')
    expect(pack.files.map((file) => file.file)).toContain('.github/workflows/product-vulnerability-review-trend-enforcement.yml')
    expect(pack.readme).toContain('GitGov Workflow Template Pack')
    expect(JSON.stringify(pack)).not.toContain('__DEFAULT_BRANCH__')
    expect(JSON.stringify(pack)).not.toContain('__JIRA_PROJECT_KEY__')
    expect(JSON.stringify(pack)).not.toContain('GITGOV_API_KEY=')
    expect(JSON.stringify(pack)).not.toContain('SONAR_TOKEN=')
  })

  it('builds a stable workflow template pack filename', () => {
    expect(buildEnterpriseWorkflowTemplatePackFilename(DEFAULT_ENTERPRISE_ADOPTION_PROFILE)).toBe(
      'exampleco-example-org-example-repo-workflow-template-pack.json',
    )
  })

  it('builds provider health from profile and observable evidence', () => {
    const health = buildEnterpriseProviderHealth(DEFAULT_ENTERPRISE_ADOPTION_PROFILE, {
      githubEventsTotal: 42,
      jiraCommitsWithTicket: 12,
      jiraCoveragePercentage: 80,
      pipelineRuns7d: 10,
      pipelineSuccess7d: 9,
      sonarRuns: 3,
      sonarSuccessful: 3,
      activeRepos: 1,
    })

    expect(health.map((check) => [check.provider, check.status])).toEqual([
      ['github', 'ready'],
      ['jira', 'ready'],
      ['jenkins', 'ready'],
      ['sonarqube', 'ready'],
    ])
    expect(JSON.stringify(health)).not.toContain('token')
    expect(JSON.stringify(health)).not.toContain('secret value')
  })

  it('marks selected providers as needing evidence when telemetry has not arrived', () => {
    const health = buildEnterpriseProviderHealth(DEFAULT_ENTERPRISE_ADOPTION_PROFILE)

    expect(health.map((check) => [check.provider, check.status])).toEqual([
      ['github', 'needs-evidence'],
      ['jira', 'needs-evidence'],
      ['jenkins', 'needs-evidence'],
      ['sonarqube', 'needs-evidence'],
    ])
  })

  it('marks Jira provider as needing config when the project key is missing', () => {
    const profile: EnterpriseAdoptionProfile = {
      ...DEFAULT_ENTERPRISE_ADOPTION_PROFILE,
      jira_project_key: '',
      providers: ['jira'],
    }

    const health = buildEnterpriseProviderHealth(profile, {
      jiraCommitsWithTicket: 4,
      jiraCoveragePercentage: 50,
    })

    expect(health).toEqual([
      expect.objectContaining({
        provider: 'jira',
        status: 'needs-config',
      }),
    ])
  })
})
