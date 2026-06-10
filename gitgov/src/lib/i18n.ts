import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'

export type AppLanguage = 'en' | 'es'

export const DEFAULT_APP_LANGUAGE: AppLanguage = 'en'
export const APP_LANGUAGE_STORAGE_KEY = 'gitgov.content_language_v1'

export const LANGUAGE_OPTIONS: Array<{ value: AppLanguage; label: string; nativeLabel: string }> = [
  { value: 'en', label: 'English', nativeLabel: 'English' },
  { value: 'es', label: 'Spanish', nativeLabel: 'Español' },
]

const resources = {
  en: {
    translation: {
      common: {
        cancel: 'Cancel',
        close: 'Close',
        continue: 'Continue',
        notConfigured: 'Not configured',
        notSelected: 'Not selected',
        noRole: 'No role',
        ready: 'Ready',
        checking: 'Checking',
        disconnected: 'Disconnected',
        connected: 'Connected',
        maintenance: 'Maintenance',
      },
      navigation: {
        home: 'Workspace',
        actionCenter: 'Action Center',
        governance: 'Governance',
        audit: 'Audit',
        settings: 'Settings',
        help: 'Help / FAQ',
        switchUser: 'Switch user (sign out)',
      },
      language: {
        title: 'Content language',
        prompt: 'Choose the language for GitGov content.',
        helper: 'You can change this later in Settings.',
        active: 'Active language: {{language}}',
        pending: 'Updating...',
        optionSubtitles: {
          en: 'English',
          es: 'Spanish',
        },
      },
      login: {
        tagline: 'Git workflow control with roles and audit evidence',
        desktopRequiredTitle: 'GitGov Desktop required',
        desktopRequiredBody: 'GitHub authentication is only available in the desktop application.',
        desktopStepsLabel: 'To use all GitGov features:',
        desktopStepDownload: 'Download GitGov Desktop',
        desktopStepInstall: 'Install the application',
        desktopStepOpen: 'Open the app to authenticate',
        downloadDesktop: 'Download GitGov Desktop',
        connectPrompt: 'Connect your GitHub account to identify your local actions.',
        sessionReuse: 'GitGov reuses the secure local session when available. It only asks for GitHub again if the token is missing, expired, or you switch users.',
        connectGitHub: 'Connect with GitHub',
        deviceInstruction: 'Go to GitHub, enter this code, and authorize GitGov:',
        copied: 'Copied',
        copyCode: 'Click to copy',
        openGitHub: 'Open GitHub',
        connectingTitle: 'Connecting to GitHub...',
        connectingBody: 'Validating authorization. If GitHub does not confirm in time, you will return to the code with a retry option.',
      },
      settings: {
        title: 'Settings',
        body: 'Configure Desktop preferences, Control Plane connection, organization access, updates, account security, and repository context.',
        tabs: {
          preferences: {
            label: 'Preferences',
            description: 'Language, timezone, and desktop notifications.',
          },
          connection: {
            label: 'System',
            description: 'Control Plane connection, transport, and Desktop updates.',
          },
          organization: {
            label: 'Organization',
            description: 'Admin onboarding, team access, API keys, and rules.',
          },
          account: {
            label: 'Account',
            description: 'GitHub session, Control Plane role, and local PIN.',
          },
          repository: {
            label: 'Repository',
            description: 'Local repository path and GitGov config preview.',
          },
        },
        languageSectionTitle: 'Language',
        languageSectionBody: 'Select the language used for onboarding, guidance, and configurable product content.',
        timezone: {
          title: 'Audit Trail timezone',
          body: 'Events are stored in UTC in the database. Select the local timezone used to display timestamps correctly in legal audits.',
          field: 'Display timezone',
          autoDetect: 'Auto-detect from system',
          active: 'Active:',
        },
        connection: {
          title: 'Control Plane',
          body: 'Configure the endpoint, API key, role, organization scope, and transport used by GitGov Desktop. Operational governance views live in Governance.',
          endpoint: 'Endpoint',
          role: 'Role',
          scope: 'Scope',
          transport: 'Transport',
          liveStream: 'live stream',
          httpConnected: 'HTTP connected',
        },
        outbox: {
          title: 'Desktop outbox',
          body: 'Local audit events wait here while the Control Plane is unavailable. Terminal failures move to dead-letter and stop retrying.',
          refresh: 'Refresh',
          pending: 'Pending',
          scheduled: 'Backoff',
          deadLetter: 'Dead-letter',
          maxAttempts: 'Max attempts',
          nextRetry: 'Next retry:',
          lastDeadLetter: 'Last dead-letter:',
          none: 'None',
        },
        notifications: {
          title: 'Desktop notifications',
          body: 'Receive native operating system alerts when relevant events happen in GitGov.',
          enable: 'Enable notifications',
          master: 'Master switch - disables every notification',
          newEvents: 'New Control Plane events',
          blockedPush: 'Blocked push (protected branch or governance)',
          governanceWarn: 'Governance warnings (warn mode)',
        },
        organization: {
          title: 'Organization administration',
          body: 'Admin onboarding, team management, and API keys are managed from Settings. JSON export stays outside this view.',
          connectFirst: 'Connect to the Control Plane first to administer organization and role-based access.',
          configureControlPlane: 'Configure Control Plane',
          adminRequired: 'The current user can use Desktop and visible surfaces, but organization administration requires the Admin role.',
          policyMovedTitle: 'Governance policy lives in Governance',
          policyMovedBody: 'Rules for {{repo}} are edited from Governance > Policy so compliance decisions have one owner.',
          openGovernancePolicy: 'Open Governance Policy',
        },
        updates: {
          title: 'Desktop updates',
          channel: 'Update channel',
          stableTitle: 'Recommended channel for end users',
          betaTitle: 'Beta channel for internal testing',
          activeChannel: 'Active channel:',
          status: 'Updater status',
          unsupported: 'In-app updater is not available outside Tauri Desktop.',
          notConfigured: 'Updater is not configured (signed endpoint/pubkey missing).',
          mandatory: 'This version requires a mandatory update.',
          updateAvailable: 'New version available: {{version}}',
          installed: 'Update installed. Restart GitGov to apply changes.',
          downloading: 'Downloading update...',
          checking: 'Checking for updates...',
          noUpdate: 'GitGov is up to date.',
          idle: 'Ready to check for updates.',
          lastChecked: 'Last check:',
          telemetry: 'Checks: {{checks}} - With update: {{withUpdate}} - Downloads: {{downloads}} - Installed: {{installed}} - Failed: {{failed}}',
          lastOutcome: 'Last result:',
          mandatoryActive: 'Mandatory update active.',
          minimumSupported: 'Minimum supported: v{{version}}.',
          configHint: 'Configure `plugins.updater` in `tauri.conf.json` with endpoint(s) and signing pubkey to activate in-app updates.',
          current: 'Current:',
          hideChangelog: 'Hide changelog',
          showChangelog: 'View changelog',
          downloadInstall: 'Download and install',
          retryDownload: 'Retry download',
          downloadedKb: '{{kb}} KB downloaded',
          preparingDownload: 'Preparing download...',
          changelog: 'Changelog',
          noChangelog: 'No changelog in this release.',
          check: 'Check for updates',
          manualDownload: 'Manual download',
          fallbackTitle: 'Fallback if the updater is not configured or fails',
        },
        account: {
          title: 'Session',
          admin: 'Administrator',
          controlPlane: 'Control Plane',
          roleSeparation: 'GitHub login and Control Plane role are independent.',
          currentRole: 'Current role:',
          signOut: 'Sign out',
          switchUser: 'Switch user',
          pinTitle: 'Local PIN (optional)',
          pinBody: 'Protect local access to the app on this machine. It does not replace server authentication.',
          newPin: 'New PIN (4-6 digits)',
          pinPlaceholder: 'PIN (4-6 digits)',
          updatePin: 'Update PIN',
          enablePin: 'Enable PIN',
          disablePin: 'Disable PIN',
          lockNow: 'Lock now',
        },
        repository: {
          title: 'Repository',
          currentPath: 'Current path',
          noneSelected: 'Not selected',
          change: 'Change repository',
          configTitle: 'GitGov configuration',
          modalTitle: 'Change repository',
          modalBody: 'Select a new repository to manage',
          modalAction: 'Select from the main selector',
        },
      },
      serverConfig: {
        maintenanceTitle: 'Server in maintenance',
        serverUrl: 'Server URL',
        identity: 'Control Plane identity',
        adminRequired: 'Authenticated as {{role}}. Use an Admin API key for admin operations.',
        maintenanceBody: 'The server is updating. Reconnecting every 10 seconds...',
        connectedTitle: 'Connected to Control Plane',
        disconnect: 'Disconnect',
        connectTitle: 'Connect to Control Plane',
        urlHint: 'Use localhost only when the local Control Plane is running; otherwise use the configured server URL.',
        apiKey: 'API Key (optional)',
        apiKeyPlaceholder: 'Your API key',
        connect: 'Connect',
      },
      governance: {
        title: 'Governance',
        body: 'Governance is organized by evidence, policy, adoption, release decisions, and citation-grounded explanation. Control Plane connection settings live in Settings.',
        accessTitle: 'Admin workspace required',
        accessBody: 'This governance tool is admin-scoped. The current role can still use the local Workspace and visible audit surfaces without opening privileged admin editors.',
        sections: {
          evidence: {
            label: 'Evidence',
            description: 'Traceability, pipeline evidence, packets, telemetry, and exports.',
          },
          policy: {
            label: 'Policy',
            description: 'Branch, review, traceability, and enforcement rules.',
          },
          adoption: {
            label: 'Adoption',
            description: 'Enterprise setup, providers, workflows, and onboarding tasks.',
          },
          releases: {
            label: 'Releases',
            description: 'Approval decisions, evidence hashes, and governance evaluation.',
          },
          copilot: {
            label: 'Copilot',
            description: 'Citation-grounded governance explanation.',
          },
        },
        metrics: {
          traceability: 'Traceability',
          traceabilityDetail: '{{withTicket}}/{{total}} commits linked to ticket evidence',
          pipelineEvidence: 'Pipeline evidence',
          pipelineDetail: '{{total}} run(s), {{failures}} failing in 7d',
          githubSignals: 'GitHub signals',
          githubSignalsReady: 'PR, review, comment, and status evidence observed',
          githubSignalsMissing: 'Missing: {{signals}}',
          evidenceGaps: 'Evidence gaps',
          evidenceGapsDetail: '{{traceability}} traceability gap(s), {{blocked}} blocked push(es) today, {{critical}} critical violation(s)',
          releaseReadiness: 'Release readiness',
          releaseReadinessDetail: '{{band}}; standard target {{target}}',
          qualityEvidence: 'Quality evidence',
          qualityEvidenceDetail: '{{passed}}/{{total}} Sonar/quality pipeline(s) passed',
          releaseBlockers: 'Release blockers',
          releaseBlockersDetail: '{{critical}} critical violation(s), {{failures}} failing pipeline(s)',
        },
      },
    },
  },
  es: {
    translation: {
      common: {
        cancel: 'Cancelar',
        close: 'Cerrar',
        continue: 'Continuar',
        notConfigured: 'No configurado',
        notSelected: 'No seleccionado',
        noRole: 'sin rol',
        ready: 'Listo',
        checking: 'Verificando',
        disconnected: 'Desconectado',
        connected: 'Conectado',
        maintenance: 'Mantenimiento',
      },
      navigation: {
        home: 'Workspace',
        actionCenter: 'Action Center',
        governance: 'Gobernanza',
        audit: 'Auditoría',
        settings: 'Configuración',
        help: 'Ayuda / FAQ',
        switchUser: 'Cambiar usuario (cerrar sesión)',
      },
      language: {
        title: 'Idioma del contenido',
        prompt: 'Elige el idioma para el contenido de GitGov.',
        helper: 'Puedes cambiarlo luego en Configuración.',
        active: 'Idioma activo: {{language}}',
        pending: 'Actualizando...',
        optionSubtitles: {
          en: 'Inglés',
          es: 'Español',
        },
      },
      login: {
        tagline: 'Control de flujo Git con roles y auditoría',
        desktopRequiredTitle: 'Requiere GitGov Desktop',
        desktopRequiredBody: 'La autenticación con GitHub solo está disponible en la aplicación desktop.',
        desktopStepsLabel: 'Para usar todas las funciones de GitGov:',
        desktopStepDownload: 'Descarga GitGov Desktop',
        desktopStepInstall: 'Instala la aplicación',
        desktopStepOpen: 'Abre la aplicación para autenticarte',
        downloadDesktop: 'Descargar GitGov Desktop',
        connectPrompt: 'Conecta tu cuenta de GitHub para identificar tus acciones locales.',
        sessionReuse: 'GitGov reutiliza la sesión local segura cuando está disponible. Solo pedirá GitHub de nuevo si el token falta, expira o cambias de usuario.',
        connectGitHub: 'Conectar con GitHub',
        deviceInstruction: 'Ve a GitHub, ingresa este código y autoriza GitGov:',
        copied: 'Copiado',
        copyCode: 'Click para copiar',
        openGitHub: 'Abrir GitHub',
        connectingTitle: 'Conectando con GitHub...',
        connectingBody: 'Validando la autorización. Si GitHub no confirma a tiempo, volverás al código con una opción de reintento.',
      },
      settings: {
        title: 'Configuración',
        body: 'Configura preferencias de Desktop, conexión al Control Plane, acceso de organización, actualizaciones, seguridad de cuenta y contexto del repositorio.',
        tabs: {
          preferences: {
            label: 'Preferencias',
            description: 'Idioma, zona horaria y notificaciones de escritorio.',
          },
          connection: {
            label: 'Sistema',
            description: 'Conexión al Control Plane, transporte y updates de Desktop.',
          },
          organization: {
            label: 'Organización',
            description: 'Onboarding admin, acceso del equipo, API keys y reglas.',
          },
          account: {
            label: 'Cuenta',
            description: 'Sesión GitHub, rol del Control Plane y PIN local.',
          },
          repository: {
            label: 'Repositorio',
            description: 'Ruta local y vista previa de configuración GitGov.',
          },
        },
        languageSectionTitle: 'Idioma',
        languageSectionBody: 'Selecciona el idioma usado en onboarding, guías y contenido configurable del producto.',
        timezone: {
          title: 'Zona horaria del Audit Trail',
          body: 'Los eventos se almacenan en UTC en la base de datos. Selecciona la zona horaria local para mostrar los timestamps correctamente en auditorías legales.',
          field: 'Zona horaria de visualización',
          autoDetect: 'Auto-detectar del sistema',
          active: 'Activa:',
        },
        connection: {
          title: 'Control Plane',
          body: 'Configura el endpoint, API key, rol, alcance de organización y transporte que usa GitGov Desktop. Las vistas operativas de gobernanza viven en Governance.',
          endpoint: 'Endpoint',
          role: 'Rol',
          scope: 'Alcance',
          transport: 'Transporte',
          liveStream: 'live stream',
          httpConnected: 'HTTP conectado',
        },
        outbox: {
          title: 'Outbox Desktop',
          body: 'Los eventos locales de auditoría esperan aquí cuando el Control Plane no está disponible. Los fallos terminales pasan a dead-letter y dejan de reintentarse.',
          refresh: 'Refrescar',
          pending: 'Pendientes',
          scheduled: 'Backoff',
          deadLetter: 'Dead-letter',
          maxAttempts: 'Máx. intentos',
          nextRetry: 'Próximo reintento:',
          lastDeadLetter: 'Último dead-letter:',
          none: 'Ninguno',
        },
        notifications: {
          title: 'Notificaciones de escritorio',
          body: 'Recibe alertas nativas del sistema operativo cuando ocurren eventos relevantes en GitGov.',
          enable: 'Activar notificaciones',
          master: 'Interruptor principal - desactiva todas las notificaciones',
          newEvents: 'Nuevos eventos en el Control Plane',
          blockedPush: 'Push bloqueado (rama protegida o gobernanza)',
          governanceWarn: 'Advertencias de gobernanza (modo warn)',
        },
        organization: {
          title: 'Administración de organización',
          body: 'Onboarding admin, gestión de equipo y API keys se gestionan desde Settings. Export JSON se mantiene fuera de esta vista.',
          connectFirst: 'Conecta primero al Control Plane para administrar organización y acceso por rol.',
          configureControlPlane: 'Configurar Control Plane',
          adminRequired: 'El usuario actual puede usar Desktop y las superficies visibles, pero la administración de organización requiere rol Admin.',
          policyMovedTitle: 'La política vive en Gobernanza',
          policyMovedBody: 'Las reglas de {{repo}} se editan desde Gobernanza > Políticas para que las decisiones de compliance tengan un solo dueño.',
          openGovernancePolicy: 'Abrir políticas',
        },
        updates: {
          title: 'Actualizaciones Desktop',
          channel: 'Canal de actualizaciones',
          stableTitle: 'Canal recomendado para usuarios finales',
          betaTitle: 'Canal beta para pruebas internas',
          activeChannel: 'Canal activo:',
          status: 'Estado del updater',
          unsupported: 'Updater in-app no disponible fuera de Tauri Desktop.',
          notConfigured: 'Updater no configurado (faltan endpoint/pubkey firmados).',
          mandatory: 'Esta versión requiere actualización obligatoria.',
          updateAvailable: 'Nueva versión disponible: {{version}}',
          installed: 'Update instalado. Reinicia GitGov para aplicar cambios.',
          downloading: 'Descargando actualización...',
          checking: 'Buscando actualizaciones...',
          noUpdate: 'GitGov está actualizado.',
          idle: 'Listo para verificar actualizaciones.',
          lastChecked: 'Última verificación:',
          telemetry: 'Checks: {{checks}} - Con update: {{withUpdate}} - Descargas: {{downloads}} - Instaladas: {{installed}} - Fallidas: {{failed}}',
          lastOutcome: 'Último resultado:',
          mandatoryActive: 'Update obligatorio activo.',
          minimumSupported: 'Mínimo soportado: v{{version}}.',
          configHint: 'Configura `plugins.updater` en `tauri.conf.json` con endpoint(s) y pubkey de firma para activar updates in-app.',
          current: 'Actual:',
          hideChangelog: 'Ocultar changelog',
          showChangelog: 'Ver changelog',
          downloadInstall: 'Descargar e instalar',
          retryDownload: 'Reintentar descarga',
          downloadedKb: '{{kb}} KB descargados',
          preparingDownload: 'Preparando descarga...',
          changelog: 'Changelog',
          noChangelog: 'Sin changelog en esta release.',
          check: 'Buscar actualizaciones',
          manualDownload: 'Descarga manual',
          fallbackTitle: 'Fallback si el updater no está configurado o falla',
        },
        account: {
          title: 'Sesión',
          admin: 'Administrador',
          controlPlane: 'Control Plane',
          roleSeparation: 'Login GitHub y rol Control Plane son independientes.',
          currentRole: 'Rol actual:',
          signOut: 'Cerrar sesión',
          switchUser: 'Cambiar usuario',
          pinTitle: 'PIN local (opcional)',
          pinBody: 'Protege el acceso local a la app en esta máquina. No reemplaza autenticación de servidor.',
          newPin: 'Nuevo PIN (4-6 dígitos)',
          pinPlaceholder: 'PIN (4-6 dígitos)',
          updatePin: 'Actualizar PIN',
          enablePin: 'Activar PIN',
          disablePin: 'Desactivar PIN',
          lockNow: 'Bloquear ahora',
        },
        repository: {
          title: 'Repositorio',
          currentPath: 'Ruta actual',
          noneSelected: 'No seleccionado',
          change: 'Cambiar repositorio',
          configTitle: 'Configuración GitGov',
          modalTitle: 'Cambiar repositorio',
          modalBody: 'Selecciona un nuevo repositorio para gestionar',
          modalAction: 'Seleccionar desde el selector principal',
        },
      },
      serverConfig: {
        maintenanceTitle: 'Servidor en mantenimiento',
        serverUrl: 'URL del servidor',
        identity: 'Identidad Control Plane',
        adminRequired: 'Autenticado como {{role}}. Usa una API key Admin para operaciones administrativas.',
        maintenanceBody: 'El servidor se está actualizando. Reconectando cada 10 segundos...',
        connectedTitle: 'Conectado al Control Plane',
        disconnect: 'Desconectar',
        connectTitle: 'Conectar al Control Plane',
        urlHint: 'Usa localhost solo si el Control Plane local está levantado; de lo contrario usa la URL configurada del servidor.',
        apiKey: 'API Key (opcional)',
        apiKeyPlaceholder: 'Tu API key',
        connect: 'Conectar',
      },
      governance: {
        title: 'Gobernanza',
        body: 'La gobernanza está organizada por evidencia, políticas, adopción, decisiones de release y explicación con citas. La conexión al Control Plane vive en Configuración.',
        accessTitle: 'Workspace admin requerido',
        accessBody: 'Esta herramienta de gobernanza requiere alcance admin. El rol actual aún puede usar el Workspace local y las superficies visibles de auditoría sin abrir editores admin privilegiados.',
        sections: {
          evidence: {
            label: 'Evidencia',
            description: 'Trazabilidad, evidencia de pipeline, paquetes, telemetría y exports.',
          },
          policy: {
            label: 'Políticas',
            description: 'Reglas de ramas, revisión, trazabilidad y enforcement.',
          },
          adoption: {
            label: 'Adopción',
            description: 'Setup enterprise, providers, workflows y tareas de onboarding.',
          },
          releases: {
            label: 'Releases',
            description: 'Decisiones de aprobación, hashes de evidencia y evaluación de gobernanza.',
          },
          copilot: {
            label: 'Copilot',
            description: 'Explicación de gobernanza basada en citas.',
          },
        },
        metrics: {
          traceability: 'Trazabilidad',
          traceabilityDetail: '{{withTicket}}/{{total}} commits vinculados a evidencia de ticket',
          pipelineEvidence: 'Evidencia de pipeline',
          pipelineDetail: '{{total}} run(s), {{failures}} fallando en 7d',
          githubSignals: 'Señales GitHub',
          githubSignalsReady: 'Evidencia de PR, review, comentario y status observada',
          githubSignalsMissing: 'Faltan: {{signals}}',
          evidenceGaps: 'Brechas de evidencia',
          evidenceGapsDetail: '{{traceability}} brecha(s) de trazabilidad, {{blocked}} push(es) bloqueados hoy, {{critical}} violación(es) críticas',
          releaseReadiness: 'Readiness de release',
          releaseReadinessDetail: '{{band}}; objetivo standard {{target}}',
          qualityEvidence: 'Evidencia de calidad',
          qualityEvidenceDetail: '{{passed}}/{{total}} pipeline(s) Sonar/calidad pasados',
          releaseBlockers: 'Bloqueos de release',
          releaseBlockersDetail: '{{critical}} violación(es) críticas, {{failures}} pipeline(s) fallando',
        },
      },
    },
  },
} as const

