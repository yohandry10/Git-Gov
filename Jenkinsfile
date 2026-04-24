import groovy.json.JsonOutput
import groovy.json.JsonSlurperClassic

pipeline {
  agent any

  environment {
    // Ajusta estas variables en Jenkins (Manage Jenkins -> System / Credentials)
    GITGOV_URL = credentials('gitgov-url') // ej: http://host.docker.internal:3000
    GITGOV_API_KEY = credentials('gitgov-api-key')
    // Opcional si activaste JENKINS_WEBHOOK_SECRET en el server
    GITGOV_JENKINS_SECRET = credentials('gitgov-jenkins-secret')
    // true => falla el build si GitGov responde >=400 o si curl falla
    GITGOV_STRICT = 'false'
    // Runtime telemetry enrichments (set by optional Sonar stage)
    GITGOV_SONAR_STATUS = 'NOT_RUN'
    GITGOV_SONAR_PROJECT_KEY = ''
    GITGOV_SONAR_DASHBOARD_URL = ''
    GITGOV_SONAR_HOST_URL = ''
    // Release readiness gate (optional)
    GITGOV_RELEASE_GATE_ENABLED = 'false'
    GITGOV_RELEASE_GATE_TIER = 'standard' // critical|standard|internal
    GITGOV_RELEASE_GATE_MIN = '0' // 0 => use tier default SLA target
    GITGOV_RELEASE_GATE_FAIL_MISSING = 'false' // fail when one of the 3 signals is missing
    GITGOV_RELEASE_GATE_HOURS = '168'
    GITGOV_RELEASE_GATE_CORRELATION_LIMIT = '500'
    GITGOV_RELEASE_GATE_ORG_NAME = '' // optional org_name filter for Jira coverage
  }

  options {
    timestamps()
  }

  stages {
    stage('Checkout') {
      steps {
        checkout scm
      }
    }

    stage('Sonar Scan (Optional)') {
      steps {
        script {
          def sonarStatus = 'SKIPPED'
          def inferredProjectKey = inferSonarProjectKey()
          def sonarProjectKey = (env.SONAR_PROJECT_KEY ?: inferredProjectKey).trim()
          def sonarHostUrl = (env.SONAR_HOST_URL ?: 'http://host.docker.internal:9000').trim()
          def sonarDashboardUrl = ''

          def persistSonarMeta = {
            def statusValue = (sonarStatus ?: '').replace('\n', ' ').replace('\r', ' ')
            def projectKeyValue = (sonarProjectKey ?: '').replace('\n', ' ').replace('\r', ' ')
            def hostUrlValue = (sonarHostUrl ?: '').replace('\n', ' ').replace('\r', ' ')
            def dashboardValue = (sonarDashboardUrl ?: '').replace('\n', ' ').replace('\r', ' ')
            writeFile file: 'gitgov-sonar-meta.properties', text: """status=${statusValue}
project_key=${projectKeyValue}
host_url=${hostUrlValue}
dashboard_url=${dashboardValue}
"""
          }

          def sonarToken = (env.SONAR_TOKEN ?: '').trim()
          if (!sonarToken) {
            try {
              withCredentials([string(credentialsId: 'gitgov-token', variable: 'SONAR_TOKEN_FROM_CREDENTIAL')]) {
                sonarToken = (env.SONAR_TOKEN_FROM_CREDENTIAL ?: '').trim()
              }
            } catch (ignored) {
              sonarToken = ''
            }
          }
          if (!sonarToken || !sonarProjectKey) {
            echo 'Skipping Sonar scan (missing SONAR_TOKEN or SONAR_PROJECT_KEY).'
            persistSonarMeta()
            return
          }

          try {
            def scannerBin = ensureSonarScannerBinary()
            withEnv([
              "SONAR_SCANNER_BIN=${scannerBin}",
              "SONAR_HOST_URL=${sonarHostUrl}",
              "SONAR_PROJECT_KEY=${sonarProjectKey}",
              "SONAR_TOKEN=${sonarToken}"
            ]) {
              def scanFailed = false
              def scanStatus = sh(
                script: '''
                  set +x
                  set -euo pipefail
                  "${SONAR_SCANNER_BIN}" \
                    -Dsonar.projectKey="${SONAR_PROJECT_KEY}" \
                    -Dsonar.projectName="GitGov" \
                    -Dsonar.sources=gitgov/gitgov-server/src,gitgov/src,gitgov/src-tauri/src,gitgov-web \
                    -Dsonar.exclusions=**/node_modules/**,**/target/**,**/dist/**,**/.next/**,**/coverage/**,**/public/**,**/*.min.js \
                    -Dsonar.sourceEncoding=UTF-8 \
                    -Dsonar.scm.provider=git \
                    -Dsonar.host.url="${SONAR_HOST_URL}" \
                    -Dsonar.token="${SONAR_TOKEN}"
                ''',
                returnStatus: true
              )
              if (scanStatus != 0) {
                scanFailed = true
                sonarStatus = 'SCAN_FAILED'
                def msg = "Sonar scanner returned non-zero exit (${scanStatus})"
                if (gitgovStrictModeEnabled()) {
                  error("${msg}; aborting because GITGOV_STRICT=true")
                }
                echo "${msg}; continuing because GITGOV_STRICT=false"
              }
              if (scanFailed) {
                return
              }
            }

            if (!fileExists('.scannerwork/report-task.txt')) {
              sonarStatus = 'UNKNOWN'
              echo 'Sonar report-task not found; unable to resolve quality gate.'
              return
            }

            def ceTaskId = sh(
              script: '''awk -F= '/^ceTaskId=/{print $2}' .scannerwork/report-task.txt | tail -n 1''',
              returnStdout: true
            ).trim()
            def dashboardUrl = sh(
              script: '''awk -F= '/^dashboardUrl=/{print $2}' .scannerwork/report-task.txt | tail -n 1''',
              returnStdout: true
            ).trim()
            def serverUrl = sh(
              script: '''awk -F= '/^serverUrl=/{print $2}' .scannerwork/report-task.txt | tail -n 1''',
              returnStdout: true
            ).trim()

            if (dashboardUrl) {
              if (sonarProjectKey && dashboardUrl.endsWith('id=')) {
                sonarDashboardUrl = "${dashboardUrl}${sonarProjectKey}"
              } else if (sonarProjectKey && dashboardUrl.endsWith('id')) {
                sonarDashboardUrl = "${dashboardUrl}=${sonarProjectKey}"
              } else {
                sonarDashboardUrl = dashboardUrl
              }
            }
            if (serverUrl) {
              sonarHostUrl = serverUrl
            }
            if (!sonarDashboardUrl && sonarProjectKey) {
              def normalizedHost = sonarHostUrl?.endsWith('/') ? sonarHostUrl[0..-2] : sonarHostUrl
              if (normalizedHost) {
                sonarDashboardUrl = "${normalizedHost}/dashboard?id=${sonarProjectKey}"
              }
            }

            if (!ceTaskId) {
              sonarStatus = 'UNKNOWN'
              echo 'Sonar CE task id is empty; unable to resolve quality gate.'
              return
            }

            def ceStatus = 'PENDING'
            def analysisId = ''
            withEnv([
              "SQ_HOST_URL=${sonarHostUrl}",
              "SQ_CE_TASK_ID=${ceTaskId}",
              "SONAR_TOKEN=${sonarToken}"
            ]) {
              for (int i = 0; i < 60; i++) {
                def ceTaskRaw = sh(
                  script: '''
                    set +x
                    set -e
                    url="${SQ_HOST_URL%/}/api/ce/task?id=${SQ_CE_TASK_ID}"
                    if body="$(curl -fsS -u "${SONAR_TOKEN}:" "$url")"; then
                      :
                    else
                      body="$(curl -fsS "$url")"
                    fi
                    printf '%s' "$body"
                  ''',
                  returnStdout: true
                ).trim()
                ceStatus = (extractJsonObjectField(ceTaskRaw, 'task', 'status') ?: 'UNKNOWN').toUpperCase()
                if (ceStatus == 'SUCCESS') {
                  analysisId = extractJsonObjectField(ceTaskRaw, 'task', 'analysisId') ?: ''
                  break
                }
                if (ceStatus in ['FAILED', 'CANCELED']) {
                  break
                }
                sleep(time: 5, unit: 'SECONDS')
              }
            }

            if (!analysisId) {
              sonarStatus = (ceStatus == 'PENDING') ? 'TIMEOUT' : ceStatus
              echo "Sonar analysis id unavailable (ce_status=${sonarStatus})."
              return
            }

            withEnv([
              "SQ_HOST_URL=${sonarHostUrl}",
              "SQ_ANALYSIS_ID=${analysisId}",
              "SONAR_TOKEN=${sonarToken}"
            ]) {
              def gateRaw = sh(
                script: '''
                  set +x
                  set -e
                  url="${SQ_HOST_URL%/}/api/qualitygates/project_status?analysisId=${SQ_ANALYSIS_ID}"
                  if body="$(curl -fsS -u "${SONAR_TOKEN}:" "$url")"; then
                    :
                  else
                    body="$(curl -fsS "$url")"
                  fi
                  printf '%s' "$body"
                ''',
                  returnStdout: true
                ).trim()
              writeFile file: 'sonar-quality-gate.json', text: gateRaw
              sonarStatus = (extractJsonObjectField(gateRaw, 'projectStatus', 'status') ?: 'UNKNOWN').toUpperCase()
            }

            echo "Sonar quality gate status: ${sonarStatus}"
          } catch (err) {
            sonarStatus = 'SCAN_FAILED'
            def msg = "Sonar scan stage failed: ${err}"
            if (gitgovStrictModeEnabled()) {
              error("${msg}; aborting because GITGOV_STRICT=true")
            }
            echo "${msg}; continuing because GITGOV_STRICT=false"
          } finally {
            persistSonarMeta()
          }
        }
      }
    }

    stage('Policy Check (Advisory)') {
      steps {
        script {
          def repoName = env.GIT_URL ? env.GIT_URL.replaceFirst('^.*github\\.com[/:]', '').replaceFirst('\\.git$', '') : ''
          def branchName = env.BRANCH_NAME ?: env.GIT_BRANCH ?: 'unknown'
          def commitSha = env.GIT_COMMIT ?: sh(script: 'git rev-parse HEAD', returnStdout: true).trim()
          def payload = JsonOutput.toJson([
            repo      : repoName,
            branch    : branchName,
            commit    : commitSha,
            user_login: (env.BUILD_USER_ID ?: env.BUILD_USER ?: 'jenkins'),
          ])
          writeFile file: 'gitgov-policy-check.json', text: payload

          def policyHttpCode = sh(
            script: '''
              set +x
              curl -sS \
                -o gitgov-policy-check-response.json \
                -w "%{http_code}" \
                -X POST "${GITGOV_URL%/}/policy/check" \
                -H "Authorization: Bearer ${GITGOV_API_KEY}" \
                -H "Content-Type: application/json" \
                --data @gitgov-policy-check.json
            ''',
            returnStdout: true
          ).trim()

          def responseRaw = fileExists('gitgov-policy-check-response.json')
            ? readFile('gitgov-policy-check-response.json').trim()
            : ''

          if (!(policyHttpCode in ['200', '409'])) {
            def msg = "GitGov policy/check transport failed (http=${policyHttpCode})"
            if (gitgovStrictModeEnabled()) {
              error("${msg}; aborting because GITGOV_STRICT=true")
            }
            echo "${msg}; continuing because GITGOV_STRICT=false"
            return
          }

          def reasons = extractJsonStringArray(responseRaw, 'reasons').join('; ')
          def warnings = extractJsonStringArray(responseRaw, 'warnings').join('; ')
          def allowed = extractJsonBoolean(responseRaw, 'allowed', false)
          def advisory = extractJsonBoolean(responseRaw, 'advisory', true)
          def enforcementApplied = extractJsonString(responseRaw, 'enforcement_applied') ?: 'unknown'

          if (warnings) {
            echo "GitGov policy warnings: ${warnings}"
          }

          if (!allowed) {
            def msg = "GitGov policy denied change (enforcement=${enforcementApplied}, advisory=${advisory}, reasons=${reasons})"
            if (advisory && !gitgovStrictModeEnabled()) {
              echo "${msg}; continuing because advisory and GITGOV_STRICT=false"
            } else {
              error(msg)
            }
          }
        }
      }
    }

    stage('Release Readiness Gate (Optional)') {
      steps {
        script {
          def persistReleaseMeta = { Map meta ->
            def statusValue = ((meta.status ?: 'SKIPPED') as String).replace('\n', ' ').replace('\r', ' ')
            def scoreValue = ((meta.score ?: '0') as String).replace('\n', ' ').replace('\r', ' ')
            def targetValue = ((meta.target ?: '0') as String).replace('\n', ' ').replace('\r', ' ')
            def coverageValue = ((meta.coverage ?: '0/3') as String).replace('\n', ' ').replace('\r', ' ')
            def tierValue = ((meta.tier ?: 'standard') as String).replace('\n', ' ').replace('\r', ' ')
            def reasonsValue = ((meta.reasons ?: '') as String).replace('\n', ' ').replace('\r', ' ')
            def warningsValue = ((meta.warnings ?: '') as String).replace('\n', ' ').replace('\r', ' ')
            writeFile file: 'gitgov-release-readiness.properties', text: """status=${statusValue}
score=${scoreValue}
target=${targetValue}
signal_coverage=${coverageValue}
tier=${tierValue}
reasons=${reasonsValue}
warnings=${warningsValue}
"""
          }

          if (!isEnabledFlag(env.GITGOV_RELEASE_GATE_ENABLED)) {
            echo 'Skipping Release Readiness Gate (GITGOV_RELEASE_GATE_ENABLED=false).'
            persistReleaseMeta([status: 'SKIPPED'])
            return
          }

          def repoName = env.GIT_URL ? env.GIT_URL.replaceFirst('^.*github\\.com[/:]', '').replaceFirst('\\.git$', '') : ''
          def branchName = (env.BRANCH_NAME ?: env.GIT_BRANCH ?: 'unknown').replaceFirst('^origin/', '')
          def tier = (env.GITGOV_RELEASE_GATE_TIER ?: 'standard').trim().toLowerCase()
          def tierProfiles = [
            critical: [pipeline: 0.5d, traceability: 0.2d, sonar: 0.3d, target: 85],
            standard: [pipeline: 0.45d, traceability: 0.25d, sonar: 0.3d, target: 75],
            internal: [pipeline: 0.4d, traceability: 0.2d, sonar: 0.4d, target: 65],
          ]
          def profile = tierProfiles.containsKey(tier) ? tierProfiles[tier] : tierProfiles.standard
          tier = tierProfiles.containsKey(tier) ? tier : 'standard'

          def toIntSafe = { String raw, int fallback ->
            try {
              return (raw ?: '').trim() ? Integer.parseInt((raw ?: '').trim()) : fallback
            } catch (ignored) {
              return fallback
            }
          }
          def toDoubleSafe = { Object raw, double fallback ->
            try {
              return raw == null ? fallback : Double.parseDouble(raw.toString())
            } catch (ignored) {
              return fallback
            }
          }
          def clampPercent = { double value ->
            if (Double.isNaN(value) || Double.isInfinite(value)) {
              return 0d
            }
            return Math.max(0d, Math.min(100d, value))
          }
          def parseJsonSafe = { String raw ->
            if (!raw?.trim()) {
              return [:]
            }
            try {
              return new JsonSlurperClassic().parseText(raw)
            } catch (ignored) {
              return [:]
            }
          }
          def fetchJson = { String path, String outputFile ->
            def code = sh(
              script: """
                set +x
                curl -sS \
                  -o ${outputFile} \
                  -w "%{http_code}" \
                  -X GET "\${GITGOV_URL%/}${path}" \
                  -H "Authorization: Bearer \${GITGOV_API_KEY}" \
                  -H "Content-Type: application/json"
              """,
              returnStdout: true
            ).trim()
            def raw = fileExists(outputFile) ? readFile(outputFile).trim() : ''
            return [code: code, raw: raw]
          }

          def gateHours = Math.max(1, toIntSafe(env.GITGOV_RELEASE_GATE_HOURS, 168))
          def correlationLimit = Math.max(1, toIntSafe(env.GITGOV_RELEASE_GATE_CORRELATION_LIMIT, 500))
          def minReadiness = Math.max(0, Math.min(100, toIntSafe(env.GITGOV_RELEASE_GATE_MIN, 0)))
          def targetReadiness = minReadiness > 0 ? minReadiness : (profile.target as int)
          def failOnMissingSignals = isEnabledFlag(env.GITGOV_RELEASE_GATE_FAIL_MISSING)

          def warnings = []
          def failReasons = []

          double ticketCoveragePercent = 0d
          int ticketTotalCommits = 0
          double pipelineSuccessRate = 0d
          int pipelineTotal = 0
          double sonarPassRate = 0d
          int sonarTotal = 0

          def ticketCoveragePath = "/integrations/jira/ticket-coverage?repo_full_name=${repoName}&branch=${branchName}&hours=${gateHours}"
          if ((env.GITGOV_RELEASE_GATE_ORG_NAME ?: '').trim()) {
            ticketCoveragePath += "&org_name=${env.GITGOV_RELEASE_GATE_ORG_NAME.trim()}"
          }
          def ticketResponse = fetchJson(ticketCoveragePath, 'gitgov-release-ticket-coverage.json')
          if (ticketResponse.code == '200') {
            def payload = parseJsonSafe(ticketResponse.raw)
            ticketCoveragePercent = clampPercent(toDoubleSafe(payload.coverage_percentage, 0d))
            ticketTotalCommits = Math.max(0, (int) toDoubleSafe(payload.total_commits, 0d))
          } else {
            warnings << "ticket_coverage_http_${ticketResponse.code}"
          }

          def correlationsPath = "/integrations/jenkins/correlations?repo_full_name=${repoName}&branch=${branchName}&limit=${correlationLimit}&offset=0"
          def corrResponse = fetchJson(correlationsPath, 'gitgov-release-correlations.json')
          if (corrResponse.code == '200') {
            def payload = parseJsonSafe(corrResponse.raw)
            def correlations = (payload.correlations instanceof List) ? payload.correlations : []
            def pipelines = correlations.collect { it?.pipeline }.findAll { it instanceof Map }
            pipelineTotal = pipelines.size()
            if (pipelineTotal > 0) {
              int pipelineSuccess = pipelines.count { ((it.status ?: '').toString().trim().toLowerCase()) == 'success' }
              pipelineSuccessRate = clampPercent((100d * pipelineSuccess) / pipelineTotal)
            } else {
              warnings << 'pipeline_data_empty'
            }

            def sonarRuns = pipelines.findAll { ((it.job_name ?: '').toString().toLowerCase()).contains('sonar') }
            sonarTotal = sonarRuns.size()
            if (sonarTotal > 0) {
              int sonarSuccess = sonarRuns.count { ((it.status ?: '').toString().trim().toLowerCase()) == 'success' }
              sonarPassRate = clampPercent((100d * sonarSuccess) / sonarTotal)
            } else {
              warnings << 'sonar_data_empty'
            }
          } else {
            warnings << "jenkins_correlations_http_${corrResponse.code}"
          }

          def signals = [
            [name: 'pipeline', available: (pipelineTotal > 0), value: pipelineSuccessRate, weight: profile.pipeline],
            [name: 'traceability', available: (ticketTotalCommits > 0), value: ticketCoveragePercent, weight: profile.traceability],
            [name: 'sonar', available: (sonarTotal > 0), value: sonarPassRate, weight: profile.sonar],
          ]
          def activeSignals = signals.findAll { it.available }
          int readinessScore = 0
          if (!activeSignals.isEmpty()) {
            double totalWeight = activeSignals.collect { (it.weight as double) }.sum() as double
            if (totalWeight > 0d) {
              double weighted = activeSignals.collect { (it.value as double) * (it.weight as double) }.sum() as double
              readinessScore = Math.round(weighted / totalWeight) as int
            }
          }

          if (activeSignals.isEmpty()) {
            failReasons << 'no_release_readiness_signals'
          }
          if (failOnMissingSignals && activeSignals.size() < signals.size()) {
            failReasons << 'missing_signals_strict_mode'
          }
          if (!activeSignals.isEmpty() && readinessScore < targetReadiness) {
            failReasons << 'readiness_below_target'
          }

          def coverage = "${activeSignals.size()}/${signals.size()}"
          def outcome = failReasons.isEmpty() ? 'PASS' : (gitgovStrictModeEnabled() ? 'FAIL' : 'WARN')

          persistReleaseMeta([
            status: outcome,
            score: readinessScore.toString(),
            target: targetReadiness.toString(),
            coverage: coverage,
            tier: tier,
            reasons: failReasons.join(';'),
            warnings: warnings.join(';'),
          ])

          echo "Release readiness gate => status=${outcome}, score=${readinessScore}, target=${targetReadiness}, coverage=${coverage}, tier=${tier}"
          if (!warnings.isEmpty()) {
            echo "Release readiness warnings: ${warnings.join('; ')}"
          }

          if (outcome == 'FAIL') {
            error("Release readiness gate failed: ${failReasons.join('; ')}")
          } else if (outcome == 'WARN') {
            echo "Release readiness gate warning (non-strict): ${failReasons.join('; ')}"
          }
        }
      }
    }

    stage('Build') {
      steps {
        echo 'Reemplaza este stage con tu build real'
      }
    }
  }

  post {
    success {
      script {
        notifyGitGov('success')
      }
    }
    failure {
      script {
        notifyGitGov('failure')
      }
    }
    unstable {
      script {
        notifyGitGov('unstable')
      }
    }
    aborted {
      script {
        notifyGitGov('aborted')
      }
    }
  }
}

