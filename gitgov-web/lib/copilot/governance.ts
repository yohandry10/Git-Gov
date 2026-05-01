export type GovernanceCopilotInput = {
    question: string;
    orgName?: string;
    repositoryFullName?: string;
    branch?: string;
    ticketId?: string;
    releaseId?: string;
    environment?: string;
    hours: number;
};

export type CopilotSourceStatus = 'ok' | 'missing' | 'error' | 'skipped';

export type GovernanceCopilotSource = {
    id: string;
    label: string;
    endpoint: string;
    status: CopilotSourceStatus;
    httpStatus?: number;
    summary: unknown;
};

export type GovernanceCopilotEvidence = {
    sources: GovernanceCopilotSource[];
    warnings: string[];
};

const MAX_QUESTION_LENGTH = 2000;
const DEFAULT_HOURS = 720;
const MAX_HOURS = 24 * 366;
const MAX_TEXT_FIELD_LENGTH = 240;

const REPO_FULL_NAME_RE = /^[^\s/]+\/[^\s/]+$/;
const TICKET_ID_RE = /^[A-Z][A-Z0-9]+-[1-9][0-9]*$/;
const SHA256_RE = /^[a-fA-F0-9]{64}$/;

function cleanText(value: string, maxLength = MAX_TEXT_FIELD_LENGTH) {
    return value.replace(/[\u0000-\u001f\u007f]/g, '').trim().slice(0, maxLength);
}

function readOptionalString(body: Record<string, unknown>, ...keys: string[]) {
    for (const key of keys) {
        const value = body[key];
        if (typeof value === 'string') {
            const cleaned = cleanText(value);
            if (cleaned) return cleaned;
        }
    }
    return undefined;
}

export function parseGovernanceCopilotInput(body: unknown): { input?: GovernanceCopilotInput; errors: string[] } {
    const errors: string[] = [];

    if (!body || typeof body !== 'object' || Array.isArray(body)) {
        return { errors: ['Request body must be a JSON object.'] };
    }

    const record = body as Record<string, unknown>;
    const questionRaw = readOptionalString(record, 'question', 'message', 'prompt');
    if (!questionRaw) {
        errors.push('question is required.');
    }

    const question = questionRaw?.slice(0, MAX_QUESTION_LENGTH) ?? '';
    const orgName = readOptionalString(record, 'orgName', 'org_name');
    const repositoryFullName = readOptionalString(record, 'repositoryFullName', 'repository_full_name', 'repoFullName', 'repo_full_name');
    const branch = readOptionalString(record, 'branch');
    const ticketId = readOptionalString(record, 'ticketId', 'ticket_id')?.toUpperCase();
    const releaseId = readOptionalString(record, 'releaseId', 'release_id');
    const environment = readOptionalString(record, 'environment')?.toLowerCase();

    if (repositoryFullName && !REPO_FULL_NAME_RE.test(repositoryFullName)) {
        errors.push('repository_full_name must look like owner/repo.');
    }

    if (ticketId && !TICKET_ID_RE.test(ticketId)) {
        errors.push('ticket_id must look like KAN-38.');
    }

    let hours = DEFAULT_HOURS;
    const rawHours = record.hours;
    if (typeof rawHours === 'number' && Number.isFinite(rawHours)) {
        hours = Math.trunc(rawHours);
    } else if (typeof rawHours === 'string' && rawHours.trim()) {
        const parsed = Number.parseInt(rawHours, 10);
        if (Number.isFinite(parsed)) {
            hours = parsed;
        }
    }
    hours = Math.min(Math.max(hours, 1), MAX_HOURS);

    if (errors.length > 0) {
        return { errors };
    }

    return {
        input: {
            question,
            orgName,
            repositoryFullName,
            branch,
            ticketId,
            releaseId,
            environment,
            hours,
        },
        errors: [],
    };
}

export function normalizeGitGovBaseUrl(rawUrl: string | undefined) {
    const fallback = 'https://gitgov-api.onrender.com';
    const candidate = cleanText(rawUrl || fallback, 500) || fallback;
    const url = new URL(candidate);
    if (!['http:', 'https:'].includes(url.protocol)) {
        throw new Error('GITGOV_URL must use http or https.');
    }
    url.username = '';
    url.password = '';
    url.pathname = url.pathname.replace(/\/+$/, '');
    url.search = '';
    url.hash = '';
    return url.toString().replace(/\/+$/, '');
}

function endpointWithQuery(path: string, params: Record<string, string | number | undefined>) {
    const query = new URLSearchParams();
    for (const [key, value] of Object.entries(params)) {
        if (value !== undefined && `${value}`.trim()) {
            query.set(key, `${value}`);
        }
    }

    const qs = query.toString();
    return qs ? `${path}?${qs}` : path;
}

