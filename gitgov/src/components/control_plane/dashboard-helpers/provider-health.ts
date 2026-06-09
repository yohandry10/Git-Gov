import {
  ADOPTION_MODULE_IDS,
  ADOPTION_PROVIDER_IDS,
  ADOPTION_PROVIDER_OPTIONS,
  uniqueKnownValues,
  type AdoptionProvider,
  type EnterpriseAdoptionPack,
  type EnterpriseAdoptionProfile,
  type EnterpriseProviderHealthCheck,
  type EnterpriseProviderHealthEvidence,
} from './adoption-profile'
import { buildEnterpriseAdoptionPack } from './adoption-pack'

function hasPackVariable(pack: EnterpriseAdoptionPack, name: string): boolean {
  return pack.variables.some((variable) => variable.name === name)
}

function hasPackSecret(pack: EnterpriseAdoptionPack, name: string): boolean {
  return pack.secrets.some((secret) => secret.name === name)
}

function providerLabel(provider: AdoptionProvider): string {
  return ADOPTION_PROVIDER_OPTIONS.find((option) => option.id === provider)?.label ?? provider
}

function selectedProviders(profile: EnterpriseAdoptionProfile): AdoptionProvider[] {
  return uniqueKnownValues(profile.providers, ADOPTION_PROVIDER_IDS)
}

export function buildEnterpriseProviderHealth(
  profile: EnterpriseAdoptionProfile,
  evidence: EnterpriseProviderHealthEvidence = {},
  pack = buildEnterpriseAdoptionPack(profile),
): EnterpriseProviderHealthCheck[] {
  const checks: EnterpriseProviderHealthCheck[] = []
  const modules = uniqueKnownValues(profile.modules, ADOPTION_MODULE_IDS)
  const jiraKey = profile.jira_project_key.trim()
  const githubEventsTotal = evidence.githubEventsTotal ?? 0
  const jiraCommitsWithTicket = evidence.jiraCommitsWithTicket ?? 0
  const jiraCoveragePercentage = evidence.jiraCoveragePercentage ?? 0
  const pipelineRuns7d = evidence.pipelineRuns7d ?? 0
  const pipelineSuccess7d = evidence.pipelineSuccess7d ?? 0
  const sonarRuns = evidence.sonarRuns ?? 0
  const sonarSuccessful = evidence.sonarSuccessful ?? 0
  const activeRepos = evidence.activeRepos ?? 0

  for (const provider of selectedProviders(profile)) {
    if (provider === 'github') {
      const hasTelemetryConfig = hasPackVariable(pack, 'GITGOV_URL') && hasPackSecret(pack, 'GITGOV_API_KEY')
      checks.push({
        provider,
        label: providerLabel(provider),
        status: !hasTelemetryConfig ? 'needs-config' : githubEventsTotal > 0 ? 'ready' : 'needs-evidence',
        evidence: githubEventsTotal > 0
          ? `${githubEventsTotal} GitHub events observed`
          : 'No GitHub webhook or workflow evidence observed yet',
        next_step: hasTelemetryConfig
          ? 'Confirm signed webhook events and GitGov workflow telemetry are installed.'
          : 'Add GITGOV_URL and GITGOV_API_KEY to the adoption pack.',
      })
      continue
    }

    if (provider === 'jira') {
      const hasTraceabilityEvidence = jiraCommitsWithTicket > 0 || jiraCoveragePercentage > 0
      checks.push({
        provider,
        label: providerLabel(provider),
        status: !jiraKey ? 'needs-config' : hasTraceabilityEvidence ? 'ready' : 'needs-evidence',
        evidence: hasTraceabilityEvidence
          ? `${jiraCommitsWithTicket} ticket-linked commits, ${jiraCoveragePercentage.toFixed(2)}% coverage`
          : 'No Jira ticket correlation evidence observed yet',
        next_step: jiraKey
          ? 'Run Jira ingest/correlation and confirm ticket IDs appear in PRs, branches, or commits.'
          : 'Set the Jira project key for traceability validation.',
      })
      continue
    }

    if (provider === 'jenkins') {
      checks.push({
        provider,
        label: providerLabel(provider),
        status: pipelineRuns7d > 0 ? 'ready' : 'needs-evidence',
        evidence: pipelineRuns7d > 0
          ? `${pipelineRuns7d} pipeline runs observed in 7d, ${pipelineSuccess7d} successful`
          : 'No Jenkins pipeline evidence observed in the current 7d window',
        next_step: 'Publish Jenkins job telemetry to GitGov and verify pipeline evidence appears.',
      })
      continue
    }

    if (provider === 'sonarqube') {
      const hasSonarConfig = hasPackVariable(pack, 'SONAR_HOST_URL') && hasPackVariable(pack, 'SONAR_PROJECT_KEY')
      checks.push({
        provider,
        label: providerLabel(provider),
        status: !modules.includes('quality-gates') || !hasSonarConfig
          ? 'needs-config'
          : sonarRuns > 0
            ? 'ready'
            : 'needs-evidence',
        evidence: sonarRuns > 0
          ? `${sonarRuns} Sonar/quality runs observed, ${sonarSuccessful} successful`
          : 'No quality gate evidence observed in current dashboard stats',
        next_step: modules.includes('quality-gates')
          ? 'Validate SonarQube runtime reachability from the chosen runner.'
          : 'Enable the Quality gates module before validating SonarQube.',
      })
      continue
    }

    if (provider === 'render') {
      checks.push({
        provider,
        label: providerLabel(provider),
        status: activeRepos > 0 ? 'ready' : 'needs-evidence',
        evidence: activeRepos > 0
          ? `${activeRepos} active repositories observed by GitGov`
          : 'No deployment-provider evidence is available in the current adoption profile',
        next_step: 'Record deployment health and release metadata without storing provider tokens.',
      })
      continue
    }

    checks.push({
      provider,
      label: providerLabel(provider),
      status: 'needs-evidence',
      evidence: 'No Vercel deployment evidence is available in the current adoption profile',
      next_step: 'Connect deployment status or preview evidence when Vercel is used by the customer.',
    })
  }

  return checks
}
