import { NextRequest, NextResponse } from 'next/server';

/**
 * Contact form API endpoint — Placeholder.
 *
 * In production, this should:
 * - Send email via SendGrid / Resend / etc.
 * - Store in CRM / database
 * - Send Slack notification
 *
 * Currently validates payload and returns 200.
 */
const MAX_BODY_BYTES = 8 * 1024;
const FIELD_LIMITS = {
    name: 120,
    email: 254,
    company: 160,
    teamSize: 80,
    toolchain: 300,
    interestType: 120,
    message: 3000,
} as const;

function readStringField(body: Record<string, unknown>, field: keyof typeof FIELD_LIMITS, required = true) {
    const value = body[field];
    if (typeof value !== 'string') {
        return required ? null : '';
    }

    const trimmed = value.trim();
    if (required && !trimmed) {
        return null;
    }

    return trimmed.slice(0, FIELD_LIMITS[field]);
}

export async function POST(request: NextRequest) {
    try {
        const text = await request.text();
        if (new TextEncoder().encode(text).length > MAX_BODY_BYTES) {
            return NextResponse.json(
                { error: 'Request body is too large' },
                { status: 413 }
            );
        }

        const body = JSON.parse(text) as Record<string, unknown>;

        const name = readStringField(body, 'name');
        const email = readStringField(body, 'email');
        const company = readStringField(body, 'company');
        const teamSize = readStringField(body, 'teamSize');
        const toolchain = readStringField(body, 'toolchain', false) || '';
        const interestType = readStringField(body, 'interestType');
        const message = readStringField(body, 'message');

        if (!name) {
            return NextResponse.json(
                { error: 'Name is required' },
                { status: 400 }
            );
        }

        if (!email || !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
            return NextResponse.json(
                { error: 'Valid email is required' },
                { status: 400 }
            );
        }

        if (!company) {
            return NextResponse.json(
                { error: 'Company is required' },
                { status: 400 }
            );
        }

        if (!teamSize) {
            return NextResponse.json(
                { error: 'Team size is required' },
                { status: 400 }
            );
        }

        if (!interestType) {
            return NextResponse.json(
                { error: 'Interest type is required' },
                { status: 400 }
            );
        }

        if (!message) {
            return NextResponse.json(
                { error: 'Message is required' },
                { status: 400 }
            );
        }

        const emailDomain = email.split('@')[1] || 'unknown';
        console.log('[Contact Form Submission]', {
            emailDomain,
            companyLength: company.length,
            teamSize,
            toolchainLength: toolchain.length,
            interestType,
            messageLength: message.length,
            timestamp: new Date().toISOString(),
        });

        return NextResponse.json(
            { success: true, message: 'Message received. We will get back to you soon.' },
            { status: 200 }
        );
    } catch {
        return NextResponse.json(
            { error: 'Invalid request body' },
            { status: 400 }
        );
    }
}
