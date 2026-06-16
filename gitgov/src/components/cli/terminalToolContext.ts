export type NativeTerminalToolName = 'terraform' | 'docker-compose' | 'helm' | 'kubernetes'

export interface NativeTerminalToolDetection {
  tool: NativeTerminalToolName
  detected: boolean
  confidence: 'high' | 'none' | string
  reason: string
  safe_command_ids: string[]
}

export interface NativeTerminalToolContext {
  cwd_kind: 'git_repo' | 'non_git' | 'unknown' | string
  tools: NativeTerminalToolDetection[]
  scan_limited: boolean
  secrets_read: boolean
  network_used: boolean
  detected_at_ms: number
}

const TOOL_LABELS: Record<NativeTerminalToolName, string> = {
  terraform: 'Terraform detected',
  'docker-compose': 'Docker Compose detected',
  helm: 'Helm context detected',
  kubernetes: 'Kubernetes context detected',
}

export function terminalToolContextDetectedLabels(
  context: NativeTerminalToolContext | null,
): string[] {
  if (!context || context.secrets_read || context.network_used) return []
  return context.tools
    .filter((tool) => tool.detected)
    .map((tool) => TOOL_LABELS[tool.tool])
    .filter(Boolean)
}

export function terminalToolContextHasCommand(
  context: NativeTerminalToolContext | null,
  toolName: string,
  commandId: string,
): boolean {
  if (!context || context.secrets_read || context.network_used) return false
  return context.tools.some(
    (tool) =>
      tool.detected &&
      tool.tool === toolName &&
      tool.safe_command_ids.includes(commandId),
  )
}

export function terminalToolContextSafetyLabel(
  context: NativeTerminalToolContext | null,
): string {
  if (!context) return 'Tool context pending'
  if (context.secrets_read || context.network_used) return 'Tool context unavailable'
  if (context.scan_limited) return 'Limited local scan'
  const detected = terminalToolContextDetectedLabels(context)
  return detected.length > 0 ? detected.join(', ') : 'No local tools detected'
}
