import { useState } from 'react'
import { Header } from '@/components/layout/Header'
import { ChevronDown, ChevronRight, Shield, Database, Eye, Lock, Server, HelpCircle, ExternalLink } from 'lucide-react'
import clsx from 'clsx'

interface FaqItem {
  q: string
  a: string
}

interface FaqSection {
  title: string
  icon: React.ElementType
  items: FaqItem[]
}

const GITGOV_DOCS_URL = 'https://gitgov.cloud/docs/faq'
const GITGOV_CONTACT_URL = 'https://gitgov.cloud/contact'

const faqData: FaqSection[] = [
  {
    title: 'Qué GitGov NO hace',
    icon: Shield,
    items: [
      {
        q: '¿GitGov lee mi código fuente?',
        a: 'No. GitGov solo captura metadatos: tipo de evento, SHA del commit, rama, autor, timestamp, conteo de archivos y nombre del repo. El código fuente, contenido de archivos, diffs y cuerpos de mensajes de commit nunca se transmiten y nunca abandonan tu estación de trabajo.',
      },
      {
        q: '¿GitGov monitoriza mi pantalla, teclado o aplicaciones?',
        a: 'No. GitGov solo observa operaciones Git (commit, push, creación de ramas). No tiene acceso a tu pantalla, portapapeles, navegador, IDE ni ninguna aplicación fuera de Git.',
      },
      {
        q: '¿GitGov analiza la calidad del código?',
        a: 'No. GitGov no hace lint, review ni evalúa la calidad de tu código. Captura metadatos sobre cuándo y dónde ocurren los eventos Git, no qué contiene el código.',
      },
      {
        q: '¿GitGov reemplaza CI/CD?',
        a: 'No. GitGov se integra con herramientas CI/CD (Jenkins, GitHub Actions) para correlacionar commits con pipelines. No ejecuta builds, tests ni despliegues.',
      },
      {
        q: '¿GitGov bloquea operaciones Git?',
        a: 'No. GitGov es una herramienta de detección y observabilidad, no de enforcement. Puede señalar que ocurrió un push a una rama protegida, pero no impide que el push se realice.',
      },
      {
        q: '¿GitGov toma decisiones de RRHH?',
        a: 'No. Las señales son observaciones consultivas — indican que una regla se activó. No establecen intención, negligencia ni culpa. La organización es plenamente responsable de cualquier decisión basada en señales.',
      },
      {
        q: '¿GitGov perfila productividad individual?',
        a: 'No. No hay "líneas de código por día", "puntuaciones de commits" ni rankings de productividad. GitGov es una herramienta de gobernanza y cumplimiento, no de rendimiento.',
      },
    ],
  },
  {
    title: 'Datos y seguridad',
    icon: Lock,
    items: [
      {
        q: '¿Dónde se almacenan mis datos?',
        a: 'Los eventos se almacenan en una base de datos PostgreSQL controlada por tu organización (Supabase o self-hosted). La app de escritorio mantiene un outbox local SQLite para resiliencia offline.',
      },
      {
        q: '¿Mis datos están cifrados?',
        a: 'Sí, en múltiples capas: TLS (HTTPS) en tránsito entre Desktop y Control Plane; AES-256 en reposo en las bases de datos Supabase; API keys almacenadas como hashes SHA-256; y en tu estación de trabajo, las claves se guardan en el keyring del SO (Windows DPAPI, macOS Keychain, Linux Secret Service).',
      },
      {
        q: '¿Se pueden modificar o eliminar registros de auditoría?',
        a: 'No. Los registros son append-only por diseño. La API no expone UPDATE ni DELETE sobre tablas de eventos. Cada exportación se registra también como evento de auditoría.',
      },
      {
        q: '¿Quién puede ver mis eventos?',
        a: 'El acceso se controla con RBAC: los Developers solo ven sus propios eventos. Los Admins ven todos los eventos, estadísticas y dashboard. No hay forma de que un Developer acceda a eventos de otro desarrollador.',
      },
      {
        q: '¿Cómo se protegen las API keys?',
        a: 'Se hashean con SHA-256 antes de almacenarse en la base de datos. La clave en texto plano solo se muestra una vez al crearla. En el escritorio, se almacenan en el keyring del sistema operativo.',
      },
      {
        q: '¿GitGov vende o comparte mis datos?',
        a: 'No. Todos los datos pertenecen a tu organización. GitGov no tiene modelo de monetización de datos. Los datos no se comparten con terceros.',
      },
    ],
  },
  {
    title: 'App de escritorio',
    icon: Database,
    items: [
      {
        q: '¿Qué plataformas soporta GitGov Desktop?',
        a: 'GitGov Desktop está construido con Tauri y soporta Windows, macOS y Linux.',
      },
      {
        q: '¿Qué pasa si pierdo la conexión a internet?',
        a: 'Los eventos se encolan en un outbox local SQLite y se sincronizan automáticamente cuando se restablece la conectividad. No se pierde ningún evento.',
      },
      {
        q: '¿Cómo configuro las políticas de gobernanza?',
        a: 'Las políticas se definen en un archivo gitgov.toml en la raíz de tu repositorio. Puedes verlo desde la pestaña Configuración de esta app.',
      },
    ],
  },
  {
    title: 'Control Plane',
    icon: Server,
    items: [
      {
        q: '¿Qué es el Control Plane?',
        a: 'Es el servidor central Axum (Rust) que recibe eventos de los clientes desktop, procesa webhooks de GitHub/Jenkins/Jira, ejecuta verificaciones de política y sirve el dashboard de administración.',
      },
      {
        q: '¿Puedo self-hostear el Control Plane?',
        a: 'Sí. Puede desplegarse en cualquier servidor que ejecute binarios Rust. Requiere una base de datos PostgreSQL.',
      },
      {
        q: '¿Qué integraciones están soportadas?',
        a: 'GitHub (webhooks de push y ramas, audit log streaming), Jenkins (ingesta de pipelines y correlación commit-pipeline), y Jira (ingesta de tickets, correlación commit-ticket y reportes de cobertura).',
      },
    ],
  },
  {
    title: 'Cumplimiento',
    icon: Eye,
    items: [
      {
        q: '¿GitGov ayuda con SOC 2?',
        a: 'Sí. GitGov proporciona pistas de auditoría append-only, control de acceso basado en roles y registros de eventos inmutables — controles clave para SOC 2 Tipo II.',
      },
      {
        q: '¿GitGov ayuda con RGPD?',
        a: 'Sí. Se diseña con principios RGPD: minimización de datos (solo metadatos), derecho de acceso (desarrolladores ven sus propios eventos), portabilidad (POST /export) y distinción responsable/encargado.',
      },
    ],
  },
]