function evidencePacketSummary(data: unknown) {
    const packet = readObject(readObject(data).packet);
    const completeness = readObject(packet.completeness);
    return {
        found: Boolean(readObject(data).found),
        subject: readString(packet.subject),
        content_hash: readHash(packet.content_hash),
        completeness: {
            ticket_found: Boolean(completeness.ticket_found),
            commits: readNumber(completeness.commits),
            pull_requests: readNumber(completeness.pull_requests),
            pipelines: readNumber(completeness.pipelines),
            quality_gates: readNumber(completeness.quality_gates),
            missing: readStringArray(completeness.missing, 12),
        },
    };
}

function ticketCoverageSummary(data: unknown) {
    const record = readObject(data);
    return {
        org: readString(record.org),
        period: readString(record.period),
        total_commits: readNumber(record.total_commits),
        commits_with_ticket: readNumber(record.commits_with_ticket),
        coverage_percentage: readNumber(record.coverage_percentage),
        commits_without_ticket_count: readArray(record.commits_without_ticket).length,
        tickets_without_commits_count: readArray(record.tickets_without_commits).length,
    };
}

function releaseApprovalSummary(data: unknown) {
    const record = readObject(data);
    const items = readArray(record.items)
        .slice(0, 5)
        .map((item) => {
            const approval = readObject(item);
            return {
                release_id: readString(approval.release_id),
                repository_full_name: readString(approval.repository_full_name),
                environment: readString(approval.environment),
                decision: readString(approval.decision),
                approver: readString(approval.approver),
                ticket_id: readString(approval.ticket_id),
                risk_severity: readString(approval.risk_severity),
                approval_hash: readHash(approval.approval_hash),
                expires_at: readNumberOrNull(approval.expires_at),
                created_at: readNumberOrNull(approval.created_at),
            };
        });

    return {
        total: readNumber(record.total),
        items,
    };
}

function adoptionProfileSummary(data: unknown) {
    const record = readObject(data);
    const profile = readObject(readObject(record.profile).profile);
    return {
        found: Boolean(record.found),
        customer_name: readString(profile.customer_name),
        repository_full_name: readString(profile.repository_full_name),
        default_branch: readString(profile.default_branch),
        policy_preset: readString(profile.policy_preset),
        providers: readStringArray(profile.providers, 12),
        modules: readStringArray(profile.modules, 16),
    };
}

function readObject(value: unknown): Record<string, unknown> {
    return value && typeof value === 'object' && !Array.isArray(value) ? value as Record<string, unknown> : {};
}

function readArray(value: unknown): unknown[] {
    return Array.isArray(value) ? value : [];
}

function readString(value: unknown) {
    return typeof value === 'string' ? cleanText(value, 500) : undefined;
}

function readHash(value: unknown) {
    return typeof value === 'string' && SHA256_RE.test(value) ? value : undefined;
}

function readNumber(value: unknown) {
    return typeof value === 'number' && Number.isFinite(value) ? value : 0;
}

function readNumberOrNull(value: unknown) {
    return typeof value === 'number' && Number.isFinite(value) ? value : null;
}

function readStringArray(value: unknown, limit: number) {
    return readArray(value)
        .filter((item): item is string => typeof item === 'string')
        .map((item) => cleanText(item, 120))
        .filter(Boolean)
        .slice(0, limit);
}

async function fetchGitGovJson(baseUrl: string, authorization: string, endpoint: string) {
    const url = new URL(endpoint, `${baseUrl}/`);
    const response = await fetch(url, {
        method: 'GET',
        headers: {
            Accept: 'application/json',
            Authorization: authorization,
        },
        cache: 'no-store',
    });

    const text = await response.text();
    if (!response.ok) {
        return {
            ok: false,
            status: response.status,
            data: undefined,
        };
    }

    return {
        ok: true,
        status: response.status,
        data: text ? JSON.parse(text) as unknown : {},
    };
}

async function collectSource(
    baseUrl: string,
    authorization: string,
    source: Pick<GovernanceCopilotSource, 'id' | 'label' | 'endpoint'>,
    summarize: (data: unknown) => unknown,
): Promise<GovernanceCopilotSource> {
    try {
        const result = await fetchGitGovJson(baseUrl, authorization, source.endpoint);
        if (!result.ok) {
            return {
                ...source,
                status: result.status === 404 ? 'missing' : 'error',
                httpStatus: result.status,
                summary: { error: `GitGov returned HTTP ${result.status}` },
            };
        }

        return {
            ...source,
            status: 'ok',
            httpStatus: result.status,
            summary: summarize(result.data),
        };
    } catch {
        return {
            ...source,
            status: 'error',
            summary: { error: 'GitGov evidence fetch failed' },
        };
    }
}

