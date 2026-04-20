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
export async function POST(request: NextRequest) {
    try {
        const body = await request.json();

        const { name, email, company, teamSize, toolchain, interestType, message } = body;

        // Basic validation
        if (!name || typeof name !== 'string' || !name.trim()) {
            return NextResponse.json(
                { error: 'Name is required' },
                { status: 400 }
            );
        }

        if (!email || typeof email !== 'string' || !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
            return NextResponse.json(
                { error: 'Valid email is required' },
                { status: 400 }
            );
        }

        if (!company || typeof company !== 'string' || !company.trim()) {
            return NextResponse.json(
                { error: 'Company is required' },
                { status: 400 }
            );
        }

        if (!teamSize || typeof teamSize !== 'string' || !teamSize.trim()) {
            return NextResponse.json(
                { error: 'Team size is required' },
                { status: 400 }
            );
        }

        if (!interestType || typeof interestType !== 'string' || !interestType.trim()) {
            return NextResponse.json(
                { error: 'Interest type is required' },
                { status: 400 }
            );
        }

        if (!message || typeof message !== 'string' || !message.trim()) {
            return NextResponse.json(
                { error: 'Message is required' },
                { status: 400 }
            );
        }

        // Log in development (placeholder for real service)
        console.log('[Contact Form Submission]', {
            name: name.trim(),
            email: email.trim(),
            company: (company || '').trim(),
            teamSize: (teamSize || '').trim(),
            toolchain: (toolchain || '').trim(),
            interestType: (interestType || '').trim(),
            message: message.trim(),
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
