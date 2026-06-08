import { lazy, Suspense, type ReactNode } from 'react'
import { createBrowserRouter, Link, Navigate, RouterProvider, useRouteError } from 'react-router-dom'
import { MainLayout } from '@/components/layout/MainLayout'
import { DashboardPage } from '@/pages/DashboardPage'
import { AuditPage } from '@/pages/AuditPage'
import { SettingsPage } from '@/pages/SettingsPage'
import { HelpPage } from '@/pages/HelpPage'
import { AlertCircle } from 'lucide-react'

const ActionCenterPage = lazy(() =>
  import('@/pages/ActionCenterPage').then((module) => ({ default: module.ActionCenterPage })),
)
const GovernancePage = lazy(() =>
  import('@/pages/GovernancePage').then((module) => ({ default: module.GovernancePage })),
)

function RouteSuspense({ children }: { children: ReactNode }) {
  return (
    <Suspense
      fallback={(
        <div className="flex h-full items-center justify-center text-xs text-surface-500">
          Loading view...
        </div>
      )}
    >
      {children}
    </Suspense>
  )
}

function NotFoundPage() {
  return (
    <div className="flex flex-col items-center justify-center h-full text-center p-8">
      <AlertCircle size={40} className="text-surface-600 mb-4" />
      <h1 className="text-xl font-semibold text-white mb-2">Página no encontrada</h1>
      <p className="text-sm text-surface-400 mb-6">La ruta solicitada no existe.</p>
      <Link
        to="/"
        className="px-4 py-2 bg-brand-600 hover:bg-brand-500 text-white text-sm rounded-lg transition-colors"
      >
        Volver al inicio
      </Link>
    </div>
  )
}

function RouteErrorPage() {
  const routeError = useRouteError() as { message?: string; statusText?: string } | null
  const message = routeError?.message || routeError?.statusText || 'Error inesperado en la vista actual.'
  return (
    <div className="flex flex-col items-center justify-center h-full text-center p-8">
      <AlertCircle size={40} className="text-danger-500 mb-4" />
      <h1 className="text-xl font-semibold text-white mb-2">Error en la vista</h1>
      <p className="text-sm text-surface-400 mb-2 max-w-xl">{message}</p>
      <p className="text-xs text-surface-500 mb-6">La app sigue activa. Puedes volver al inicio y continuar.</p>
      <Link
        to="/"
        className="px-4 py-2 bg-brand-600 hover:bg-brand-500 text-white text-sm rounded-lg transition-colors"
      >
        Volver al inicio
      </Link>
    </div>
  )
}

const appRouter = createBrowserRouter([
  {
    path: '/',
    element: (
      <MainLayout>
        <DashboardPage />
      </MainLayout>
    ),
    errorElement: (
      <MainLayout>
        <RouteErrorPage />
      </MainLayout>
    ),
  },
  {
    path: '/audit',
    element: (
      <MainLayout>
        <AuditPage />
      </MainLayout>
    ),
    errorElement: (
      <MainLayout>
        <RouteErrorPage />
      </MainLayout>
    ),
  },
  {
    path: '/settings',
    element: (
      <MainLayout>
        <SettingsPage />
      </MainLayout>
    ),
    errorElement: (
      <MainLayout>
        <RouteErrorPage />
      </MainLayout>
    ),
  },
  {
    path: '/control-plane',
    element: (
      <MainLayout>
        <Navigate to="/settings#control-plane" replace />
      </MainLayout>
    ),
    errorElement: (
      <MainLayout>
        <RouteErrorPage />
      </MainLayout>
    ),
  },
  {
    path: '/governance',
    element: (
      <MainLayout>
        <RouteSuspense>
          <GovernancePage />
        </RouteSuspense>
      </MainLayout>
    ),
    errorElement: (
      <MainLayout>
        <RouteErrorPage />
      </MainLayout>
    ),
  },
  {
    path: '/governance/:section',
    element: (
      <MainLayout>
        <RouteSuspense>
          <GovernancePage />
        </RouteSuspense>
      </MainLayout>
    ),
    errorElement: (
      <MainLayout>
        <RouteErrorPage />
      </MainLayout>
    ),
  },
  {
    path: '/action-center',
    element: (
      <MainLayout>
        <RouteSuspense>
          <ActionCenterPage />
        </RouteSuspense>
      </MainLayout>
    ),
    errorElement: (
      <MainLayout>
        <RouteErrorPage />
      </MainLayout>
    ),
  },
  {
    path: '/help',
    element: (
      <MainLayout>
        <HelpPage />
      </MainLayout>
    ),
    errorElement: (
      <MainLayout>
        <RouteErrorPage />
      </MainLayout>
    ),
  },
  {
    path: '*',
    element: (
      <MainLayout>
        <NotFoundPage />
      </MainLayout>
    ),
    errorElement: (
      <MainLayout>
        <RouteErrorPage />
      </MainLayout>
    ),
  },
])

export function AppRouter() {
  return <RouterProvider router={appRouter} />
}
