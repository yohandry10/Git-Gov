export const nativeTerminalDisabledEnv = 'GITGOV_ENABLE_NATIVE_TERMINAL'

export const nativeTerminalDisabledMessage =
  `Native terminal disabled by local configuration. Remove ${nativeTerminalDisabledEnv}=false or set it to true, then restart GitGov Desktop.`

export function isNativeTerminalDisabledError(error: unknown): boolean {
  const message = error instanceof Error ? error.message : String(error ?? '')
  const normalized = message.toLowerCase()

  return (
    message.includes(nativeTerminalDisabledEnv) &&
    normalized.includes('native terminal') &&
    normalized.includes('disabled')
  )
}
