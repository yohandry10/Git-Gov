import {
  DEFAULT_ENTERPRISE_ADOPTION_PROFILE,
  buildEnterpriseAdoptionPack,
  buildEnterpriseOnboardingGuide,
  buildEnterpriseOnboardingReadinessReport,
  buildEnterpriseOnboardingReadinessReportFilename,
  buildEnterpriseOnboardingRemediationPlan,
  buildEnterpriseOnboardingRemediationPlanFilename,
  buildEnterpriseWorkflowTemplatePack,
  buildEnterpriseWorkflowTemplatePackFilename,
  buildFirstGovernedRepoSetupBaseline,
  buildEnterpriseProviderHealth,
  buildReleaseGovernanceEnvironmentRows,
  buildOperationalEvidenceMetrics,
  formatOperationalMetricDuration,
  normalizeFirstGovernedRepoSetupDraft,
  normalizeEnterpriseOnboardingChecklistTracking,
  removeReleaseGovernanceEnvironmentOverride,
  updateReleaseGovernanceBaseMode,
  updateReleaseGovernanceEnvironmentOverrideMode,
  validateFirstGovernedRepoSetupDraft,
  upsertEnterpriseOnboardingChecklistTrackingItem,
  validateEnterpriseAdoptionProfile,
  type EnterpriseAdoptionProfile,
  type OperationalPipelineEvidence,
} from '@/components/control_plane/dashboard-helpers'