function FaqAccordion({ section, className }: { section: FaqSection; className?: string }) {
  const [openIndex, setOpenIndex] = useState<number | null>(null)
  const Icon = section.icon

  return (
    <section
      id={section.title.toLowerCase().replace(/\s+/g, '-')}
      className={clsx('overflow-hidden rounded-xl border border-surface-700/30 bg-surface-800/40', className)}
    >
      <div className="flex items-center gap-2.5 border-b border-surface-700/20 px-4 py-3">
        <div className="w-7 h-7 rounded-lg bg-brand-600/15 flex items-center justify-center">
          <Icon size={14} strokeWidth={1.5} className="text-brand-400" />
        </div>
        <h2 className="text-[13px] font-semibold text-white">{section.title}</h2>
      </div>
      <div className="divide-y divide-surface-700/20">
        {section.items.map((item, i) => {
          const isOpen = openIndex === i
          return (
            <div key={i}>
              <button
                onClick={() => setOpenIndex(isOpen ? null : i)}
                className="flex w-full items-center gap-3 px-4 py-3 text-left transition-colors hover:bg-white/[0.02]"
              >
                {isOpen ? (
                  <ChevronDown size={14} className="text-brand-400 flex-shrink-0" />
                ) : (
                  <ChevronRight size={14} className="text-surface-500 flex-shrink-0" />
                )}
                <span className={clsx(
                  'text-[12px] font-medium transition-colors',
                  isOpen ? 'text-white' : 'text-surface-300'
                )}>
                  {item.q}
                </span>
              </button>
              {isOpen && (
                <div className="px-4 pb-4 pl-11">
                  <p className="text-[11px] leading-relaxed text-surface-400">
                    {item.a}
                  </p>
                </div>
              )}
            </div>
          )
        })}
      </div>
    </section>
  )
}