def notifyGitGov(String status) {
  def repoName = env.GIT_URL ? env.GIT_URL.replaceFirst('^.*github\\.com[/:]', '').replaceFirst('\\.git$', '') : ''
  def branchName = (env.BRANCH_NAME ?: env.GIT_BRANCH ?: 'unknown').replaceFirst('^origin/', '')
  def commitSha = env.GIT_COMMIT ?: sh(script: 'git rev-parse HEAD', returnStdout: true).trim()
  def durationMs = currentBuild.duration ?: 0
  def sonarMeta = loadSimplePropertiesFile('gitgov-sonar-meta.properties')
  def sonarStatus = (sonarMeta.status ?: env.GITGOV_SONAR_STATUS ?: '').trim()
  def sonarProjectKey = (sonarMeta.project_key ?: env.GITGOV_SONAR_PROJECT_KEY ?: '').trim()
  def sonarHostUrl = (sonarMeta.host_url ?: env.GITGOV_SONAR_HOST_URL ?: '').trim()
  def sonarDashboardUrl = (sonarMeta.dashboard_url ?: env.GITGOV_SONAR_DASHBOARD_URL ?: '').trim()
  def readinessMeta = loadSimplePropertiesFile('gitgov-release-readiness.properties')
  def readinessStatus = (readinessMeta.status ?: '').trim().toUpperCase()
  def readinessScore = (readinessMeta.score ?: '').trim()
  def readinessTarget = (readinessMeta.target ?: '').trim()
  def readinessCoverage = (readinessMeta.signal_coverage ?: '').trim()
  def readinessTier = (readinessMeta.tier ?: '').trim()
  def readinessReasons = (readinessMeta.reasons ?: '').trim()
  def readinessWarnings = (readinessMeta.warnings ?: '').trim()

  def stagesPayload = []
  if (sonarStatus && sonarStatus != 'NOT_RUN') {
    stagesPayload << [
      name       : 'quality_gate',
      status     : sonarStatus,
      duration_ms: null,
      metadata   : [
        provider   : 'sonarqube',
        project_key: sonarProjectKey,
        host_url   : sonarHostUrl,
      ],
    ]
  }

  if (readinessStatus && !['NOT_RUN', 'SKIPPED'].contains(readinessStatus)) {
    stagesPayload << [
      name       : 'release_readiness',
      status     : readinessStatus,
      duration_ms: null,
      metadata   : [
        score          : readinessScore,
        target         : readinessTarget,
        signal_coverage: readinessCoverage,
        tier           : readinessTier,
        reasons        : readinessReasons,
        warnings       : readinessWarnings,
      ],
    ]
  }

  def artifactsPayload = []
  if (sonarDashboardUrl) {
    artifactsPayload << sonarDashboardUrl
  }

  def payload = JsonOutput.toJson([
    pipeline_id   : "${env.JOB_NAME ?: 'unknown'}#${env.BUILD_NUMBER ?: '0'}",
    job_name      : (env.JOB_NAME ?: 'unknown'),
    status        : status,
    commit_sha    : commitSha,
    branch        : branchName,
    repo_full_name: repoName,
    duration_ms   : durationMs,
    triggered_by  : (env.BUILD_USER_ID ?: env.BUILD_USER ?: 'jenkins'),
    stages        : stagesPayload,
    artifacts     : artifactsPayload,
    timestamp     : System.currentTimeMillis(),
  ])

  writeFile file: 'gitgov-pipeline-event.json', text: payload

  def publishStatus = sh(
    script: '''
      set +x
      if [ -n "${GITGOV_JENKINS_SECRET:-}" ] && [ "${GITGOV_JENKINS_SECRET}" != "unused" ]; then
        curl --fail-with-body -sS -X POST "${GITGOV_URL%/}/integrations/jenkins" \
          -H "Authorization: Bearer ${GITGOV_API_KEY}" \
          -H "Content-Type: application/json" \
          -H "x-gitgov-jenkins-secret: ${GITGOV_JENKINS_SECRET}" \
          --data @gitgov-pipeline-event.json
      else
        curl --fail-with-body -sS -X POST "${GITGOV_URL%/}/integrations/jenkins" \
          -H "Authorization: Bearer ${GITGOV_API_KEY}" \
          -H "Content-Type: application/json" \
          --data @gitgov-pipeline-event.json
      fi
    ''',
    returnStatus: true
  )
  if (publishStatus != 0) {
    def msg = "GitGov telemetry publish failed (exit=${publishStatus})"
    if (gitgovStrictModeEnabled()) {
      error("${msg}; aborting because GITGOV_STRICT=true")
    }
    echo "${msg}; continuing because GITGOV_STRICT=false"
  }
}

