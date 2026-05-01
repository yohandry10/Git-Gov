import { createGoogleGenerativeAI } from '@ai-sdk/google';
import { generateText } from 'ai';
import { NextRequest, NextResponse } from 'next/server';
import {
    buildDeterministicCopilotBrief,
    buildGovernanceCopilotPrompt,
    buildSourceCitations,
    collectGovernanceCopilotEvidence,
    parseGovernanceCopilotInput,
} from '@/lib/copilot/governance';

export const runtime = 'nodejs';

const MAX_BODY_BYTES = 12 * 1024;
const DEFAULT_GATEWAY_MODEL = 'openai/gpt-5.4';
const DEFAULT_GOOGLE_MODEL = 'gemini-2.5-flash';

type CopilotAiTarget =
    | {
        provider: 'google';
        apiKey: string;
        model: string;
        displayModel: string;
    }
    | {
        provider: 'gateway';
        model: string;
        displayModel: string;
    };

function resolveGitGovAuthorization(request: NextRequest) {
    const authorization = request.headers.get('authorization')?.trim();
    if (authorization && /^Bearer\s+\S+$/i.test(authorization)) {
        return { authorization };
    }

    if (process.env.GITGOV_COPILOT_USE_SERVER_API_KEY !== 'true') {
        return {
            error: NextResponse.json(
                { error: 'Bearer Authorization is required for GitGov evidence access.' },
                { status: 401 },
            ),
        };
    }

    const accessToken = process.env.GITGOV_COPILOT_ACCESS_TOKEN;
    const providedToken = request.headers.get('x-gitgov-copilot-token')?.trim();
    if (!accessToken || providedToken !== accessToken) {
        return {
            error: NextResponse.json(
                { error: 'Copilot access token is required for server-key mode.' },
                { status: 401 },
            ),
        };
    }

    const apiKey = process.env.GITGOV_API_KEY;
    if (!apiKey) {
        return {
            error: NextResponse.json(
                { error: 'GitGov server API key is not configured.' },
                { status: 503 },
            ),
        };
    }

    return { authorization: `Bearer ${apiKey}` };
}

function readCopilotProviderPreference() {
    const provider = process.env.GITGOV_COPILOT_PROVIDER?.trim().toLowerCase();
    if (provider === 'google' || provider === 'gateway' || provider === 'disabled') {
        return provider;
    }
    return 'auto';
}

function normalizeGoogleModel(rawModel: string | undefined) {
    const model = rawModel?.trim();
    if (!model) {
        return DEFAULT_GOOGLE_MODEL;
    }
    if (model.startsWith('google/')) {
        return model.slice('google/'.length) || DEFAULT_GOOGLE_MODEL;
    }
    if (model.includes('/')) {
        return DEFAULT_GOOGLE_MODEL;
    }
    return model;
}

function cleanServerEnvValue(value: string | undefined) {
    return value?.replace(/^\uFEFF/, '').trim();
}

function resolveAiGenerationTarget(): { target?: CopilotAiTarget; warning?: string } {
    if (process.env.GITGOV_COPILOT_DISABLE_AI === 'true') {
        return { warning: 'AI generation skipped because GITGOV_COPILOT_DISABLE_AI is enabled.' };
    }

    const provider = readCopilotProviderPreference();
    if (provider === 'disabled') {
        return { warning: 'AI generation skipped because GITGOV_COPILOT_PROVIDER is disabled.' };
    }

    const googleApiKey = cleanServerEnvValue(
        process.env.GOOGLE_GENERATIVE_AI_API_KEY || process.env.GEMINI_API_KEY,
    );
    if ((provider === 'google' || provider === 'auto') && googleApiKey) {
        const model = normalizeGoogleModel(
            process.env.GITGOV_COPILOT_GOOGLE_MODEL
            || process.env.GEMINI_MODEL
            || process.env.GITGOV_COPILOT_MODEL,
        );
        return {
            target: {
                provider: 'google',
                apiKey: googleApiKey,
                model,
                displayModel: `google/${model}`,
            },
        };
    }

    if (provider === 'google') {
        return { warning: 'AI generation skipped because Google Gemini API key is not configured.' };
    }

    const gatewayReady = Boolean(
        process.env.VERCEL
        || process.env.VERCEL_OIDC_TOKEN
        || process.env.AI_GATEWAY_API_KEY
    );
    if ((provider === 'gateway' || provider === 'auto') && gatewayReady) {
        const model = process.env.GITGOV_COPILOT_GATEWAY_MODEL
            || process.env.GITGOV_COPILOT_MODEL
            || DEFAULT_GATEWAY_MODEL;
        return {
            target: {
                provider: 'gateway',
                model,
                displayModel: model,
            },
        };
    }

    if (provider === 'gateway') {
        return { warning: 'AI generation skipped because AI Gateway/OIDC is not configured.' };
    }

    return { warning: 'AI generation skipped because no configured AI provider is available.' };
}