export function HelpPage() {
  return (
    <div className="h-full flex flex-col bg-surface-950">
      <Header />

      <div className="flex-1 overflow-auto">
        <div className="space-y-4 p-5 animate-fade-in">
          <div className="flex flex-col gap-3 xl:flex-row xl:items-end xl:justify-between">
            <div>
              <div className="flex items-center gap-2.5">
                <HelpCircle size={16} className="text-brand-400" />
                <h1 className="text-[15px] font-semibold text-white">Ayuda y FAQ</h1>
              </div>
              <p className="mt-1 max-w-3xl text-xs leading-5 text-surface-500">
                Respuestas operativas sobre privacidad, seguridad, Desktop, Control Plane e integraciones sin salir de GitGov.
              </p>
            </div>
            <a
              href={GITGOV_DOCS_URL}
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex w-fit items-center gap-1.5 rounded-lg border border-brand-500/25 bg-brand-500/10 px-3 py-2 text-[11px] font-medium text-brand-300 transition-colors hover:border-brand-400/45 hover:bg-brand-500/15"
            >
              Ver documentación completa
              <ExternalLink size={11} />
            </a>
          </div>

          <div className="grid grid-cols-1 gap-3 xl:grid-cols-3">
            <div className="rounded-xl border border-brand-500/20 bg-brand-500/5 px-4 py-3">
              <p className="text-[11px] text-brand-300 font-medium mb-1">
                Principio fundamental de GitGov
              </p>
              <p className="text-[11px] text-surface-300 leading-5">
                Solo metadatos, nunca código fuente. El contenido de tus archivos, diffs,
                mensajes de commit, contraseñas y secretos nunca se transmiten ni abandonan tu estación de trabajo.
              </p>
              <p className="mt-2 text-[10px] text-surface-500">
                Garantía arquitectónica, no una opción de configuración.
              </p>
            </div>
            <div className="rounded-xl border border-surface-700/30 bg-surface-800/40 p-4">
              <div className="flex items-center gap-2 text-[11px] font-semibold text-surface-100">
                <Shield size={14} className="text-success-400" />
                Seguridad por diseño
              </div>
              <p className="mt-2 text-[11px] leading-5 text-surface-400">
                RBAC, outbox local, transporte HTTPS y registros append-only explicados por dominio.
              </p>
            </div>
            <div className="rounded-xl border border-surface-700/30 bg-surface-800/40 p-4">
              <div className="flex items-center gap-2 text-[11px] font-semibold text-surface-100">
                <Eye size={14} className="text-warning-400" />
                Límites claros
              </div>
              <p className="mt-2 text-[11px] leading-5 text-surface-400">
                GitGov no lee código, no vigila pantalla y no reemplaza CI/CD ni decisiones humanas.
              </p>
            </div>
          </div>

          <div className="grid grid-cols-1 gap-4 xl:grid-cols-[280px_minmax(0,1fr)]">
            <aside className="space-y-3 xl:sticky xl:top-4 xl:self-start">
              <section className="rounded-xl border border-surface-700/30 bg-surface-800/40 p-4">
                <p className="text-[10px] font-medium uppercase tracking-widest text-surface-500">
                  Categorías
                </p>
                <div className="mt-3 grid grid-cols-1 gap-1.5 sm:grid-cols-2 xl:grid-cols-1">
                  {faqData.map((section) => {
                    const Icon = section.icon
                    return (
                      <a
                        key={section.title}
                        href={`#${section.title.toLowerCase().replace(/\s+/g, '-')}`}
                        className="flex items-center justify-between rounded-lg border border-white/6 bg-white/[0.02] px-3 py-2 text-[11px] text-surface-300 transition-colors hover:border-brand-500/30 hover:bg-brand-500/8 hover:text-surface-100"
                      >
                        <span className="flex min-w-0 items-center gap-2">
                          <Icon size={13} className="shrink-0 text-brand-300" />
                          <span className="truncate">{section.title}</span>
                        </span>
                        <span className="ml-2 rounded bg-surface-700/40 px-1.5 py-0.5 text-[10px] text-surface-500">
                          {section.items.length}
                        </span>
                      </a>
                    )
                  })}
                </div>
              </section>

              <section className="rounded-xl border border-surface-700/30 bg-surface-800/40 p-4">
                <p className="text-[10px] font-medium uppercase tracking-widest text-surface-500">
                  Soporte
                </p>
                <p className="mt-2 text-[11px] leading-5 text-surface-400">
                  Si una respuesta no cubre el caso, consulta la documentación completa o contacta a tu administrador.
                </p>
                <a
                  href={GITGOV_CONTACT_URL}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="mt-3 inline-flex items-center gap-1.5 text-[11px] font-medium text-brand-400 transition-colors hover:text-brand-300"
                >
                  gitgov.cloud/contact
                  <ExternalLink size={10} />
                </a>
              </section>
            </aside>

            <div className="grid grid-cols-1 gap-3 2xl:grid-cols-6">
              {faqData.map((section, index) => (
                <FaqAccordion
                  key={section.title}
                  section={section}
                  className={index < 2 ? '2xl:col-span-3' : '2xl:col-span-2'}
                />
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