def isEnabledFlag(String raw) {
  def value = (raw ?: '').trim().toLowerCase()
  return ['1', 'true', 'yes', 'on'].contains(value)
}

def gitgovStrictModeEnabled() {
  return isEnabledFlag(env.GITGOV_STRICT ?: 'false')
}

def ensureSonarScannerBinary() {
  return sh(
    script: '''
      set -euo pipefail

      if command -v sonar-scanner >/dev/null 2>&1; then
        command -v sonar-scanner
        exit 0
      fi

      SCANNER_VERSION="6.2.1.4610"
      BASE_DIR="$WORKSPACE/.tools/sonar-scanner"
      TARGET_DIR="$BASE_DIR/sonar-scanner-${SCANNER_VERSION}-linux-x64"
      TARGET_BIN="$TARGET_DIR/bin/sonar-scanner"

      if [ ! -x "$TARGET_BIN" ]; then
        mkdir -p "$BASE_DIR"
        curl -fsSL -o "$BASE_DIR/sonar-scanner.zip" "https://binaries.sonarsource.com/Distribution/sonar-scanner-cli/sonar-scanner-cli-${SCANNER_VERSION}-linux-x64.zip"
        rm -rf "$BASE_DIR"/sonar-scanner-*-linux-x64

        if command -v unzip >/dev/null 2>&1; then
          unzip -qo "$BASE_DIR/sonar-scanner.zip" -d "$BASE_DIR"
        elif command -v python3 >/dev/null 2>&1; then
          python3 - <<'PY'
import zipfile
zipfile.ZipFile('.tools/sonar-scanner/sonar-scanner.zip').extractall('.tools/sonar-scanner')
PY
        else
          echo "ERROR: unzip or python3 is required to extract sonar-scanner.zip" >&2
          exit 87
        fi
      fi

      if [ ! -x "$TARGET_BIN" ]; then
        echo "ERROR: sonar-scanner binary not found after bootstrap" >&2
        exit 88
      fi

      echo "$TARGET_BIN"
    ''',
    returnStdout: true
  ).trim()
}

