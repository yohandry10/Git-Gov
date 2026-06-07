import { useState } from 'react'
import { Link } from 'react-router-dom'
import { Compass, Link2, Settings, Wifi, WifiOff } from 'lucide-react'
import { ActionCenterWorkspace } from '@/components/action_center/ActionCenterWorkspace'
import { ServerConfigPanel } from '@/components/control_plane/ServerConfigPanel'
import { Modal } from '@/components/shared/Modal'
import { useControlPlaneStore } from '@/store/useControlPlaneStore'

export function ActionCenterPage() {
  const isConnected = useControlPlaneStore((state) => state.isConnected)
  const serverConfig = useControlPlaneStore((state) => state.serverConfig)
  const [showConnectionModal, setShowConnectionModal] = useState(false)

  return (
    <div className="h-full flex flex-col bg-surface-950">
      <div className="shrink-0 h-11 px-5 flex items-center justify-between border-b border-white/4 bg-surface-950">
        <div className="flex items-center gap-3">
          <Compass size={14} strokeWidth={1.5} className="text-surface-500" />
          <span className="text-[13px] font-medium text-surface-300 tracking-tight">
            Action Center
          </span>
          {isConnected ? (
            <div className="flex items-center gap-1.5 ml-1">
              <Wifi size={11} className="text-success-500" />
              <span className="text-[11px] text-success-500/80 mono-data">
                {serverConfig?.url?.replace(/^https?:\/\//, '') || 'connected'}
              </span>
            </div>
          ) : (
            <div className="flex items-center gap-1.5 ml-1">
              <WifiOff size={11} className="text-surface-600" />
              <span className="text-[11px] text-surface-600">desconectado</span>
            </div>
          )}
        </div>

        <div className="flex items-center gap-1.5">
          <button
            type="button"
            onClick={() => setShowConnectionModal(true)}
            className="w-7 h-7 flex items-center justify-center rounded-lg text-surface-500 hover:text-surface-300 hover:bg-white/4 transition-all duration-200"
            title="Conexión Control Plane"
          >
            <Link2 size={14} strokeWidth={1.5} />
          </button>
          <Link
            to="/settings"
            className="w-7 h-7 flex items-center justify-center rounded-lg text-surface-500 hover:text-surface-300 hover:bg-white/4 transition-all duration-200"
            title="Configuración"
          >
            <Settings size={14} strokeWidth={1.5} />
          </Link>
        </div>
      </div>

      <div className="flex-1 overflow-auto">
        {!isConnected ? (
          <div className="h-full flex items-center justify-center p-6">
            <div className="w-full max-w-md animate-fade-in">
              <ServerConfigPanel />
            </div>
          </div>
        ) : (
          <ActionCenterWorkspace />
        )}
      </div>

      <Modal
        isOpen={showConnectionModal}
        onClose={() => setShowConnectionModal(false)}
        title="Conexión Control Plane"
        size="md"
      >
        <ServerConfigPanel />
      </Modal>
    </div>
  )
}
