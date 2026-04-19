import groovy.json.JsonOutput

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
          env.GITGOV_SONAR_STATUS = 'SKIPPED'
          env.GITGOV_SONAR_PROJECT_KEY = (env.SONAR_PROJECT_KEY ?: '').trim()
          env.GITGOV_SONAR_HOST_URL = (env.SONAR_HOST_URL ?: 'http://host.docker.internal:9000').trim()
          env.GITGOV_SONAR_DASHBOARD_URL = ''

          def persistSonarMeta = {
            def statusValue = (env.GITGOV_SONAR_STATUS ?: '').replace('\n', ' ').replace('\r', ' ')
            def projectKeyValue = (env.GITGOV_SONAR_PROJECT_KEY ?: '').replace('\n', ' ').replace('\r', ' ')
            def hostUrlValue = (env.GITGOV_SONAR_HOST_URL ?: '').replace('\n', ' ').replace('\r', ' ')
            def dashboardValue = (env.GITGOV_SONAR_DASHBOARD_URL ?: '').replace('\n', ' ').replace('\r', ' ')
            writeFile file: 'gitgov-sonar-meta.properties', text: """status=${statusValue}
project_key=${projectKeyValue}
host_url=${hostUrlValue}
dashboard_url=${dashboardValue}
"""
          }

          def sonarToken = (env.SONAR_TOKEN ?: '').trim()
          def sonarProjectKey = (env.SONAR_PROJECT_KEY ?: '').trim()
          if (!sonarToken || !sonarProjectKey) {
            echo 'Skipping Sonar scan (missing SONAR_TOKEN or SONAR_PROJECT_KEY).'
            persistSonarMeta()
            return
          }

          try {
            def scannerBin = ensureSonarScannerBinary()
            withEnv(["SONAR_SCANNER_BIN=${scannerBin}"]) {
              def scanStatus = sh(
                script: '''
                  set -euo pipefail
                  "${SONAR_SCANNER_BIN}" \
                    -Dsonar.projectKey="${SONAR_PROJECT_KEY}" \
                    -Dsonar.projectName="GitGov" \
                    -Dsonar.sources=gitgov/gitgov-server/src,gitgov/src,gitgov/src-tauri/src,gitgov-web/src \
                    -Dsonar.exclusions=**/node_modules/**,**/target/**,**/dist/**,**/.next/**,**/coverage/**,**/public/**,**/*.min.js \
                    -Dsonar.sourceEncoding=UTF-8 \
                    -Dsonar.scm.provider=git \
                    -Dsonar.host.url="${GITGOV_SONAR_HOST_URL}" \
                    -Dsonar.token="${SONAR_TOKEN}"
                ''',
                returnStatus: true
              )
              if (scanStatus != 0) {
                env.GITGOV_SONAR_STATUS = 'SCAN_FAILED'
                def msg = "Sonar scanner returned non-zero exit (${scanStatus})"
                if (gitgovStrictModeEnabled()) {
                  error("${msg}; aborting because GITGOV_STRICT=true")
                }
                echo "${msg}; continuing because GITGOV_STRICT=false"
                return
              }
            }

            if (!fileExists('.scannerwork/report-task.txt')) {
              env.GITGOV_SONAR_STATUS = 'UNKNOWN'
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
              env.GITGOV_SONAR_DASHBOARD_URL = dashboardUrl
            }
            if (serverUrl) {
              env.GITGOV_SONAR_HOST_URL = serverUrl
            }

            if (!ceTaskId) {
              env.GITGOV_SONAR_STATUS = 'UNKNOWN'
              echo 'Sonar CE task id is empty; unable to resolve quality gate.'
              return
            }

            def ceStatus = 'PENDING'
            def analysisId = ''
            withEnv([
              "SQ_HOST_URL=${env.GITGOV_SONAR_HOST_URL}",
              "SQ_CE_TASK_ID=${ceTaskId}"
            ]) {
              for (int i = 0; i < 60; i++) {
                def ceTaskRaw = sh(
                  script: '''
                    set +x
                    curl -fsS -u "${SONAR_TOKEN}:" "${SQ_HOST_URL%/}/api/ce/task?id=${SQ_CE_TASK_ID}"
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
              env.GITGOV_SONAR_STATUS = (ceStatus == 'PENDING') ? 'TIMEOUT' : ceStatus
              echo "Sonar analysis id unavailable (ce_status=${env.GITGOV_SONAR_STATUS})."
              return
            }

            withEnv([
              "SQ_HOST_URL=${env.GITGOV_SONAR_HOST_URL}",
              "SQ_ANALYSIS_ID=${analysisId}"
            ]) {
              def gateRaw = sh(
                script: '''
                  set +x
                  curl -fsS -u "${SONAR_TOKEN}:" "${SQ_HOST_URL%/}/api/qualitygates/project_status?analysisId=${SQ_ANALYSIS_ID}"
                ''',
                  returnStdout: true
                ).trim()
              writeFile file: 'sonar-quality-gate.json', text: gateRaw
              env.GITGOV_SONAR_STATUS = (extractJsonObjectField(gateRaw, 'projectStatus', 'status') ?: 'UNKNOWN').toUpperCase()
            }

            echo "Sonar quality gate status: ${env.GITGOV_SONAR_STATUS}"
          } catch (err) {
            env.GITGOV_SONAR_STATUS = 'SCAN_FAILED'
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

  def artifactsPayload = []
  if (sonarDashboardUrl) {
    artifactsPayload << [
      name: 'sonar_dashboard',
      type: 'url',
      url : sonarDashboardUrl,
    ]
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
      args=(
        --fail-with-body
        -sS
        -X POST
        "${GITGOV_URL%/}/integrations/jenkins"
        -H "Authorization: Bearer ${GITGOV_API_KEY}"
        -H "Content-Type: application/json"
        --data @gitgov-pipeline-event.json
      )
      if [ -n "${GITGOV_JENKINS_SECRET:-}" ] && [ "${GITGOV_JENKINS_SECRET}" != "unused" ]; then
        args+=( -H "x-gitgov-jenkins-secret: ${GITGOV_JENKINS_SECRET}" )
      fi
      curl "${args[@]}"
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

def gitgovStrictModeEnabled() {
  def raw = (env.GITGOV_STRICT ?: 'false').trim().toLowerCase()
  return ['1', 'true', 'yes', 'on'].contains(raw)
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
  def objectMatcher = (raw =~ /"${objectName}"\\s*:\\s*\\{([\\s\\S]*?)\\}/)
  if (!objectMatcher.find()) {
    return null
  }
  def objectBody = objectMatcher.group(1)
  def fieldMatcher = (objectBody =~ /"${fieldName}"\\s*:\\s*"((?:\\\\.|[^"\\\\])*)"/)
  if (!fieldMatcher.find()) {
    return null
  }
  return fieldMatcher.group(1).replace('\\"', '"').replace('\\\\', '\\')
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