def inferSonarProjectKey() {
  def repoName = env.GIT_URL
    ? env.GIT_URL.replaceFirst('^.*github\\.com[/:]', '').replaceFirst('\\.git$', '')
    : (env.JOB_NAME ?: 'gitgov')
  return repoName
    .toLowerCase()
    .replaceAll('[^a-z0-9._-]+', '_')
}

def extractJsonString(String raw, String key) {
  if (!raw) {
    return null
  }
  def matcher = (raw =~ /"${key}"\\s*:\\s*"((?:\\\\.|[^"\\\\])*)"/)
  if (!matcher.find()) {
    return null
  }
  return matcher.group(1).replace('\\"', '"').replace('\\\\', '\\')
}

def extractJsonBoolean(String raw, String key, boolean defaultValue) {
  if (!raw) {
    return defaultValue
  }
  def matcher = (raw =~ /"${key}"\\s*:\\s*(true|false)/)
  if (!matcher.find()) {
    return defaultValue
  }
  return matcher.group(1) == 'true'
}

def extractJsonStringArray(String raw, String key) {
  if (!raw) {
    return []
  }
  def arrayMatcher = (raw =~ /"${key}"\\s*:\\s*\\[(.*?)\\]/)
  if (!arrayMatcher.find()) {
    return []
  }
  def inner = arrayMatcher.group(1)
  if (!inner?.trim()) {
    return []
  }
  def out = []
  def valueMatcher = (inner =~ /"((?:\\\\.|[^"\\\\])*)"/)
  while (valueMatcher.find()) {
    out << valueMatcher.group(1).replace('\\"', '"').replace('\\\\', '\\')
  }
  return out
}

