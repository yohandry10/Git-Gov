-- v29: Count all Desktop push outcome event types in compliance correlation.
--
-- Event capture fidelity split push outcomes into explicit event_type values
-- (`push_failed`, `governance_blocked_push`, `governance_warned_push`).
-- The compliance dashboard correlation counter must include those outcomes
-- alongside the original attempt/success/block values while preserving the
-- existing JSON shape.

CREATE OR REPLACE FUNCTION get_compliance_dashboard(p_org_id UUID)
RETURNS JSON AS $$
DECLARE
    result JSON;
BEGIN
    SELECT json_build_object(
        'signals', (
            SELECT json_build_object(
                'total', (SELECT COUNT(*) FROM noncompliance_signals WHERE org_id = p_org_id),
                'pending', (SELECT COUNT(*) FROM noncompliance_signals WHERE org_id = p_org_id AND status = 'pending'),
                'high_confidence', (SELECT COUNT(*) FROM noncompliance_signals WHERE org_id = p_org_id AND confidence = 'high'),
                'by_type', COALESCE((SELECT json_object_agg(signal_type, cnt) FROM (SELECT signal_type, COUNT(*) as cnt FROM noncompliance_signals WHERE org_id = p_org_id GROUP BY signal_type) t), '{}'::json)
            )
        ),
        'correlation', (
            SELECT json_build_object(
                'github_pushes_24h', (SELECT COUNT(*) FROM github_events WHERE org_id = p_org_id AND event_type = 'push' AND created_at >= NOW() - INTERVAL '24 hours'),
                'client_pushes_24h', (SELECT COUNT(*) FROM client_events WHERE org_id = p_org_id AND event_type IN ('successful_push', 'attempt_push', 'blocked_push', 'governance_blocked_push', 'governance_warned_push', 'push_failed') AND created_at >= NOW() - INTERVAL '24 hours'),
                'correlation_rate', (
                    SELECT CASE
                        WHEN COUNT(*) > 0 THEN
                            (SELECT COUNT(*) FROM github_events ge WHERE ge.org_id = p_org_id AND ge.event_type = 'push' AND ge.created_at >= NOW() - INTERVAL '24 hours' AND EXISTS (
                                SELECT 1 FROM client_events ce WHERE ce.org_id = p_org_id AND ce.commit_sha = ge.after_sha
                            ))::FLOAT / COUNT(*)
                        ELSE 1.0
                    END
                    FROM github_events WHERE org_id = p_org_id AND event_type = 'push' AND created_at >= NOW() - INTERVAL '24 hours'
                )
            )
        ),
        'policy', (
            SELECT json_build_object(
                'repos_with_policy', (SELECT COUNT(*) FROM policies p JOIN repos r ON p.repo_id = r.id WHERE r.org_id = p_org_id),
                'total_repos', (SELECT COUNT(*) FROM repos WHERE org_id = p_org_id),
                'recent_changes', (SELECT COUNT(*) FROM policy_history ph JOIN repos r ON ph.repo_id = r.id WHERE r.org_id = p_org_id AND ph.created_at >= NOW() - INTERVAL '7 days')
            )
        ),
        'exports', (
            SELECT json_build_object(
                'total', (SELECT COUNT(*) FROM export_logs WHERE org_id = p_org_id),
                'last_7_days', (SELECT COUNT(*) FROM export_logs WHERE org_id = p_org_id AND created_at >= NOW() - INTERVAL '7 days')
            )
        )
    ) INTO result;

    RETURN result;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;
