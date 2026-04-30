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
const DEFAULT_MODEL = 'openai/gpt-5.4';

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

function shouldAttemptAiGeneration() {
    if (process.env.GITGOV_COPILOT_DISABLE_AI === 'true') {
        return false;
    }

    return Boolean(
        process.env.VERCEL
        || process.env.VERCEL_OIDC_TOKEN
        || process.env.AI_GATEWAY_API_KEY
    );
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
        const model = process.env.GITGOV_COPILOT_MODEL || DEFAULT_MODEL;
        let mode: 'ai' | 'fallback' = 'fallback';
        let answer = buildDeterministicCopilotBrief(input, evidence.sources);

        if (shouldAttemptAiGeneration()) {
            try {
                const result = await generateText({
                    model,
                    prompt: buildGovernanceCopilotPrompt(input, evidence.sources),
                    maxOutputTokens: 900,
                    temperature: 0.2,
                });
                answer = result.text.trim() || answer;
                mode = 'ai';
            } catch {
                warnings.push('AI generation was unavailable; returned deterministic evidence brief.');
            }
        } else {
            warnings.push('AI generation skipped because AI Gateway/OIDC is not configured.');
        }

        return NextResponse.json({
            success: true,
            mode,
            model: mode === 'ai' ? model : undefined,
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
