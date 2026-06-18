import { Link } from 'react-router-dom'
import { Badge } from '@/components/shared/Badge'
import {
  providerSetupActionBadgeVariant,
  providerSetupStatusClass,
} from './enterprise-adoption-panel-helpers'
import {
  type AdoptionProvider,
  type EnterpriseProviderSetupDecisionKind,
  type EnterpriseProviderSetupGuidance,
  type EnterpriseProviderSetupStep,
} from './dashboard-helpers'

function providerSetupDecisionKindForStep(step: EnterpriseProviderSetupStep): EnterpriseProviderSetupDecisionKind {
  if (!step.selected) return 'intentionally-skipped'
  if (step.status === 'ready') return 'reviewed'
  return 'retry-later'
}

function providerSetupDecisionButtonLabel(decision: EnterpriseProviderSetupDecisionKind): string {
  if (decision === 'reviewed') return 'Reviewed'
  if (decision === 'intentionally-skipped') return 'Remember'
  return 'Later'
}

interface EnterpriseProviderSetupPanelProps {
  guidance: EnterpriseProviderSetupGuidance
  onDecision: (provider: AdoptionProvider, decision: EnterpriseProviderSetupDecisionKind) => void
  onClearDecision: (provider: AdoptionProvider) => void
}

export function EnterpriseProviderSetupPanel({
  guidance,
  onDecision,
  onClearDecision,
}: EnterpriseProviderSetupPanelProps) {
  return (
    <div role="region" aria-label="Provider setup guidance" className="rounded border border-white/8 bg-white/[0.03]">
      <div className="flex flex-wrap items-center justify-between gap-2 border-b border-white/5 px-3 py-2">
        <div>
          <div className="text-[10px] uppercase tracking-widest text-surface-500">Provider setup</div>
          <div className="mt-1 text-[11px] text-surface-400">
            {guidance.ready_count}/{guidance.selected_count} selected ready, {guidance.skipped_count} skipped
          </div>
        </div>
        {guidance.next_step ? (
          <Badge variant={providerSetupActionBadgeVariant(guidance.next_step.action)}>
            Next: {guidance.next_step.action_label}
          </Badge>
        ) : (
          <Badge variant="success">Ready</Badge>
        )}
      </div>
      {guidance.next_step && (
        <div className="border-b border-white/5 px-3 py-2 text-[11px] leading-5 text-surface-300">
          <span className="font-medium text-surface-100">{guidance.next_step.label}: </span>
          {guidance.next_step.validation}
        </div>
      )}
      <div className="divide-y divide-white/5">
        {guidance.steps.map((step) => {
          const decisionKind = providerSetupDecisionKindForStep(step)
          return (
            <div
              key={step.provider}
              data-provider={step.provider}
              className={`grid grid-cols-1 gap-2 px-3 py-2 sm:grid-cols-[120px_minmax(0,1fr)_260px] ${providerSetupStatusClass(step.status)}`}
            >
              <div className="text-xs font-medium text-surface-100">{step.label}</div>
              <div className="min-w-0 text-[11px] leading-5 text-surface-400">
                <div>{step.reason}</div>
                <div className="text-[10px] text-surface-500">{step.validation}</div>
              </div>
              <div className="flex flex-wrap items-center gap-2 sm:justify-end">
                <Badge variant={providerSetupActionBadgeVariant(step.action)}>
                  {step.action_label}
                </Badge>
                <Link
                  to={step.target.to}
                  aria-label={`${step.target.label} for ${step.label}`}
                  className="whitespace-nowrap rounded border border-white/10 px-2 py-1 text-[10px] font-medium text-surface-300 transition-colors hover:border-white/25 hover:bg-white/[0.04] hover:text-surface-100"
                >
                  {step.target.label}
                </Link>
                {step.operator_decision ? (
                  <>
                    <Badge variant="neutral">
                      {step.operator_decision_label}
                    </Badge>
                    <button
                      type="button"
                      onClick={() => onClearDecision(step.provider)}
                      className="whitespace-nowrap rounded border border-white/10 px-2 py-1 text-[10px] font-medium text-surface-400 transition-colors hover:border-white/25 hover:bg-white/[0.04] hover:text-surface-100"
                      title={`Clear ${step.label} setup decision`}
                    >
                      Clear
                    </button>
                  </>
                ) : (
                  <button
                    type="button"
                    onClick={() => onDecision(step.provider, decisionKind)}
                    className="whitespace-nowrap rounded border border-white/10 px-2 py-1 text-[10px] font-medium text-surface-300 transition-colors hover:border-white/25 hover:bg-white/[0.04] hover:text-surface-100"
                    title={`Remember ${step.label} setup decision`}
                  >
                    {providerSetupDecisionButtonLabel(decisionKind)}
                  </button>
                )}
              </div>
            </div>
          )
        })}
      </div>
    </div>
  )
}
