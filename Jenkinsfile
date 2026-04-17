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
            script: """
              curl -sS \
                -o gitgov-policy-check-response.json \
                -w "%{http_code}" \
                -X POST ${GITGOV_URL}/policy/check \
                -H "Authorization: Bearer ${GITGOV_API_KEY}" \
                -H "Content-Type: application/json" \
                --data @gitgov-policy-check.json
            """,
            returnStdout: true
          ).trim()

          def responseRaw = fileExists('gitgov-policy-check-response.json')
            ? readFile('gitgov-policy-check-response.json').trim()
            : ''

          def response = [:]
          if (responseRaw) {
            try {
              response = new JsonSlurperClassic().parseText(responseRaw) as Map
            } catch (ignored) {
              response = [raw: responseRaw]
            }
          }

          if (!(policyHttpCode in ['200', '409'])) {
            def msg = "GitGov policy/check transport failed (http=${policyHttpCode})"
            if (gitgovStrictModeEnabled()) {
              error("${msg}; aborting because GITGOV_STRICT=true")
            }
            echo "${msg}; continuing because GITGOV_STRICT=false"
            return
          }

          def reasons = (response.reasons instanceof List) ? response.reasons.join('; ') : ''
          def warnings = (response.warnings instanceof List) ? response.warnings.join('; ') : ''
          def allowed = response.allowed == true
          def advisory = response.advisory != false
          def enforcementApplied = response.enforcement_applied ?: 'unknown'

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
  def payload = JsonOutput.toJson([
    pipeline_id   : "${env.JOB_NAME ?: 'unknown'}#${env.BUILD_NUMBER ?: '0'}",
    job_name      : (env.JOB_NAME ?: 'unknown'),
    status        : status,
    commit_sha    : commitSha,
    branch        : branchName,
    repo_full_name: repoName,
    duration_ms   : durationMs,
    triggered_by  : (env.BUILD_USER_ID ?: env.BUILD_USER ?: 'jenkins'),
    stages        : [],
    artifacts     : [],
    timestamp     : System.currentTimeMillis(),
  ])

  writeFile file: 'gitgov-pipeline-event.json', text: payload

  def includeSecret = env.GITGOV_JENKINS_SECRET?.trim() && env.GITGOV_JENKINS_SECRET.trim() != 'unused'
  def secretHeader = includeSecret ? "-H \"x-gitgov-jenkins-secret: ${env.GITGOV_JENKINS_SECRET}\"" : ""

  def publishStatus = sh(
    script: """
      curl --fail-with-body -sS -X POST ${env.GITGOV_URL}/integrations/jenkins \
        -H "Authorization: Bearer ${env.GITGOV_API_KEY}" \
        -H "Content-Type: application/json" \
        ${secretHeader} \
        --data @gitgov-pipeline-event.json
    """,
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