function readErrorField(error: unknown, field: string) {
    if (!error || typeof error !== 'object') {
        return undefined;
    }
    const value = (error as Record<string, unknown>)[field];
    if (typeof value === 'string' || typeof value === 'number') {
        return `${value}`.replace(/[^\w.-]/g, '').slice(0, 80);
    }
    return undefined;
}

function readNestedErrorField(error: unknown, objectField: string, nestedField: string) {
    if (!error || typeof error !== 'object') {
        return undefined;
    }
    const nested = (error as Record<string, unknown>)[objectField];
    return readErrorField(nested, nestedField);
}

function sanitizeErrorMessage(error: unknown) {
    if (!error || typeof error !== 'object') {
        return undefined;
    }

    const message = (error as Record<string, unknown>).message;
    if (typeof message !== 'string' || !message.trim()) {
        return undefined;
    }

    let sanitized = message;
    for (const secret of [
        process.env.GOOGLE_GENERATIVE_AI_API_KEY,
        process.env.GEMINI_API_KEY,
        process.env.GITGOV_API_KEY,
    ]) {
        if (secret) {
            sanitized = sanitized.replaceAll(secret, '[redacted]');
        }
    }

    sanitized = sanitized
        .replace(/key=[^&\s"']+/gi, 'key=[redacted]')
        .replace(/Bearer\s+[A-Za-z0-9._~+/=-]+/gi, 'Bearer [redacted]')
        .replace(/[^\w\s().:/-]/g, '')
        .replace(/\s+/g, ' ')
        .trim()
        .slice(0, 160);

    return sanitized || undefined;
}

function describeAiGenerationFailure(error: unknown) {
    const details = [
        readErrorField(error, 'name'),
        readErrorField(error, 'code'),
        readErrorField(error, 'statusCode') || readErrorField(error, 'status'),
        readNestedErrorField(error, 'cause', 'name'),
        readNestedErrorField(error, 'cause', 'code'),
        sanitizeErrorMessage(error),
    ].filter(Boolean);

    const suffix = details.length > 0 ? ` (${details.join('/')})` : '';
    return `AI generation was unavailable${suffix}; returned deterministic evidence brief.`;
}

export async function POST(request: NextRequest) {
    const auth = resolveGitGovAuthorization(request);
    if (auth.error) {
        return auth.error;
    }

    try {
        const text = await request.text();
        if (new TextEncoder().encode(text).length > MAX_BODY_BYTES) {
            return NextResponse.json(
                { error: 'Request body is too large.' },
                { status: 413 },
            );
        }

        const parsedBody = JSON.parse(text) as unknown;
        const { input, errors } = parseGovernanceCopilotInput(parsedBody);
        if (!input || errors.length > 0) {
            return NextResponse.json(
                { error: 'Invalid copilot request.', details: errors },
                { status: 400 },
            );
        }

        const evidence = await collectGovernanceCopilotEvidence(
            input,
            auth.authorization,
            process.env.GITGOV_URL || process.env.NEXT_PUBLIC_GITGOV_URL,
        );

        const warnings = [...evidence.warnings];
        const generation = resolveAiGenerationTarget();
        let mode: 'ai' | 'fallback' = 'fallback';
        let answer = buildDeterministicCopilotBrief(input, evidence.sources);
        let model: string | undefined;

        if (generation.target) {
            try {
                const aiModel = generation.target.provider === 'google'
                    ? createGoogleGenerativeAI({ apiKey: generation.target.apiKey })(generation.target.model)
                    : generation.target.model;
                const result = await generateText({
                    model: aiModel,
                    prompt: buildGovernanceCopilotPrompt(input, evidence.sources),
                    maxOutputTokens: 900,
                    temperature: 0.2,
                });
                answer = result.text.trim() || answer;
                mode = 'ai';
                model = generation.target.displayModel;
            } catch (error) {
                warnings.push(describeAiGenerationFailure(error));
            }
        } else {
            warnings.push(generation.warning || 'AI generation skipped because no configured AI provider is available.');
        }

        return NextResponse.json({
            success: true,
            mode,
            model,
            answer,
            citations: buildSourceCitations(evidence.sources),
            sources: evidence.sources,
            warnings,
        });
    } catch {
        return NextResponse.json(
            { error: 'Invalid request body.' },
            { status: 400 },
        );
    }
}