describe('first governed repo setup helpers', () => {
  it('keeps the first repo blocked when the repository name is not owner/repo', () => {
    const draft = normalizeFirstGovernedRepoSetupDraft({
      repository_full_name: 'missing-owner',
      baseline: buildFirstGovernedRepoSetupBaseline({
        repository_full_name: 'missing-owner',
        default_branch: 'main',
        goal: 'govern_release',
        selected_providers: ['github'],
        selected_modules: ['traceability', 'release-readiness', 'evidence-packets'],
        policy_preset: 'moderate',
        policyWorkflowPreviewAcknowledged: true,
      }),
    })

    const validation = validateFirstGovernedRepoSetupDraft(draft)

    expect(validation.ready).toBe(false)
    expect(validation.gateReadiness).toBe('needs_repo')
    expect(validation.gaps).toContain('repository_full_name')
    expect(validation.errors).toContain('Repository must use owner/repo format.')
  })

  it('requires policy and workflow preview before advisory gate readiness', () => {
    const draft = normalizeFirstGovernedRepoSetupDraft({
      repository_full_name: 'example/app',
      selected_modules: ['traceability', 'release-readiness', 'evidence-packets', 'quality-gates'],
    })

    const validation = validateFirstGovernedRepoSetupDraft(draft)

    expect(validation.ready).toBe(false)
    expect(validation.gateReadiness).toBe('needs_preview')
    expect(validation.gaps).toContain('policy_workflow_preview')
    expect(validation.gaps).not.toContain('quality_gate_evidence')
  })

  it('builds a baseline-ready first result when repo, providers, modules, and preview are present', () => {
    const baseline = buildFirstGovernedRepoSetupBaseline({
      repository_full_name: 'example/app',
      default_branch: 'release',
      goal: 'generate_audit_evidence',
      selected_providers: ['github', 'jira', 'jenkins'],
      selected_modules: ['traceability', 'release-readiness', 'evidence-packets', 'quality-gates', 'formal-approval'],
      policy_preset: 'strict',
      policyWorkflowPreviewAcknowledged: true,
    })

    expect(baseline.gate_readiness).toBe('baseline_ready')
    expect(baseline.action_center_gaps).toEqual([])
    expect(baseline.first_result).toMatchObject({
      status: 'ready_for_advisory_gate',
      deployment_gate_mode: 'advisory',
      cta: 'simulate_deployment_gate',
    })
    expect(baseline.first_result.evidence_contract).toMatchObject({
      repo: 'example/app',
      branch: 'release',
      providers: ['github', 'jira', 'jenkins'],
    })
  })
})

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
    expect(pack.release_governance).toEqual({
      mode: 'record-only',
      environment: 'production',
      approval_required: false,
      enforcement: 'disabled',
      quorum: {
        enabled: false,
        rules: [],
      },
      environment_overrides: [],
    })
    expect(pack.policy_rules).toContainEqual({
      rule: 'Release approval governance',
      setting: 'record-only',
    })
    expect(pack.policy_rules).toContainEqual({
      rule: 'Release governance artifact monitor',
      setting: 'not generated by default',
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

  it('keeps formal release approval as non-blocking record-only by default', () => {
    const profile: EnterpriseAdoptionProfile = {
      ...DEFAULT_ENTERPRISE_ADOPTION_PROFILE,
      modules: [...DEFAULT_ENTERPRISE_ADOPTION_PROFILE.modules, 'formal-approval'],
    }

    const pack = buildEnterpriseAdoptionPack(profile, '2026-04-30T00:00:00.000Z')

    expect(pack.open_product_gaps).toEqual([])
    expect(pack.manual_steps).toContainEqual({
      step: 'Review release approval policy',
      detail: 'Default record-only mode stores release approval evidence and does not block customer releases. Environment overrides: none.',
    })
  })

  it('requires formal approval module before opt-in release governance modes', () => {
    const profile: EnterpriseAdoptionProfile = {
      ...DEFAULT_ENTERPRISE_ADOPTION_PROFILE,
      release_governance: {
        mode: 'approval-required',
        environment: 'production',
        approval_required: true,
        enforcement: 'blocking',
        quorum: { enabled: false, rules: [] },
      },
    }

    const validation = validateEnterpriseAdoptionProfile(profile)

    expect(validation.valid).toBe(false)
    expect(validation.errors).toContain(
      'Enable the Formal approval module before choosing advisory, approval-required, or quorum-required release governance.',
    )
  })

  it('exports explicit quorum release governance without changing secret policy', () => {
    const profile: EnterpriseAdoptionProfile = {
      ...DEFAULT_ENTERPRISE_ADOPTION_PROFILE,
      modules: [...DEFAULT_ENTERPRISE_ADOPTION_PROFILE.modules, 'formal-approval'],
      release_governance: {
        mode: 'quorum-required',
        environment: 'production',
        approval_required: true,
        enforcement: 'blocking',
        quorum: {
          enabled: true,
          rules: [
            { role: 'engineering', required: 1 },
            { role: 'security', required: 1 },
          ],
        },
      },
    }

    const pack = buildEnterpriseAdoptionPack(profile, '2026-04-30T00:00:00.000Z')

    expect(pack.release_governance.mode).toBe('quorum-required')
    expect(pack.policy_rules).toContainEqual({
      rule: 'Release approval enforcement',
      setting: 'blocking',
    })
    expect(pack.policy_rules).toContainEqual({
      rule: 'Release governance gate',
      setting: 'manual opt-in workflow',
    })
    expect(pack.policy_rules).toContainEqual({
      rule: 'Release governance environment overrides',
      setting: 'none',
    })
    expect(pack.policy_rules).toContainEqual({
      rule: 'Release approval quorum',
      setting: 'engineering:1, security:1',
    })
    expect(pack.workflow_plan.map((workflow) => workflow.file)).toContain(
      '.github/workflows/release-governance-gate.yml',
    )
    expect(pack.workflow_plan.map((workflow) => workflow.file)).toContain(
      '.github/workflows/release-governance-gate-artifact-monitor.yml',
    )
    expect(JSON.stringify(pack)).not.toContain('GITGOV_API_KEY=')
  })

  it('generates release governance gate from environment override opt-in', () => {
    const profile: EnterpriseAdoptionProfile = {
      ...DEFAULT_ENTERPRISE_ADOPTION_PROFILE,
      modules: [...DEFAULT_ENTERPRISE_ADOPTION_PROFILE.modules, 'formal-approval'],
      release_governance: {
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
      },
    }

    const pack = buildEnterpriseAdoptionPack(profile, '2026-05-01T00:00:00.000Z')
    const workflowPack = buildEnterpriseWorkflowTemplatePack(profile, '2026-05-01T00:00:00.000Z')
    const gate = workflowPack.files.find((file) => file.file === '.github/workflows/release-governance-gate.yml')
    const monitor = workflowPack.files.find((file) => file.file === '.github/workflows/release-governance-gate-artifact-monitor.yml')

    expect(pack.release_governance.mode).toBe('record-only')
    expect(pack.release_governance.environment_overrides).toHaveLength(1)
    expect(pack.policy_rules).toContainEqual({
      rule: 'Release governance environment overrides',
      setting: 'production:approval-required',
    })
    expect(pack.workflow_plan.map((workflow) => workflow.file)).toContain('.github/workflows/release-governance-gate.yml')
    expect(pack.workflow_plan.map((workflow) => workflow.file)).toContain('.github/workflows/release-governance-gate-artifact-monitor.yml')
    expect(gate?.content).toContain('default: "production"')
    expect(gate?.content).toContain('default: true')
    expect(monitor?.content).toContain('ARTIFACT_PREFIX: "release-governance-gate-"')
    expect(JSON.stringify(workflowPack)).not.toContain('GITGOV_API_KEY=')
  })

  it('keeps production stricter than staging in the environment policy matrix', () => {
    const profile: EnterpriseAdoptionProfile = {
      ...DEFAULT_ENTERPRISE_ADOPTION_PROFILE,
      modules: [...DEFAULT_ENTERPRISE_ADOPTION_PROFILE.modules, 'formal-approval'],
      release_governance: {
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
      },
    }

    const rows = buildReleaseGovernanceEnvironmentRows(profile.release_governance)

    expect(rows).toEqual([
      {
        source: 'base',
        environment: 'staging',
        mode: 'record-only',
        approval_required: false,
        enforcement: 'disabled',
        quorum_summary: 'disabled',
      },
      {
        source: 'override',
        environment: 'production',
        mode: 'approval-required',
        approval_required: true,
        enforcement: 'blocking',
        quorum_summary: 'disabled',
        override_index: 0,
      },
    ])
  })

  it('preserves environment overrides when the base release governance mode changes', () => {
    const currentPolicy = {
      mode: 'record-only' as const,
      environment: 'staging',
      approval_required: false,
      enforcement: 'disabled' as const,
      quorum: { enabled: false, rules: [] },
      environment_overrides: [
        {
          mode: 'approval-required' as const,
          environment: 'production',
          approval_required: true,
          enforcement: 'blocking' as const,
          quorum: { enabled: false, rules: [] },
        },
      ],
    }

    const updatedPolicy = updateReleaseGovernanceBaseMode(currentPolicy, 'advisory')

    expect(updatedPolicy.mode).toBe('advisory')
    expect(updatedPolicy.environment).toBe('staging')
    expect(updatedPolicy.environment_overrides).toHaveLength(1)
    expect(updatedPolicy.environment_overrides?.[0]).toMatchObject({
      mode: 'approval-required',
      environment: 'production',
      approval_required: true,
      enforcement: 'blocking',
    })
  })

  it('falls back to the base release policy after an environment override is removed', () => {
    const currentPolicy = {
      mode: 'record-only' as const,
      environment: 'staging',
      approval_required: false,
      enforcement: 'disabled' as const,
      quorum: { enabled: false, rules: [] },
      environment_overrides: [
        {
          mode: 'approval-required' as const,
          environment: 'production',
          approval_required: true,
          enforcement: 'blocking' as const,
          quorum: { enabled: false, rules: [] },
        },
      ],
    }

    const removed = removeReleaseGovernanceEnvironmentOverride(currentPolicy, 0)
    const rows = buildReleaseGovernanceEnvironmentRows(removed)

    expect(rows).toHaveLength(1)
    expect(rows[0]).toMatchObject({
      source: 'base',
      environment: 'staging',
      mode: 'record-only',
      enforcement: 'disabled',
    })
  })

  it('updates an override to quorum-required with concrete approver rules', () => {
    const currentPolicy = {
      mode: 'record-only' as const,
      environment: 'staging',
      approval_required: false,
      enforcement: 'disabled' as const,
      quorum: { enabled: false, rules: [] },
      environment_overrides: [
        {
          mode: 'approval-required' as const,
          environment: 'production',
          approval_required: true,
          enforcement: 'blocking' as const,
          quorum: { enabled: false, rules: [] },
        },
      ],
    }

    const updated = updateReleaseGovernanceEnvironmentOverrideMode(currentPolicy, 0, 'quorum-required')
    const production = buildReleaseGovernanceEnvironmentRows(updated).find((row) => row.environment === 'production')

    expect(production).toMatchObject({
      source: 'override',
      mode: 'quorum-required',
      enforcement: 'blocking',
      quorum_summary: 'engineering:1, security:1',
    })
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
    expect(pack.readme).toContain('Release governance: `record-only`')
    expect(pack.manifest.release_governance.mode).toBe('record-only')
    expect(JSON.stringify(pack)).not.toContain('__DEFAULT_BRANCH__')
    expect(JSON.stringify(pack)).not.toContain('__JIRA_PROJECT_KEY__')
    expect(JSON.stringify(pack)).not.toContain('GITGOV_API_KEY=')
    expect(JSON.stringify(pack)).not.toContain('SONAR_TOKEN=')
  })

  it('adds release governance gate template only after customer opt-in', () => {
    const profile: EnterpriseAdoptionProfile = {
      ...DEFAULT_ENTERPRISE_ADOPTION_PROFILE,
      modules: [...DEFAULT_ENTERPRISE_ADOPTION_PROFILE.modules, 'formal-approval'],
      release_governance: {
        mode: 'approval-required',
        environment: 'production',
        approval_required: true,
        enforcement: 'blocking',
        quorum: { enabled: false, rules: [] },
      },
    }

    const pack = buildEnterpriseWorkflowTemplatePack(profile, '2026-05-01T00:00:00.000Z')
    const gate = pack.files.find((file) => file.file === '.github/workflows/release-governance-gate.yml')

    expect(gate).toBeDefined()
    expect(gate?.content).toContain('name: GitGov Release Governance Gate')
    expect(gate?.content).toContain('default: true')
    expect(gate?.content).toContain('target_sha:')
    expect(gate?.content).toContain('branch = $branch')
    expect(gate?.content).toContain('target_sha = $targetSha')
    expect(gate?.content).toContain('/deployment-gates/authorize')
    expect(gate?.content).toContain('authorization_id = $authorization.authorization_id')
    expect(pack.manifest.workflow_templates.map((workflow) => workflow.file)).toContain(
      '.github/workflows/release-governance-gate.yml',
    )
    expect(pack.manifest.workflow_templates.map((workflow) => workflow.file)).toContain(
      '.github/workflows/release-governance-gate-artifact-monitor.yml',
    )
    expect(JSON.stringify(pack)).not.toContain('GITGOV_API_KEY=')
  })

  it('does not add release governance artifact monitor without artifact-monitoring opt-in', () => {
    const profile: EnterpriseAdoptionProfile = {
      ...DEFAULT_ENTERPRISE_ADOPTION_PROFILE,
      modules: [
        ...DEFAULT_ENTERPRISE_ADOPTION_PROFILE.modules.filter((module) => module !== 'artifact-monitoring'),
        'formal-approval',
      ],
      release_governance: {
        mode: 'approval-required',
        environment: 'production',
        approval_required: true,
        enforcement: 'blocking',
        quorum: { enabled: false, rules: [] },
      },
    }

    const pack = buildEnterpriseWorkflowTemplatePack(profile, '2026-05-01T00:00:00.000Z')

    expect(pack.manifest.workflow_templates.map((workflow) => workflow.file)).toContain(
      '.github/workflows/release-governance-gate.yml',
    )
    expect(pack.manifest.workflow_templates.map((workflow) => workflow.file)).not.toContain(
      '.github/workflows/release-governance-gate-artifact-monitor.yml',
    )
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

  it('builds an onboarding readiness snapshot without secret values', () => {
    const providerHealth = buildEnterpriseProviderHealth(DEFAULT_ENTERPRISE_ADOPTION_PROFILE, {
      githubEventsTotal: 42,
      jiraCommitsWithTicket: 12,
      jiraCoveragePercentage: 80,
      pipelineRuns7d: 10,
      pipelineSuccess7d: 9,
      sonarRuns: 3,
      sonarSuccessful: 3,
      activeRepos: 1,
    })

    const report = buildEnterpriseOnboardingReadinessReport(
      DEFAULT_ENTERPRISE_ADOPTION_PROFILE,
      providerHealth,
      {
        status: 'ready',
        totals: {
          workflows_missing: 0,
          workflows_different: 0,
          variables_missing: 0,
          secrets_missing: 0,
        },
      },
      '2026-05-01T00:00:00.000Z',
    )

    expect(report.status).toBe('ready')
    expect(report.readiness_score).toBe(100)
    expect(report.stage_counts).toEqual({
      ready: 6,
      'needs-action': 0,
      blocked: 0,
    })
    expect(report.safety).toEqual({
      contains_secret_values: false,
      reads_secret_values: false,
      mutates_customer_repository: false,
      mutates_provider_state: false,
      release_blocking_default: false,
    })
    expect(JSON.stringify(report)).not.toContain('GITGOV_API_KEY=')
    expect(JSON.stringify(report)).not.toContain('SONAR_TOKEN=')
  })

  it('keeps dashboard readiness honest when remote workflow readiness is missing', () => {
    const providerHealth = buildEnterpriseProviderHealth(DEFAULT_ENTERPRISE_ADOPTION_PROFILE)
    const report = buildEnterpriseOnboardingReadinessReport(
      DEFAULT_ENTERPRISE_ADOPTION_PROFILE,
      providerHealth,
      null,
      '2026-05-01T00:00:00.000Z',
    )

    expect(report.status).toBe('needs-action')
    expect(report.readiness_score).toBeLessThan(100)
    expect(report.next_actions).toContain(
      'Remote workflow readiness: Run the read-only remote workflow readiness validator after install or PR merge.',
    )
    expect(report.stages.find((stage) => stage.id === 'actions-config')?.status).toBe('needs-action')
  })

  it('blocks onboarding readiness for invalid profiles', () => {
    const profile: EnterpriseAdoptionProfile = {
      ...DEFAULT_ENTERPRISE_ADOPTION_PROFILE,
      repository_full_name: 'missing-owner',
    }

    const report = buildEnterpriseOnboardingReadinessReport(profile, [], null, '2026-05-01T00:00:00.000Z')

    expect(report.status).toBe('blocked')
    expect(report.stages.find((stage) => stage.id === 'profile')?.status).toBe('blocked')
    expect(report.next_actions.some((action) => action.includes('Repository must look like owner/repo.'))).toBe(true)
  })

  it('builds a stable onboarding readiness filename', () => {
    expect(buildEnterpriseOnboardingReadinessReportFilename(DEFAULT_ENTERPRISE_ADOPTION_PROFILE)).toBe(
      'exampleco-example-org-example-repo-onboarding-readiness.json',
    )
  })

  it('builds a secret-safe onboarding remediation plan from dashboard readiness', () => {
    const providerHealth = buildEnterpriseProviderHealth(DEFAULT_ENTERPRISE_ADOPTION_PROFILE)
    const readiness = buildEnterpriseOnboardingReadinessReport(
      DEFAULT_ENTERPRISE_ADOPTION_PROFILE,
      providerHealth,
      null,
      '2026-05-02T00:00:00.000Z',
    )
    const pack = buildEnterpriseAdoptionPack(DEFAULT_ENTERPRISE_ADOPTION_PROFILE, '2026-05-02T00:00:00.000Z')
    const plan = buildEnterpriseOnboardingRemediationPlan(readiness, pack, '2026-05-02T00:01:00.000Z')

    expect(plan.remediation_status).toBe('needs-action')
    expect(plan.action_count).toBe(3)
    expect(plan.actions.map((action) => [action.priority, action.stage_id, action.owner])).toEqual([
      [2, 'providers', 'Platform owner'],
      [4, 'remote-workflows', 'Repository admin'],
      [5, 'actions-config', 'Repository admin'],
    ])
    expect(plan.github_actions_configuration.variables_count).toBe(3)
    expect(plan.github_actions_configuration.secrets_count).toBe(2)
    expect(plan.github_actions_configuration.commands).toContainEqual({
      kind: 'variable',
      name: 'GITGOV_URL',
      command: 'gh variable set GITGOV_URL --repo example-org/example-repo --body "<value>"',
      contains_secret_value: false,
    })
    expect(plan.github_actions_configuration.commands).toContainEqual({
      kind: 'secret',
      name: 'GITGOV_API_KEY',
      command: 'gh secret set GITGOV_API_KEY --repo example-org/example-repo',
      contains_secret_value: false,
    })
    expect(plan.safety).toEqual({
      contains_secret_values: false,
      reads_secret_values: false,
      mutates_customer_repository: false,
      mutates_provider_state: false,
      creates_github_actions_variables: false,
      creates_github_actions_secrets: false,
      release_blocking_default: false,
    })
    expect(JSON.stringify(plan)).not.toContain('GITGOV_API_KEY=')
    expect(JSON.stringify(plan)).not.toContain('SONAR_TOKEN=')
  })

  it('builds a guided onboarding checklist from dashboard readiness and remediation', () => {
    const providerHealth = buildEnterpriseProviderHealth(DEFAULT_ENTERPRISE_ADOPTION_PROFILE)
    const readiness = buildEnterpriseOnboardingReadinessReport(
      DEFAULT_ENTERPRISE_ADOPTION_PROFILE,
      providerHealth,
      null,
      '2026-05-02T00:00:00.000Z',
    )
    const pack = buildEnterpriseAdoptionPack(DEFAULT_ENTERPRISE_ADOPTION_PROFILE, '2026-05-02T00:00:00.000Z')
    const plan = buildEnterpriseOnboardingRemediationPlan(readiness, pack, '2026-05-02T00:01:00.000Z')
    const guide = buildEnterpriseOnboardingGuide(readiness, plan, '2026-05-02T00:02:00.000Z')

    expect(guide.readiness_status).toBe('needs-action')
    expect(guide.completed_steps).toBe(3)
    expect(guide.total_steps).toBe(6)
    expect(guide.next_step).toEqual(expect.objectContaining({
      stage_id: 'providers',
      status: 'next',
      owner: 'Platform owner',
    }))
    expect(guide.steps.map((step) => [step.stage_id, step.status])).toEqual([
      ['profile', 'complete'],
      ['providers', 'next'],
      ['workflow-pack', 'complete'],
      ['remote-workflows', 'todo'],
      ['actions-config', 'todo'],
      ['release-governance', 'complete'],
    ])
    expect(guide.configuration_summary).toEqual({
      variable_names: ['GITGOV_URL', 'SONAR_HOST_URL', 'SONAR_PROJECT_KEY'],
      secret_names: ['GITGOV_API_KEY', 'SONAR_TOKEN'],
      commands_are_placeholders: true,
      suggested_commands_count: 5,
    })
    expect(guide.safety).toEqual({
      contains_secret_values: false,
      reads_secret_values: false,
      mutates_customer_repository: false,
      mutates_provider_state: false,
      creates_github_actions_variables: false,
      creates_github_actions_secrets: false,
      release_blocking_default: false,
    })
    expect(JSON.stringify(guide)).not.toContain('GITGOV_API_KEY=')
    expect(JSON.stringify(guide)).not.toContain('SONAR_TOKEN=')
  })

  it('shows a completed guide when onboarding readiness is fully ready', () => {
    const providerHealth = buildEnterpriseProviderHealth(DEFAULT_ENTERPRISE_ADOPTION_PROFILE, {
      githubEventsTotal: 42,
      jiraCommitsWithTicket: 12,
      jiraCoveragePercentage: 80,
      pipelineRuns7d: 10,
      pipelineSuccess7d: 9,
      sonarRuns: 3,
      sonarSuccessful: 3,
      activeRepos: 1,
    })
    const readiness = buildEnterpriseOnboardingReadinessReport(
      DEFAULT_ENTERPRISE_ADOPTION_PROFILE,
      providerHealth,
      {
        status: 'ready',
        totals: {
          workflows_missing: 0,
          workflows_different: 0,
          variables_missing: 0,
          secrets_missing: 0,
        },
      },
      '2026-05-02T00:00:00.000Z',
    )
    const pack = buildEnterpriseAdoptionPack(DEFAULT_ENTERPRISE_ADOPTION_PROFILE, '2026-05-02T00:00:00.000Z')
    const plan = buildEnterpriseOnboardingRemediationPlan(readiness, pack, '2026-05-02T00:01:00.000Z')
    const guide = buildEnterpriseOnboardingGuide(readiness, plan, '2026-05-02T00:02:00.000Z')

    expect(guide.next_step).toBeNull()
    expect(guide.completed_steps).toBe(6)
    expect(guide.steps.every((step) => step.status === 'complete')).toBe(true)
  })

  it('normalizes guided checklist tracking without secret values', () => {
    const tracking = normalizeEnterpriseOnboardingChecklistTracking({
      version: 1,
      items: [
        {
          stage_id: 'providers',
          status: 'waiting',
          owner: ' Platform owner ',
          note: ' Waiting for evidence ',
          external_ref: ' KAN-60 ',
          target_date: '2026-05-08',
        },
        {
          stage_id: 'providers',
          status: 'done',
          note: 'duplicate should be ignored',
        },
        {
          stage_id: 'actions-config',
          status: 'invalid' as never,
          owner: 'Repository admin',
        },
      ],
    })

    expect(tracking).toEqual({
      version: 1,
      items: [
        {
          stage_id: 'providers',
          status: 'waiting',
          owner: 'Platform owner',
          note: 'Waiting for evidence',
          external_ref: 'KAN-60',
          target_date: '2026-05-08',
          updated_at: undefined,
        },
        {
          stage_id: 'actions-config',
          status: 'open',
          owner: 'Repository admin',
          note: undefined,
          external_ref: undefined,
          target_date: undefined,
          updated_at: undefined,
        },
      ],
    })
    expect(JSON.stringify(tracking)).not.toContain('GITGOV_API_KEY=')
  })

  it('upserts guided checklist tracking items in stage order', () => {
    const tracking = normalizeEnterpriseOnboardingChecklistTracking()
    const withActions = upsertEnterpriseOnboardingChecklistTrackingItem(tracking, {
      stage_id: 'actions-config',
      status: 'in-progress',
      owner: 'Repository admin',
      note: 'Configuring names only',
      updated_at: '2026-05-02T00:00:00.000Z',
    })
    const updated = upsertEnterpriseOnboardingChecklistTrackingItem(withActions, {
      stage_id: 'providers',
      status: 'waiting',
      owner: 'Platform owner',
      external_ref: 'KAN-60',
      updated_at: '2026-05-02T00:01:00.000Z',
    })

    expect(updated.items.map((item) => [item.stage_id, item.status])).toEqual([
      ['providers', 'waiting'],
      ['actions-config', 'in-progress'],
    ])
    expect(updated.items[0].external_ref).toBe('KAN-60')
    expect(updated.items[1].note).toBe('Configuring names only')
  })

  it('builds a stable onboarding remediation plan filename', () => {
    expect(buildEnterpriseOnboardingRemediationPlanFilename(DEFAULT_ENTERPRISE_ADOPTION_PROFILE)).toBe(
      'exampleco-example-org-example-repo-onboarding-remediation-plan.json',
    )
  })
})