def extractJsonObjectField(String raw, String objectName, String fieldName) {
  if (!raw) {
    return null
  }
  def objectMarker = "\"${objectName}\""
  def objectStart = raw.indexOf(objectMarker)
  if (objectStart < 0) {
    return null
  }

  def fieldMarker = "\"${fieldName}\""
  def fieldStart = raw.indexOf(fieldMarker, objectStart)
  if (fieldStart < 0) {
    return null
  }

  def colon = raw.indexOf(':', fieldStart + fieldMarker.length())
  if (colon < 0) {
    return null
  }

  def valueStart = raw.indexOf('"', colon + 1)
  if (valueStart < 0) {
    return null
  }

  def valueEnd = raw.indexOf('"', valueStart + 1)
  if (valueEnd < 0) {
    return null
  }

  return raw.substring(valueStart + 1, valueEnd)
}

def loadSimplePropertiesFile(String path) {
  if (!fileExists(path)) {
    return [:]
  }
  def out = [:]
  readFile(path).split(/\r?\n/).each { line ->
    def clean = line?.trim()
    if (!clean || clean.startsWith('#') || !clean.contains('=')) {
      return
    }
    int idx = clean.indexOf('=')
    if (idx <= 0) {
      return
    }
    def key = clean.substring(0, idx).trim()
    def value = clean.substring(idx + 1).trim()
    out[key] = value
  }
  return out
}
