export type Locale = 'en' | 'es';

export const translations = {
    // ═══ Navigation ═══
    'nav.features': { en: 'Features', es: 'Características' },
    'nav.download': { en: 'Download', es: 'Descargar' },
    'nav.docs': { en: 'Docs', es: 'Documentación' },
    'nav.pricing': { en: 'Pricing', es: 'Precios' },
    'nav.contact': { en: 'Contact', es: 'Contacto' },

    // ═══ Hero ═══
    'hero.badge': { en: 'Desktop Available', es: 'Desktop Disponible' },
    'hero.title1': { en: 'Git Governance and', es: 'Gobernanza y trazabilidad' },
    'hero.title2': { en: 'Operational Traceability', es: 'operativa de Git' },
    'hero.subtitle': {
        en: 'Full traceability from commit to CI to compliance. One platform for engineering teams that take operational evidence seriously.',
        es: 'Trazabilidad completa desde el commit hasta CI hasta compliance. Una plataforma para equipos de ingeniería que toman en serio la evidencia operativa.',
    },
    'hero.cta': { en: 'Request a Demo', es: 'Solicitar Demo' },
    'hero.ctaSecondary': { en: 'Download Desktop', es: 'Descargar Desktop' },
    'hero.trust.metadata': { en: 'Metadata only', es: 'Solo metadatos' },
    'hero.trust.selfhosted': { en: 'Self-hosted', es: 'Self-hosted' },
    'hero.trust.appendonly': { en: 'Append-only audit', es: 'Auditoría inmutable' },
    'hero.stat.traceability': { en: 'Commit Traceability', es: 'Trazabilidad de Commits' },
    'hero.stat.full': { en: 'Full', es: 'Completa' },
    'hero.stat.correlation': { en: 'CI Correlation', es: 'Correlación CI' },
    'hero.stat.audit': { en: 'Audit Trail', es: 'Pista de Auditoría' },
    'hero.stat.immutable': { en: 'Immutable', es: 'Inmutable' },

    // ═══ What is GitGov ═══
    'whatIs.badge': { en: 'What is GitGov', es: 'Qué es GitGov' },
    'whatIs.title': { en: 'Governance at the', es: 'Gobernanza en el' },
    'whatIs.titleAccent': { en: 'Source', es: 'Origen' },
    'whatIs.description': {
        en: 'GitGov is a distributed governance system that connects every Git commit to its CI pipeline, Jira ticket, and compliance audit trail — giving CTOs, CISOs, and engineering managers the visibility they need.',
        es: 'GitGov es un sistema de gobernanza distribuido que conecta cada commit de Git con su pipeline CI, ticket de Jira y pista de auditoría de compliance — dando a CTOs, CISOs y gerentes de ingeniería la visibilidad que necesitan.',
    },
    'whatIs.problemTitle': { en: 'The Problem', es: 'El Problema' },
    'whatIs.problemDescription': {
        en: 'Engineering teams ship code without a clear audit trail. Commits happen, pipelines run, tickets close — but nobody can trace the full chain of evidence when compliance asks.',
        es: 'Los equipos de ingeniería envían código sin una pista de auditoría clara. Los commits ocurren, los pipelines se ejecutan, los tickets se cierran — pero nadie puede rastrear la cadena completa de evidencia cuando compliance pregunta.',
    },
    'whatIs.solutionTitle': { en: 'The Solution', es: 'La Solución' },
    'whatIs.solutionDescription': {
        en: "GitGov captures every operation at the source — the developer's machine — and correlates it through your CI and project management tools, creating an immutable record of execution.",
        es: 'GitGov captura cada operación en el origen — la máquina del desarrollador — y la correlaciona a través de tus herramientas de CI y gestión de proyectos, creando un registro inmutable de ejecución.',
    },

    // ═══ Problem ═══
    'problem.badge': { en: 'The Disconnect', es: 'La Desconexión' },
    'problem.title': { en: 'The engineering evidence chain is', es: 'La cadena de evidencia de ingeniería está' },
    'problem.titleAccent': { en: 'fragmented', es: 'fragmentada' },
    'problem.description': {
        en: 'Commits happen, pipelines run, tickets close — but nobody can trace the full chain of execution when an audit or incident occurs.',
        es: 'Los commits ocurren, los pipelines se ejecutan, los tickets se cierran — pero nadie puede rastrear la cadena completa de ejecución cuando ocurre una auditoría o un incidente.'
    },
    'problem.challenge.title': { en: 'Manual Review & Uncertainty', es: 'Revisión Manual e Incertidumbre' },
    'problem.challenge.desc': {
        en: 'Teams waste hours manually collecting evidence across Jira, Jenkins, and Git to prove compliance.',
        es: 'Los equipos pierden horas recolectando evidencia manualmente en Jira, Jenkins y Git para probar el cumplimiento.'
    },
    'problem.solution.title': { en: 'Defensible Operations', es: 'Operaciones Defendibles' },
    'problem.solution.desc': {
        en: 'GitGov captures every operation at the workstation and correlates it automatically, creating an immutable record of execution.',
        es: 'GitGov captura cada operación en la estación de trabajo y la correlaciona automáticamente, creando un registro inmutable de ejecución.'
    },

    // ═══ Trust ═══
    'trust.badge': { en: 'Trust & Architecture', es: 'Confianza y Arquitectura' },
    'trust.title': { en: 'Built for enterprise', es: 'Construido con seguridad' },
    'trust.titleAccent': { en: 'security', es: 'empresarial' },
    'trust.metadata.title': { en: 'Metadata Capture Only', es: 'Solo Captura de Metadatos' },
    'trust.metadata.desc': {
        en: 'No source code, file contents, or secrets ever leave the developer workstation.',
        es: 'Ningún código fuente, contenido de archivo o secreto abandona la estación de trabajo del desarrollador.'
    },
    'trust.selfhosted.title': { en: 'Self-Hosted Deployment', es: 'Despliegue Self-Hosted' },
    'trust.selfhosted.desc': {
        en: 'Keep your audit data in your own infrastructure. Supported on any modern Kubernetes or Docker environment.',
        es: 'Mantén tus datos de auditoría en tu propia infraestructura. Soportado en cualquier entorno moderno de Kubernetes o Docker.'
    },
    'trust.encrypted.title': { en: 'Encrypted at Rest & Transit', es: 'Cifrado en Reposo y Tránsito' },
    'trust.encrypted.desc': {
        en: 'All communication is secured with TLS. Audit logs are protected by database-level AES-256 encryption.',
        es: 'Toda comunicación está asegurada con TLS. Los logs de auditoría son protegidos por cifrado AES-256 a nivel de base de datos.'
    },
    'trust.appendonly.title': { en: 'Append-Only Audit Trail', es: 'Pista de Auditoría Inmutable' },
    'trust.appendonly.desc': {
        en: 'Once recorded, no event can be deleted or modified through the API. Your evidence chain remains unbroken.',
        es: 'Una vez registrado, ningún evento puede ser eliminado o modificado a través de la API. Tu cadena de evidencia permanece intacta.'
    },

    // ═══ How It Works ═══
    'howItWorks.badge': { en: 'How It Works', es: 'Cómo Funciona' },
    'howItWorks.title': { en: 'From Commit to', es: 'Del Commit al' },
    'howItWorks.titleAccent': { en: 'Compliance', es: 'Cumplimiento' },
    'howItWorks.description': {
        en: 'Three layers working together to capture, centralize, and correlate every engineering action.',
        es: 'Tres capas trabajando juntas para capturar, centralizar y correlacionar cada acción de ingeniería.',
    },
    'howItWorks.desktop': { en: 'Desktop App', es: 'App Desktop' },
    'howItWorks.desktopDesc': {
        en: "Capture every Git operation at the developer's machine",
        es: 'Captura cada operación Git en la máquina del desarrollador',
    },
    'howItWorks.controlPlane': { en: 'Control Plane', es: 'Control Plane' },
    'howItWorks.controlPlaneDesc': {
        en: 'Centralize events, enforce policies, generate audit trails',
        es: 'Centraliza eventos, aplica políticas, genera pistas de auditoría',
    },
    'howItWorks.integrations': { en: 'Integrations', es: 'Integraciones' },
    'howItWorks.integrationsDesc': {
        en: 'Correlate with Jenkins CI, Jira tickets, GitHub webhooks',
        es: 'Correlaciona con Jenkins CI, tickets de Jira, webhooks de GitHub',
    },

    // ═══ Capabilities ═══
    'capabilities.badge': { en: 'Capabilities', es: 'Capacidades' },
    'capabilities.title': { en: 'Built for', es: 'Construido para' },
    'capabilities.titleAccent': { en: 'Operational Evidence', es: 'Evidencia Operativa' },
    'capabilities.description': {
        en: 'Every feature is designed to answer one question: can you prove what happened, and when?',
        es: 'Cada funcionalidad está diseñada para responder una pregunta: ¿puedes probar qué sucedió y cuándo?',
    },
    'capabilities.governance.title': { en: 'Git Operation Governance', es: 'Gobernanza de Operaciones Git' },
    'capabilities.governance.desc': {
        en: 'Capture commits, pushes, merges, and rebases at the developer workstation level. No gaps.',
        es: 'Captura commits, pushes, merges y rebases a nivel de la estación del desarrollador. Sin vacíos.',
    },
    'capabilities.audit.title': { en: 'Immutable Audit Trail', es: 'Pista de Auditoría Inmutable' },
    'capabilities.audit.desc': {
        en: 'Append-only event logs with deduplication. Every action recorded, nothing overwritten.',
        es: 'Logs de eventos solo-agregar con deduplicación. Cada acción registrada, nada sobreescrito.',
    },
    'capabilities.ci.title': { en: 'CI Pipeline Correlation', es: 'Correlación de Pipeline CI' },
    'capabilities.ci.desc': {
        en: 'Correlate each commit with its Jenkins pipeline execution, build status, and timing.',
        es: 'Correlaciona cada commit con su ejecución de pipeline Jenkins, estado de build y timing.',
    },
    'capabilities.ticket.title': { en: 'Ticket Traceability', es: 'Trazabilidad de Tickets' },
    'capabilities.ticket.desc': {
        en: 'Map commits and CI runs to Jira tickets for complete coverage visibility.',
        es: 'Mapea commits y ejecuciones CI a tickets de Jira para visibilidad completa de cobertura.',
    },

    // ═══ Roles ═══
    'roles.badge': { en: 'Built for your role', es: 'Construido para tu rol' },
    'roles.title': { en: 'Governance for', es: 'Gobernanza para' },
    'roles.titleAccent': { en: 'Every Stakeholder', es: 'Cada Stakeholder' },
    'roles.description': {
        en: 'Different roles, same need: knowing exactly what happened in your engineering pipeline.',
        es: 'Diferentes roles, misma necesidad: saber exactamente qué sucedió en tu pipeline de ingeniería.',
    },
    'roles.cto.role': { en: 'CTO / CISO', es: 'CTO / CISO' },
    'roles.cto.pain': {
        en: 'No single source of truth for engineering activity when audits or incidents happen.',
        es: 'Sin fuente única de verdad para la actividad de ingeniería cuando ocurren auditorías o incidentes.',
    },
    'roles.cto.solution': {
        en: 'Complete audit trail from Git to CI to tickets. Evidence on demand, no manual collection.',
        es: 'Pista de auditoría completa de Git a CI a tickets. Evidencia bajo demanda, sin recolección manual.',
    },
    'roles.em.role': { en: 'Engineering Manager', es: 'Gerente de Ingeniería' },
    'roles.em.pain': {
        en: 'Fragmented visibility across Git, Jenkins, and Jira. Impossible to correlate at scale.',
        es: 'Visibilidad fragmentada entre Git, Jenkins y Jira. Imposible de correlacionar a escala.',
    },
    'roles.em.solution': {
        en: 'Automated correlation of commits → builds → tickets. See execution flow in one place.',
        es: 'Correlación automatizada de commits → builds → tickets. Ve el flujo de ejecución en un solo lugar.',
    },
    'roles.devops.role': { en: 'DevOps / Platform', es: 'DevOps / Plataforma' },
    'roles.devops.pain': {
        en: 'Policy enforcement relies on manual reviews and tribal knowledge.',
        es: 'La aplicación de políticas depende de revisiones manuales y conocimiento tribal.',
    },
    'roles.devops.solution': {
        en: 'Enforce quality gates (Off/Warn/Block) natively from the workstation before code is even pushed.',
        es: 'Aplica umbrales de calidad (Off/Warn/Block) nativamente desde la estación de trabajo antes de enviar código.',
    },
    'roles.auditor.role': { en: 'Internal Auditor', es: 'Auditor Interno' },
    'roles.auditor.pain': {
        en: 'Relies on screenshots and manual reports from engineering to prepare SOC2/ISO27001 evidence packages.',
        es: 'Depende de capturas de pantalla y reportes manuales de ingeniería para preparar paquetes de evidencia SOC2/ISO27001.',
    },
    'roles.auditor.solution': {
        en: 'Append-only audit trails generated automatically. Evidence is always complete, tamper-proof, and audit-ready.',
        es: 'Pistas de auditoría inmutables generadas automáticamente. La evidencia siempre está completa, inalterable y lista para auditoría.',
    },
    'roles.vpe.role': { en: 'VP of Engineering', es: 'VP de Ingeniería' },
    'roles.vpe.pain': {
        en: 'No org-wide visibility into whether teams follow governance practices consistently across projects.',
        es: 'Sin visibilidad organizacional sobre si los equipos siguen prácticas de gobernanza de forma consistente entre proyectos.',
    },
    'roles.vpe.solution': {
        en: 'Centralized governance dashboard across all teams. Policy compliance metrics without chasing manual reports.',
        es: 'Dashboard centralizado de gobernanza entre todos los equipos. Métricas de cumplimiento de políticas sin perseguir reportes manuales.',
    },

    // ═══ CTA ═══
    'cta.title': { en: 'Ready to govern your', es: '¿Listo para gobernar tu' },
    'cta.titleAccent': { en: 'Git workflow?', es: 'flujo de trabajo Git?' },
    'cta.description': {
        en: 'Download the Desktop app and start capturing operational evidence in minutes.',
        es: 'Descarga la app Desktop y empieza a capturar evidencia operativa en minutos.',
    },
    'cta.primary': { en: 'Download Desktop', es: 'Descargar Desktop' },
    'cta.secondary': { en: 'Read the Docs', es: 'Leer la Documentación' },

    // ═══ Features Page ═══
    'features.badge': { en: 'Product Overview', es: 'Producto' },
    'features.title': { en: 'Operational capabilities of', es: 'Capacidades operativas de' },
    'features.titleAccent': { en: 'GitGov', es: 'GitGov' },
    'features.description': {
        en: 'From workstation capture and governance checks to CI and ticket correlation, GitGov organizes operational evidence into a product surface built for audit, readiness, and reporting.',
        es: 'Desde captura en la estación de trabajo y checks de gobernanza hasta correlación con CI y tickets, GitGov organiza la evidencia operativa en una superficie de producto preparada para auditoría, readiness y reporting.',
    },
    'features.proof.metadata': { en: 'Metadata-only capture', es: 'Captura solo de metadatos' },
    'features.proof.offline': { en: 'Offline queue', es: 'Cola offline' },
    'features.proof.gates': { en: 'Off / Warn / Block', es: 'Off / Warn / Block' },
    'features.proof.integrations': { en: 'Jenkins · Jira · GitHub', es: 'Jenkins · Jira · GitHub' },
    'features.proof.exports': { en: 'Exportable evidence', es: 'Evidencia exportable' },
    'features.hero.mapLabel': { en: 'Product surfaces', es: 'Superficies del producto' },
    'features.hero.mapDescription': {
        en: 'A compact map of how GitGov moves from local capture into governance, correlation, and reporting.',
        es: 'Un mapa compacto de cómo GitGov pasa de la captura local a gobernanza, correlación y reporting.',
    },
    'features.hero.mapSignal': { en: 'Source to reporting', es: 'Del origen al reporte' },
    'features.hero.point.capture.title': { en: 'Capture at the source', es: 'Captura en el origen' },
    'features.hero.point.capture.desc': {
        en: 'Start with workstation-level Git events and resilient local queueing.',
        es: 'Empieza con eventos Git a nivel de estación de trabajo y cola local resiliente.',
    },
    'features.hero.point.governance.title': { en: 'Apply governance early', es: 'Aplica gobernanza temprano' },
    'features.hero.point.governance.desc': {
        en: 'See where branch, ticket, and traceability checks intervene before push.',
        es: 'Ve dónde intervienen los checks de ramas, tickets y trazabilidad antes del push.',
    },
    'features.hero.point.outcomes.title': { en: 'Connect evidence to outcomes', es: 'Conecta evidencia con resultados' },
    'features.hero.point.outcomes.desc': {
        en: 'Follow the path from CI and ticket correlation into reporting and audit outputs.',
        es: 'Sigue el recorrido desde correlación con CI y tickets hasta reporting y salidas de auditoría.',
    },
    'features.hero.navLabel': { en: 'Explore by surface', es: 'Explorar por superficie' },
    'features.hero.navDescription': {
        en: 'Use this map to move from overview into the product details below.',
        es: 'Usa este mapa para pasar del overview al detalle del producto más abajo.',
    },
    'features.hero.surface.capture': {
        en: 'Local-first Git event capture with resilient queueing before anything reaches the network.',
        es: 'Captura local-first de eventos Git con cola resiliente antes de que nada llegue a la red.',
    },
    'features.hero.surface.governance': {
        en: 'Policy checks and configurable enforcement modes applied before push and traceability breaks.',
        es: 'Checks de política y modos de enforcement configurables aplicados antes del push y ante quiebres de trazabilidad.',
    },
    'features.hero.surface.correlation': {
        en: 'Evidence linked across CI pipelines, tickets, and repository activity in a single operational chain.',
        es: 'Evidencia vinculada entre pipelines CI, tickets y actividad del repositorio en una sola cadena operativa.',
    },
    'features.hero.surface.reporting': {
        en: 'Readiness, export history, and audit-facing reporting derived from correlated operational signals.',
        es: 'Readiness, historial de exportación y reporting orientado a auditoría derivados de señales operativas correlacionadas.',
    },
    'features.nav.capture': { en: 'Workstation Capture', es: 'Captura en la Estación' },
    'features.nav.governance': { en: 'Governance Engine', es: 'Motor de Gobernanza' },
    'features.nav.correlation': { en: 'Correlations', es: 'Correlaciones' },
    'features.nav.reporting': { en: 'Risk & Reporting', es: 'Riesgo y Reportes' },
    'features.core.badge': { en: 'Capture', es: 'Captura' },
    'features.core.title': { en: 'Workstation', es: 'Estación de Trabajo' },
    'features.core.titleAccent': { en: 'Capture', es: 'Captura' },
    'features.core.description': {
        en: "Everything begins at the source. GitGov Desktop captures operations natively before they hit the network.",
        es: 'Todo comienza en el origen. GitGov Desktop captura operaciones nativamente antes de que lleguen a la red.',
    },
    'features.commit.title': { en: 'Immutable Commit Logging', es: 'Registro Inmutable de Commits' },
    'features.commit.desc': {
        en: 'Every commit, push, and stage is recorded with local context (author, branch, timestamp) in an append-only log.',
        es: 'Cada commit, push y stage se registra con contexto local (autor, rama, timestamp) en un registro solo-agregar.',
    },
    'features.offline.title': { en: 'Zero Data Loss Architecture', es: 'Arquitectura sin Pérdida de Datos' },
    'features.offline.desc': {
        en: 'Local queuing with exponential backoff ensures evidence is safely stored even without network connectivity.',
        es: 'La cola local con backoff exponencial garantiza que la evidencia se guarde de forma segura incluso sin red.',
    },
    'features.policy.badge': { en: 'Enforcement', es: 'Aplicación' },
    'features.policy.title': { en: 'Governance', es: 'Motor de' },
    'features.policy.titleAccent': { en: 'Engine', es: 'Gobernanza' },
    'features.policy.description': {
        en: 'Run policy-aware checks before push with configurable enforcement across branches, traceability, and quality gates.',
        es: 'Ejecuta verificaciones de gobernanza antes del push con enforcement configurable sobre ramas, trazabilidad y quality gates.',
    },
    'features.policy.check.title': { en: 'Configurable Quality Gates', es: 'Umbrales de Calidad Configurables' },
    'features.policy.check.desc': {
        en: 'Set Off, Warn, or Block modes for branch rules, ticket linkage, commit conventions, and correlated quality-gate evidence.',
        es: 'Configura modos Off, Warn o Block para reglas de ramas, vinculación de tickets, convenciones de commit y evidencia correlacionada de quality gates.',
    },
    'features.policy.mode.off': { en: 'Off (Audit only)', es: 'Off (solo auditoría)' },
    'features.policy.mode.warn': { en: 'Warn (Notify developer)', es: 'Warn (notifica al desarrollador)' },
    'features.policy.mode.block': { en: 'Block (Prevent action)', es: 'Block (impide la acción)' },
    'features.integrations.badge': { en: 'Correlation', es: 'Correlación' },
    'features.integrations.title': { en: 'Integrations &', es: 'Integraciones &' },
    'features.integrations.titleAccent': { en: 'Evidence', es: 'Evidencia' },
    'features.integrations.description': {
        en: 'Connect the dots between what was coded, built, and tracked.',
        es: 'Conecta los puntos entre lo que se programó, se construyó y se rastreó.',
    },
    'features.jenkins.title': { en: 'CI Pipeline Tracing', es: 'Trazabilidad de Pipelines CI' },
    'features.jenkins.desc': {
        en: 'Correlate commits with Jenkins builds and surface execution status, duration, and release-readiness context.',
        es: 'Correlaciona commits con builds de Jenkins y expone estado de ejecución, duración y contexto de release readiness.',
    },
    'features.risk.badge': { en: 'Visibility', es: 'Visibilidad' },
    'features.risk.title': { en: 'Risk, Readiness &', es: 'Riesgo, Preparación &' },
    'features.risk.titleAccent': { en: 'Reporting', es: 'Reportes' },
    'features.risk.description': {
        en: 'Turn workstation, CI, and ticket evidence into release-readiness, risk, and exportable reporting surfaces.',
        es: 'Convierte evidencia de workstation, CI y tickets en superficies de release readiness, riesgo y reportes exportables.',
    },
    'features.centralized.title': { en: 'Live Compliance Dashboard', es: 'Dashboard de Cumplimiento en Vivo' },
    'features.centralized.desc': {
        en: 'The Control Plane centralizes pipeline health, ticket coverage, risk outcomes, and export history in one reporting surface.',
        es: 'El Control Plane centraliza salud de pipelines, cobertura de tickets, risk outcomes e historial de exportación en una sola superficie de reporte.',
    },
    'features.risk.audit.title': { en: 'Append-Only Audit Trails', es: 'Pistas de Auditoría Inmutables' },
    'features.risk.audit.desc': {
        en: 'Evidence remains append-only from workstation capture through policy decisions and export history, supporting readiness reviews and formal audits.',
        es: 'La evidencia se mantiene append-only desde la captura local hasta decisiones de política e historial de exportación, apoyando revisiones de readiness y auditorías formales.',
    },
    'features.jira.title': { en: 'Jira Ticket Coverage', es: 'Cobertura de Tickets Jira' },
    'features.jira.desc': {
        en: "Map commits, correlated CI runs, PR titles, and PR-linked comments to Jira-style ticket IDs. Surface coverage gaps when changes aren't linked to a ticket.",
        es: 'Mapea commits, ejecuciones CI correlacionadas, títulos de PR y comentarios vinculados a PRs a IDs estilo Jira. Expone brechas cuando los cambios no están vinculados a un ticket.',
    },

    'features.dashboard.title': { en: 'Admin Dashboard', es: 'Panel de Administración' },
    'features.dashboard.desc': {
        en: 'Built-in dashboard with recent commits, Pipeline Health (7d), Ticket Coverage, Risk Outcomes, Export Panel, and 30-second refresh.',
        es: 'Panel integrado con commits recientes, Pipeline Health (7d), Ticket Coverage, Risk Outcomes, Export Panel y refresco de 30 segundos.',
    },
    'features.github.title': { en: 'GitHub Webhooks', es: 'Webhooks de GitHub' },
    'features.github.desc': {
        en: 'Receive webhook evidence for pushes, branches, PR lifecycle, reviews, PR-linked comments, and status checks. Comment text improves ticket traceability only when it contains ticket IDs.',
        es: 'Recibe evidencia vía webhooks para pushes, ramas, ciclo de PR, reviews, comentarios vinculados a PRs y status checks. El texto de comentarios mejora trazabilidad solo cuando contiene IDs de ticket.',
    },
    'features.dashboard.surface.title': { en: 'Operational Reporting Surface', es: 'Superficie de Reporte Operativa' },
    'features.dashboard.surface.pipeline': { en: 'Pipeline Health (7d)', es: 'Pipeline Health (7d)' },
    'features.dashboard.surface.pipelineDesc': { en: 'Jenkins execution health and failure visibility.', es: 'Salud de ejecución Jenkins y visibilidad de fallos.' },
    'features.dashboard.surface.coverage': { en: 'Ticket Coverage', es: 'Cobertura de Tickets' },
    'features.dashboard.surface.coverageDesc': { en: 'Commit-to-ticket coverage, including PR title/comment evidence when ticket IDs are present.', es: 'Cobertura commit-ticket, incluyendo evidencia de títulos/comentarios PR cuando hay IDs de ticket.' },
    'features.dashboard.surface.risk': { en: 'Risk Outcomes', es: 'Risk Outcomes' },
    'features.dashboard.surface.riskDesc': { en: 'Tier-aware readiness and operational risk signals.', es: 'Señales tier-aware de readiness y riesgo operativo.' },
    'features.dashboard.surface.export': { en: 'Export History', es: 'Historial de Exportación' },
    'features.dashboard.surface.exportDesc': { en: 'Audit exports with content hashes and traceable history.', es: 'Exports de auditoría con content hash e historial trazable.' },
    'features.risk.audit.item1': { en: 'Policy history snapshots', es: 'Snapshots de historial de políticas' },
    'features.risk.audit.item2': { en: 'Violation decisions', es: 'Decisiones sobre violaciones' },
    'features.risk.audit.item3': { en: 'Export logs with integrity hash', es: 'Logs de exportación con hash de integridad' },
    'features.cta.title': { en: 'See it in', es: 'Verlo en' },
    'features.cta.titleAccent': { en: 'Action', es: 'Acción' },
    'features.cta.desc': {
        en: 'Download the Desktop app and connect to your Control Plane to start capturing evidence.',
        es: 'Descarga la app Desktop y conéctate a tu Control Plane para empezar a capturar evidencia.',
    },
    'features.cta.primary': { en: 'Download Desktop', es: 'Descargar Desktop' },
    'features.cta.secondary': { en: 'Read Documentation', es: 'Leer Documentación' },

    // ═══ Download Page ═══
    'download.badge': { en: 'Download', es: 'Descargar' },
    'download.title': { en: 'Get', es: 'Obtén' },
    'download.titleAccent': { en: 'GitGov Desktop', es: 'GitGov Desktop' },
    'download.description': {
        en: 'Start capturing Git operations on your machine. Free for development teams.',
        es: 'Empieza a capturar operaciones Git en tu máquina. Gratis para equipos de desarrollo.',
    },
    'download.button': { en: 'Download .exe', es: 'Descargar .exe' },
    'download.otherPlatforms': {
        en: 'macOS and Linux builds are planned for future releases.',
        es: 'Los builds de macOS y Linux están planeados para versiones futuras.',
    },
    'download.planned': { en: 'Planned', es: 'Planeado' },
    'download.notice': {
        en: 'Build available internally. Contact the team for access.',
        es: 'Build disponible internamente. Contacta al equipo para acceso.',
    },
    'download.installNotes': { en: 'Installation Notes', es: 'Notas de Instalación' },
    'download.step1': {
        en: 'Download the <code>.exe</code> installer',
        es: 'Descarga el instalador <code>.exe</code>',
    },
    'download.step2': {
        en: 'Run the installer. If Windows displays a security verification prompt, follow the on-screen steps.',
        es: 'Ejecuta el instalador. Si Windows muestra una verificación de seguridad, sigue los pasos en pantalla.',
    },
    'download.step3': {
        en: 'Launch GitGov Desktop — it connects automatically',
        es: 'Inicia GitGov Desktop — se conecta automáticamente',
    },
    'download.step4': {
        en: 'Start working — every Git operation will be captured automatically',
        es: 'Empieza a trabajar — cada operación Git será capturada automáticamente',
    },

    'download.file': { en: 'File', es: 'Archivo' },
    'download.checksum': { en: 'Integrity (SHA256)', es: 'Integridad (SHA256)' },
    'download.copyChecksum': { en: 'Copy SHA256', es: 'Copiar SHA256' },
    'download.copiedChecksum': { en: 'Copied', es: 'Copiado' },
    'download.buttonMsi': { en: 'Download .msi', es: 'Descargar .msi' },
    'download.unsignedBanner': {
        en: 'Official GitGov installer (v0.1.0). If Windows shows a security verification prompt, follow the installation notes below.',
        es: 'Instalador oficial de GitGov (v0.1.0). Si Windows muestra una verificación de seguridad, sigue las notas de instalación de abajo.',
    },
    'download.verifyHash.title': {
        en: 'Optional integrity check (SHA256) on Windows',
        es: 'Verificación opcional de integridad (SHA256) en Windows',
    },
    'download.verifyHash.command': {
        en: 'Run in PowerShell:',
        es: 'Ejecuta en PowerShell:',
    },
    'download.verifyHash.example': {
        en: 'Expected output (Hash field):',
        es: 'Salida esperada (campo Hash):',
    },
    'download.side.heading': {
        en: 'Everything runs on your workstation',
        es: 'Todo corre en tu estación de trabajo',
    },
    'download.side.intro': {
        en: 'GitGov Desktop is a lightweight native app that captures Git events locally and syncs them to your Control Plane — no cloud dependency required.',
        es: 'GitGov Desktop es una app nativa ligera que captura eventos Git localmente y los sincroniza con tu Control Plane — sin dependencia de la nube.',
    },
    'download.side.detailTitle': {
        en: 'What the desktop covers from day one',
        es: 'Qué cubre el desktop desde el primer día',
    },
    'download.side.h1title': { en: 'Git Event Capture', es: 'Captura de Eventos Git' },
    'download.side.h1desc': {
        en: 'Commits, pushes, stages, and merges captured automatically at the workstation level.',
        es: 'Commits, pushes, stages y merges capturados automáticamente a nivel de estación de trabajo.',
    },
    'download.side.h2title': { en: 'Offline Resilience', es: 'Resiliencia Offline' },
    'download.side.h2desc': {
        en: 'Local outbox queues events when the server is unreachable. Syncs automatically on reconnection.',
        es: 'La bandeja de salida local encola eventos cuando el servidor no está disponible. Sincroniza automáticamente al reconectarse.',
    },
    'download.side.h3title': { en: 'Control Plane Ready', es: 'Control Plane Listo' },
    'download.side.h3desc': {
        en: 'Connect to your self-hosted server and access the full governance dashboard instantly.',
        es: 'Conéctate a tu servidor self-hosted y accede al dashboard de gobernanza completo de inmediato.',
    },
    'download.side.h4title': { en: 'Governance Checks', es: 'Verificaciones de Gobernanza' },
    'download.side.h4desc': {
        en: 'Configurable policy checks for commits, branches, and traceability. Automatic signals for policy violations.',
        es: 'Verificaciones de políticas configurables para commits, ramas y trazabilidad. Señales automáticas ante violaciones de políticas.',
    },
    'download.side.h5title': { en: 'Native Notifications', es: 'Notificaciones Nativas' },
    'download.side.h5desc': {
        en: 'OS-level alerts for blocked pushes and governance violations — no browser tab required.',
        es: 'Alertas a nivel de SO para pushes bloqueados y violaciones de gobernanza — sin necesidad de navegador.',
    },
    'download.side.h6title': { en: 'AI Governance Bot', es: 'Bot de Gobernanza IA' },
    'download.side.h6desc': {
        en: 'Ask questions about your audit data in natural language. Powered by Gemini with 11+ query types.',
        es: 'Consulta tus datos de auditoría en lenguaje natural. Potenciado por Gemini con 11+ tipos de consulta.',
    },
    'download.side.h7title': { en: 'Automatic Updates', es: 'Actualizaciones Automáticas' },
    'download.side.h7desc': {
        en: 'Signed OTA updates with Stable and Beta channels. Never miss a security patch.',
        es: 'Actualizaciones OTA firmadas con canales Estable y Beta. Nunca te pierdas un parche de seguridad.',
    },
    'download.side.h8title': { en: 'Compliance Signals', es: 'Señales de Compliance' },
    'download.side.h8desc': {
        en: 'Automatic detection of unreviewed merges, missing tickets, force pushes, and large changesets.',
        es: 'Detección automática de merges sin revisión, tickets faltantes, force pushes y changesets grandes.',
    },
    'download.side.sysreq': {
        en: 'Windows 10 / 11 · x64 · ~15 MB · no runtime dependencies',
        es: 'Windows 10 / 11 · x64 · ~15 MB · sin dependencias de runtime',
    },
    'download.value.security.title': { en: 'Enterprise Security', es: 'Seguridad Empresarial' },
    'download.value.security.desc': {
        en: 'Encrypted transport, metadata-only capture, and no source code transmission.',
        es: 'Transporte cifrado, captura solo de metadatos y sin transmisión de código fuente.',
    },
    'download.value.zeroOverhead.title': { en: 'Zero Overhead', es: 'Sin Fricción' },
    'download.value.zeroOverhead.desc': {
        en: 'Runs quietly in the background without disrupting the developer workflow.',
        es: 'Corre en segundo plano sin interrumpir el flujo de trabajo del desarrollador.',
    },
    'download.value.offline.title': { en: 'Offline Resilience', es: 'Resiliencia Offline' },
    'download.value.offline.desc': {
        en: 'Events queue locally when offline so evidence is never dropped.',
        es: 'Los eventos se encolan localmente cuando no hay red para no perder evidencia.',
    },

    // ═══ Contact Page ═══
    'contact.badge': { en: 'Contact', es: 'Contacto' },
    'contact.title': { en: 'Get in', es: 'Ponte en' },
    'contact.titleAccent': { en: 'Touch', es: 'Contacto' },
    'contact.description': {
        en: "Have questions about GitGov? Want to discuss enterprise deployment? We'd love to hear from you.",
        es: '¿Tienes preguntas sobre GitGov? ¿Quieres discutir un despliegue empresarial? Nos encantaría escucharte.',
    },
    'contact.form.title': { en: 'Send us a message', es: 'Envíanos un mensaje' },
    'contact.form.subtitle': {
        en: 'Tell us about your team, current stack, and evaluation path.',
        es: 'Cuéntanos sobre tu equipo, stack actual y ruta de evaluación.',
    },
    'contact.form.name': { en: 'Name', es: 'Nombre' },
    'contact.form.namePlaceholder': { en: 'Your name', es: 'Tu nombre' },
    'contact.form.email': { en: 'Email', es: 'Correo electrónico' },
    'contact.form.emailPlaceholder': { en: 'you@company.com', es: 'tu@empresa.com' },
    'contact.form.company': { en: 'Company', es: 'Empresa' },
    'contact.form.companyPlaceholder': { en: 'Your company', es: 'Tu empresa' },
    'contact.form.teamSize': { en: 'Engineering Team Size', es: 'Tamaño del Equipo' },
    'contact.form.teamSizePlaceholder': { en: 'Select team size...', es: 'Selecciona...' },
    'contact.form.teamSize.option1': { en: '1-10 developers', es: '1-10 desarrolladores' },
    'contact.form.teamSize.option2': { en: '11-50 developers', es: '11-50 desarrolladores' },
    'contact.form.teamSize.option3': { en: '51-200 developers', es: '51-200 desarrolladores' },
    'contact.form.teamSize.option4': { en: '201-1000 developers', es: '201-1000 desarrolladores' },
    'contact.form.teamSize.option5': { en: '1000+ developers', es: '1000+ desarrolladores' },
    'contact.form.toolchain': { en: 'Primary CI/CD Stack', es: 'Stack CI/CD Principal' },
    'contact.form.toolchainPlaceholder': { en: 'e.g. Jenkins, GitHub, Jira', es: 'ej. Jenkins, GitHub, Jira' },
    'contact.form.interestType': { en: 'Primary Interest', es: 'Interés Principal' },
    'contact.form.interestTypePlaceholder': { en: 'Select primary interest...', es: 'Selecciona una opción...' },
    'contact.form.interestType.demo': { en: 'Product demo', es: 'Demo de producto' },
    'contact.form.interestType.pilot': { en: 'Pilot program', es: 'Programa piloto' },
    'contact.form.interestType.pricing': { en: 'Pricing discussion', es: 'Conversación comercial' },
    'contact.form.interestType.partnership': { en: 'Partnership', es: 'Partnership' },
    'contact.form.interestType.other': { en: 'Other', es: 'Otro' },
    'contact.form.message': { en: 'Message', es: 'Mensaje' },
    'contact.form.messagePlaceholder': {
        en: 'Tell us about your governance needs...',
        es: 'Cuéntanos sobre tus necesidades de gobernanza...',
    },
    'contact.form.send': { en: 'Send Message', es: 'Enviar Mensaje' },
    'contact.form.sending': { en: 'Sending...', es: 'Enviando...' },
    'contact.success.title': { en: 'Message Sent', es: 'Mensaje Enviado' },
    'contact.success.description': {
        en: "Thank you for reaching out. We'll get back to you as soon as possible.",
        es: 'Gracias por contactarnos. Te responderemos lo antes posible.',
    },
    'contact.success.button': { en: 'Send another message', es: 'Enviar otro mensaje' },
    'contact.error': {
        en: 'Something went wrong. Please try again.',
        es: 'Algo salió mal. Por favor inténtalo de nuevo.',
    },
    'contact.errors.name': { en: 'Name is required', es: 'El nombre es requerido' },
    'contact.errors.company': { en: 'Company is required', es: 'La empresa es requerida' },
    'contact.errors.email': { en: 'Email is required', es: 'El correo es requerido' },
    'contact.errors.emailInvalid': { en: 'Invalid email address', es: 'Correo electrónico inválido' },
    'contact.errors.teamSize': { en: 'Team size is required', es: 'El tamaño del equipo es requerido' },
    'contact.errors.interestType': { en: 'Primary interest is required', es: 'El interés principal es requerido' },
    'contact.errors.message': { en: 'Message is required', es: 'El mensaje es requerido' },
    'contact.side.heading': { en: 'What happens next', es: 'Qué sucede después' },
    'contact.side.intro': {
        en: "We'll use your team profile and deployment goals to shape the first conversation around GitGov.",
        es: 'Usaremos el perfil de tu equipo y tus objetivos de despliegue para orientar la primera conversación sobre GitGov.',
    },
    'contact.side.responseTime': {
        en: 'We typically respond within 1 business day.',
        es: 'Solemos responder en menos de 1 día hábil.',
    },
    'contact.side.h1title': { en: 'We review your environment', es: 'Revisamos tu entorno' },
    'contact.side.h1desc': { en: 'We look at team size, governance needs, and current tooling before the first call.', es: 'Evaluamos el tamaño del equipo, necesidades de gobernanza y tooling actual antes de la primera llamada.' },
    'contact.side.h2title': { en: 'We scope the right deployment', es: 'Definimos el despliegue adecuado' },
    'contact.side.h2desc': { en: 'Self-hosted, hybrid, or managed rollout depending on your security and operations model.', es: 'Self-hosted, híbrido o gestionado según tu modelo de seguridad y operación.' },
    'contact.side.h3title': { en: 'We schedule a focused demo', es: 'Agendamos una demo enfocada' },
    'contact.side.h3desc': { en: 'The next step is a short conversation tailored to your repositories, CI, and audit priorities.', es: 'El siguiente paso es una conversación breve adaptada a tus repositorios, CI y prioridades de auditoría.' },

    // ═══ Pricing Page ═══
    'pricing.badge': { en: 'Sales', es: 'Ventas' },
    'pricing.title': { en: "Let's talk about your", es: 'Hablemos de tu' },
    'pricing.titleAccent': { en: 'deployment', es: 'despliegue' },
    'pricing.description': {
        en: 'GitGov is sold around rollout scope, security posture, and operational complexity, not fake self-serve tiers.',
        es: 'GitGov se vende en función del alcance del rollout, postura de seguridad y complejidad operativa, no con tiers ficticios de autoservicio.',
    },
    'pricing.story.badge': { en: 'Commercial motion', es: 'Movimiento comercial' },
    'pricing.story.title': { en: 'A buying path built for enterprise evaluation', es: 'Una ruta comercial pensada para evaluación enterprise' },
    'pricing.story.desc': {
        en: 'We scope GitGov with your repositories, CI stack, governance model, and audit expectations before proposing a rollout.',
        es: 'Definimos GitGov junto con tus repositorios, stack de CI, modelo de gobernanza y exigencias de auditoría antes de proponer un rollout.',
    },
    'pricing.deployment.title': { en: 'Deployment options', es: 'Opciones de despliegue' },
    'pricing.deployment.desc': {
        en: 'Choose the operating model that fits your security boundaries and internal platform team.',
        es: 'Elige el modelo operativo que mejor encaje con tus límites de seguridad y tu equipo de plataforma.',
    },
    'pricing.deployment.self.title': { en: 'Self-hosted', es: 'Self-hosted' },
    'pricing.deployment.self.desc': {
        en: 'For organizations that want GitGov inside their own infrastructure, network controls, and database boundary.',
        es: 'Para organizaciones que quieren GitGov dentro de su propia infraestructura, controles de red y perímetro de base de datos.',
    },
    'pricing.deployment.managed.title': { en: 'Managed rollout', es: 'Rollout gestionado' },
    'pricing.deployment.managed.desc': {
        en: 'For teams that want faster evaluation with guided onboarding, support, and operational assistance.',
        es: 'Para equipos que quieren evaluar más rápido con onboarding guiado, soporte y asistencia operativa.',
    },
    'pricing.deployment.hybrid.title': { en: 'Hybrid', es: 'Híbrido' },
    'pricing.deployment.hybrid.desc': {
        en: 'For enterprises that need local evidence boundaries with centralized rollout support and phased adoption.',
        es: 'Para empresas que necesitan límites locales para la evidencia con soporte centralizado y adopción por fases.',
    },
    'pricing.fit.title': { en: 'Who should talk to sales', es: 'Quién debería hablar con ventas' },
    'pricing.fit.desc': {
        en: 'This page is for teams evaluating GitGov as a control-plane product, not a commodity download.',
        es: 'Esta página es para equipos que evalúan GitGov como producto de control-plane, no como una descarga commodity.',
    },
    'pricing.fit.item1': { en: 'You need governance evidence across Git, CI, and ticketing systems.', es: 'Necesitas evidencia de gobernanza entre Git, CI y sistemas de tickets.' },
    'pricing.fit.item2': { en: 'You are planning a pilot, regulated rollout, or internal audit readiness program.', es: 'Estás preparando un piloto, rollout regulado o programa de preparación para auditoría interna.' },
    'pricing.fit.item3': { en: 'You need help deciding between self-hosted, managed, or phased enterprise deployment.', es: 'Necesitas ayuda para decidir entre despliegue self-hosted, gestionado o enterprise por fases.' },
    'pricing.process.title': { en: 'What happens next', es: 'Qué sucede después' },
    'pricing.process.step1.title': { en: '1. Qualification call', es: '1. Llamada de calificación' },
    'pricing.process.step1.desc': {
        en: 'We review your repositories, CI/CD tooling, governance priorities, and rollout constraints.',
        es: 'Revisamos tus repositorios, tooling de CI/CD, prioridades de gobernanza y restricciones de rollout.',
    },
    'pricing.process.step2.title': { en: '2. Deployment recommendation', es: '2. Recomendación de despliegue' },
    'pricing.process.step2.desc': {
        en: 'We recommend the right operating model and the minimum scope for a credible pilot or production rollout.',
        es: 'Recomendamos el modelo operativo correcto y el alcance mínimo para un piloto creíble o rollout productivo.',
    },
    'pricing.process.step3.title': { en: '3. Commercial proposal', es: '3. Propuesta comercial' },
    'pricing.process.step3.desc': {
        en: 'Once scope is clear, we move to pilot planning, support model, and commercial terms.',
        es: 'Cuando el alcance está claro, pasamos a la planificación del piloto, modelo de soporte y términos comerciales.',
    },
    'pricing.cta.primary': { en: 'Talk to Sales', es: 'Hablar con Ventas' },
    'pricing.cta.secondary': { en: 'Explore Docs', es: 'Explorar Docs' },

    // ═══ 404 ═══
    '404.title': { en: 'Page Not Found', es: 'Página No Encontrada' },
    '404.description': {
        en: "The page you're looking for doesn't exist or has been moved. Check the URL or head back to a known route.",
        es: 'La página que buscas no existe o ha sido movida. Revisa la URL o regresa a una ruta conocida.',
    },
    '404.home': { en: 'Back to Home', es: 'Volver al Inicio' },
    '404.docs': { en: 'Browse Docs', es: 'Explorar Docs' },

    // ═══ Navigation extra ═══
    'nav.privacy': { en: 'Privacy Policy', es: 'Política de Privacidad' },

    // ═══ Footer ═══
    'footer.product': { en: 'Product', es: 'Producto' },
    'footer.resources': { en: 'Resources', es: 'Recursos' },
    'footer.resources.documentation': { en: 'Documentation', es: 'Documentación' },
    'footer.resources.installationguide': { en: 'Installation Guide', es: 'Guía de Instalación' },
    'footer.resources.controlplanesetup': { en: 'Control Plane Setup', es: 'Configuración Control Plane' },
    'footer.company': { en: 'Company', es: 'Empresa' },
    'footer.rights': { en: 'All rights reserved.', es: 'Todos los derechos reservados.' },
    'footer.tagline': { en: 'Governance · Traceability · Compliance', es: 'Gobernanza · Trazabilidad · Cumplimiento' },

    // ═══ Docs ═══
    'docs.title': { en: 'Documentation', es: 'Documentación' },
    'docs.category.evaluate': { en: 'Evaluate', es: 'Evaluar' },
    'docs.category.deploy': { en: 'Deploy', es: 'Desplegar' },
    'docs.category.operate': { en: 'Operate', es: 'Operar' },

    // ═══ Misc ═══
    'advisory': { en: 'Advisory', es: 'Consultivo' },
    'preview': { en: 'Preview', es: 'Vista Previa' },
    'inProgress': { en: 'In Progress', es: 'En Progreso' },
    'available': { en: 'Available', es: 'Disponible' },
    'challenge': { en: 'Challenge', es: 'Desafío' },
    'withGitGov': { en: 'With GitGov', es: 'Con GitGov' },
} as const;

export type TranslationKey = keyof typeof translations;
