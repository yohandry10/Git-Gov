import type {
  DeploymentGateAuthorizationRecord,
  ServerConfig,
} from '@/store/useControlPlaneStore/types'
import type { TerminalGovernanceTarget } from './terminalGovernanceContext'

export type TerminalBranchGateStatusTone = 'ready' | 'review' | 'muted'

export interface TerminalBranchGateStatusSummary {
  label: string
  title: string
  tone: TerminalBranchGateStatusTone
  visible: boolean
}

const ADVISORY_PREFIX = 'Advisory only. Does not block terminal commands, commits, pushes, or deployments.'

function normalizeDecision(value: unknown): string {
  return typeof value === 'string' ? value.trim().toLowerCase() : ''
}

function nestedDecision(value: unknown): string {
  if (!value || typeof value !== 'object') return ''
  const decision = (value as { decision?: unknown }).decision
  return normalizeDecision(decision)
}

export function terminalBranchGateInitialStatus(
  target: TerminalGovernanceTarget,
  serverConfig: ServerConfig | null,
): TerminalBranchGateStatusSummary {
  if (target.status === 'pending') {
    return {
      label: 'Gate...',
      title: `${ADVISORY_PREFIX} Waiting for terminal Git context.`,
      tone: 'muted',
      visible: false,
    }
  }

  if (target.status === 'no-git-repo' || target.status === 'missing-remote') {
    return {
      label: 'Gate n/a',
      title: `${ADVISORY_PREFIX} GitGov cannot map this terminal location to Control Plane gate evidence yet.`,
      tone: 'muted',
      visible: false,
    }
  }

  if (!serverConfig) {
    return {
      label: 'Gate n/a',
      title: `${ADVISORY_PREFIX} Control Plane is not configured for this desktop session.`,
      tone: 'muted',
      visible: true,
    }
  }

  return {
    label: 'Gate...',
    title: `${ADVISORY_PREFIX} Loading latest branch gate evidence.`,
    tone: 'muted',
    visible: true,
  }
}

export function summarizeTerminalBranchGateStatus(
  authorization: DeploymentGateAuthorizationRecord | null,
): TerminalBranchGateStatusSummary {
  if (!authorization) {
    return {
      label: 'No gate',
      title: `${ADVISORY_PREFIX} No Deployment Gate authorization was found for this repository and branch.`,
      tone: 'muted',
      visible: true,
    }
  }

  const decision = normalizeDecision(authorization.decision)
  const governanceDecision = nestedDecision(authorization.governance_decision)
  const detailDecision = nestedDecision(authorization.details?.governance_decision)
  const sharedDecision = nestedDecision(authorization.details?.shared_governance_decision)
  const allDecisions = [decision, governanceDecision, detailDecision, sharedDecision].filter(Boolean)
  const needsReview =
    authorization.approved === false ||
    authorization.blocking ||
    authorization.would_block ||
    allDecisions.some((item) =>
      [
        'blocked',
        'would_block',
        'would-block',
        'requires_approval',
        'insufficient_evidence',
        'missing_evidence',
        'denied',
        'rejected',
      ].includes(item),
    )

  if (needsReview) {
    return {
      label: 'Gate review',
      title: `${ADVISORY_PREFIX} Latest gate evidence needs manual review: ${authorization.reason || authorization.decision}.`,
      tone: 'review',
      visible: true,
    }
  }

  if (authorization.approved) {
    return {
      label: 'Gate ready',
      title: `${ADVISORY_PREFIX} Latest gate evidence is approved for ${authorization.environment || 'this environment'}.`,
      tone: 'ready',
      visible: true,
    }
  }

  return {
    label: 'Gate check',
    title: `${ADVISORY_PREFIX} Latest gate decision is ${authorization.decision || 'unknown'} and should be checked manually.`,
    tone: 'muted',
    visible: true,
  }
}

export function terminalBranchGateErrorStatus(message: string): TerminalBranchGateStatusSummary {
  return {
    label: 'Gate n/a',
    title: `${ADVISORY_PREFIX} Latest gate evidence is unavailable: ${message}`,
    tone: 'muted',
    visible: true,
  }
}