export async function collectGovernanceCopilotEvidence(
    input: GovernanceCopilotInput,
    authorization: string,
    rawBaseUrl: string | undefined,
): Promise<GovernanceCopilotEvidence> {
    const baseUrl = normalizeGitGovBaseUrl(rawBaseUrl);
    const warnings: string[] = [];
    const sources: GovernanceCopilotSource[] = [];

    if (input.ticketId) {
        sources.push(await collectSource(
            baseUrl,
            authorization,
            {
                id: 'evidence-packet',
                label: `Evidence Packet ${input.ticketId}`,
                endpoint: endpointWithQuery(`/evidence/packets/tickets/${encodeURIComponent(input.ticketId)}`, {
                    org_name: input.orgName,
                    repo_full_name: input.repositoryFullName,
                    branch: input.branch,
                    hours: input.hours,
                }),
            },
            evidencePacketSummary,
        ));
    } else {
        sources.push({
            id: 'evidence-packet',
            label: 'Evidence Packet',
            endpoint: '/evidence/packets/tickets/{ticket_id}',
            status: 'skipped',
            summary: { reason: 'ticket_id was not provided' },
        });
        warnings.push('Evidence Packet was skipped because ticket_id was not provided.');
    }

    sources.push(await collectSource(
        baseUrl,
        authorization,
        {
            id: 'ticket-coverage',
            label: 'Jira ticket coverage',
            endpoint: endpointWithQuery('/integrations/jira/ticket-coverage', {
                org_name: input.orgName,
                repo_full_name: input.repositoryFullName,
                branch: input.branch,
                hours: input.hours,
            }),
        },
        ticketCoverageSummary,
    ));

    sources.push(await collectSource(
        baseUrl,
        authorization,
        {
            id: 'release-approvals',
            label: 'Formal release approvals',
            endpoint: endpointWithQuery('/enterprise/release-approvals', {
                org_name: input.orgName,
                repository_full_name: input.repositoryFullName,
                release_id: input.releaseId || input.ticketId,
                environment: input.environment,
                limit: 5,
            }),
        },
        releaseApprovalSummary,
    ));

    if (input.orgName) {
        sources.push(await collectSource(
            baseUrl,
            authorization,
            {
                id: 'adoption-profile',
                label: 'Enterprise adoption profile',
                endpoint: endpointWithQuery('/enterprise/adoption-profile', {
                    org_name: input.orgName,
                }),
            },
            adoptionProfileSummary,
        ));
    }

    return { sources, warnings };
}

export function buildGovernanceCopilotPrompt(input: GovernanceCopilotInput, sources: GovernanceCopilotSource[]) {
    return [
        'You are GitGov Copilot, an evidence-grounded governance assistant.',
        'Answer in the same language as the user question.',
        'Use only the provided evidence sources. If evidence is missing, say exactly what is missing.',
        'Cite source IDs inline using [source:id]. Do not cite sources that are missing or skipped.',
        'Do not invent approvals, provider state, vulnerabilities, release readiness, or production status.',
        'Do not expose secrets, tokens, raw Authorization headers, or provider credentials.',
        '',
        `Question: ${input.question}`,
        '',
        'Request context:',
        JSON.stringify({
            org_name: input.orgName,
            repository_full_name: input.repositoryFullName,
            branch: input.branch,
            ticket_id: input.ticketId,
            release_id: input.releaseId,
            environment: input.environment,
            hours: input.hours,
        }, null, 2),
        '',
        'Evidence sources:',
        JSON.stringify(sources, null, 2),
        '',
        'Response shape:',
        '- Direct answer first.',
        '- Then bullets for blockers, evidence, and next action when relevant.',
        '- Keep it concise and operational.',
    ].join('\n');
}

export function buildDeterministicCopilotBrief(input: GovernanceCopilotInput, sources: GovernanceCopilotSource[]) {
    const usable = sources.filter((source) => source.status === 'ok');
    const missing = sources.filter((source) => source.status !== 'ok');
    const lines = [
        `AI generation is unavailable, so this is a deterministic GitGov evidence brief for: ${input.question}`,
        '',
        'Evidence loaded:',
        ...usable.map((source) => `- ${source.label} [source:${source.id}]`),
    ];

    if (missing.length > 0) {
        lines.push('', 'Evidence not available:');
        lines.push(...missing.map((source) => `- ${source.label}: ${source.status}`));
    }

    const approvalSource = usable.find((source) => source.id === 'release-approvals');
    const evidenceSource = usable.find((source) => source.id === 'evidence-packet');
    if (approvalSource || evidenceSource) {
        lines.push('', 'Key evidence:');
        if (approvalSource) {
            lines.push(`- Formal release approval data is present [source:${approvalSource.id}].`);
        }
        if (evidenceSource) {
            lines.push(`- Evidence Packet data is present [source:${evidenceSource.id}].`);
        }
    }

    lines.push('', 'Next action: configure Google Gemini or Vercel AI Gateway for generated narrative answers.');
    return lines.join('\n');
}

export function buildSourceCitations(sources: GovernanceCopilotSource[]) {
    return sources.map((source) => ({
        id: source.id,
        label: source.label,
        endpoint: source.endpoint,
        status: source.status,
        httpStatus: source.httpStatus,
    }));
}