export function normalizeAppLanguage(value: string | null | undefined): AppLanguage {
  return value === 'es' ? 'es' : DEFAULT_APP_LANGUAGE
}

export function readStoredLanguage(): AppLanguage {
  if (typeof window === 'undefined') return DEFAULT_APP_LANGUAGE
  try {
    return normalizeAppLanguage(window.localStorage.getItem(APP_LANGUAGE_STORAGE_KEY))
  } catch {
    return DEFAULT_APP_LANGUAGE
  }
}

export function persistAppLanguage(language: AppLanguage): void {
  if (typeof window === 'undefined') return
  try {
    window.localStorage.setItem(APP_LANGUAGE_STORAGE_KEY, language)
  } catch {
    // Ignore localStorage failures; i18n still changes for the current session.
  }
}

export function getAppLanguage(): AppLanguage {
  return normalizeAppLanguage(i18n.resolvedLanguage || i18n.language || readStoredLanguage())
}

export async function setAppLanguage(language: AppLanguage): Promise<void> {
  const nextLanguage = normalizeAppLanguage(language)
  persistAppLanguage(nextLanguage)
  if (getAppLanguage() !== nextLanguage) {
    await i18n.changeLanguage(nextLanguage)
  }
}

if (!i18n.isInitialized) {
  void i18n
    .use(initReactI18next)
    .init({
      resources,
      lng: readStoredLanguage(),
      fallbackLng: DEFAULT_APP_LANGUAGE,
      supportedLngs: LANGUAGE_OPTIONS.map((option) => option.value),
      interpolation: {
        escapeValue: false,
      },
      returnNull: false,
    })
}

export default i18n
