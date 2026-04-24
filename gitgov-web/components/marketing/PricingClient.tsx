'use client';

import React from 'react';
import Link from 'next/link';
import { Container } from '@/components/layout';
import { SectionHeader } from '@/components/marketing';
import { SectionReveal } from '@/components/ui';
import { useTranslation } from '@/lib/i18n';
import {
    HiOutlineCheckCircle,
    HiOutlineLightningBolt,
    HiOutlineMail,
    HiOutlineOfficeBuilding,
    HiOutlineUser,
} from 'react-icons/hi';

interface PlanFeature {
    label: string;
    included: boolean;
}

interface Plan {
    name: string;
    description: string;
    price: string;
    priceNote: string;
    features: PlanFeature[];
    ctaLabel: string;
    ctaHref: string;
    highlighted?: boolean;
    badge?: string;
    icon: React.ReactNode;
}

export function PricingClient() {
    const { locale } = useTranslation();
    const isEs = locale === 'es';

    const copy = isEs
        ? {
            badge: 'Precios',
            title: 'Planes y',
            titleAccent: 'Precios',
            description:
                'Precios realistas para el estado actual del producto: entrada mínima para evaluar, un plan de equipo pagable y enterprise cotizado por rollout.',
            footerNote:
                'Precios en USD. Starter es solo una evaluación limitada. Team cubre hasta 10 desarrolladores por workspace. Enterprise se cotiza según despliegue, soporte y alcance organizacional.',
        }
        : {
            badge: 'Pricing',
            title: 'Plans &',
            titleAccent: 'Pricing',
            description:
                'Realistic pricing for the current product maturity: minimal entry for evaluation, an affordable team plan, and enterprise scoped around rollout.',
            footerNote:
                'Prices shown in USD. Starter is a limited evaluation tier. Team covers up to 10 developers per workspace. Enterprise is scoped around deployment, support, and org-wide rollout.',
        };

    const plans: Plan[] = isEs
        ? [
            {
                name: 'Starter',
                description: 'Evaluación corta para validar la captura local y el flujo básico antes de hablar con ventas.',
                price: 'Gratis',
                priceNote: '1 developer · 1 día de prueba',
                ctaLabel: 'Probar',
                ctaHref: '/download',
                icon: <HiOutlineUser size={22} />,
                features: [
                    { label: '1 developer', included: true },
                    { label: '1 día de evaluación', included: true },
                    { label: 'Captura de operaciones Git', included: true },
                    { label: 'Timeline local de evidencia', included: true },
                    { label: 'Correlación Jenkins CI', included: false },
                    { label: 'Cobertura de tickets Jira', included: false },
                ],
            },
            {
                name: 'Team',
                description: 'El plan operativo para equipos que ya quieren usar GitGov en trabajo real y no solo en una demo.',
                price: '$299',
                priceNote: 'por workspace / mes',
                ctaLabel: 'Contactar por Precios',
                ctaHref: '/contact',
                highlighted: true,
                badge: 'Más popular',
                icon: <HiOutlineLightningBolt size={22} />,
                features: [
                    { label: 'Hasta 10 developers', included: true },
                    { label: 'Logs de auditoría inmutables', included: true },
                    { label: 'Correlación Jenkins CI', included: true },
                    { label: 'Cobertura de tickets Jira', included: true },
                    { label: 'Verificaciones de gobernanza', included: true },
                    { label: 'Reportes exportables', included: true },
                ],
            },
            {
                name: 'Enterprise',
                description: 'Para despliegues self-hosted, rollout por áreas, soporte prioritario y operación organizacional.',
                price: 'Desde $1,200',
                priceNote: 'por mes',
                ctaLabel: 'Hablar con Ventas',
                ctaHref: '/contact',
                icon: <HiOutlineOfficeBuilding size={22} />,
                features: [
                    { label: 'Todo en Team', included: true },
                    { label: 'Usuarios ilimitados', included: true },
                    { label: 'Arquitectura self-hosted o híbrida', included: true },
                    { label: 'Soporte prioritario', included: true },
                    { label: 'Onboarding dedicado', included: true },
                    { label: 'Acompañamiento de rollout', included: true },
                ],
            },
        ]
        : [
            {
                name: 'Starter',
                description: 'Short evaluation tier to validate local capture and the basic workflow before talking to sales.',
                price: 'Free',
                priceNote: '1 developer · 1 day trial',
                ctaLabel: 'Try It',
                ctaHref: '/download',
                icon: <HiOutlineUser size={22} />,
                features: [
                    { label: '1 developer', included: true },
                    { label: '1 day evaluation', included: true },
                    { label: 'Git operation capture', included: true },
                    { label: 'Local evidence timeline', included: true },
                    { label: 'Jenkins CI correlation', included: false },
                    { label: 'Jira ticket coverage', included: false },
                ],
            },
            {
                name: 'Team',
                description: 'The operational plan for teams that want GitGov in real work, not just in a demo.',
                price: '$299',
                priceNote: 'per workspace / month',
                ctaLabel: 'Contact for Pricing',
                ctaHref: '/contact',
                highlighted: true,
                badge: 'Most Popular',
                icon: <HiOutlineLightningBolt size={22} />,
                features: [
                    { label: 'Up to 10 developers', included: true },
                    { label: 'Immutable audit logs', included: true },
                    { label: 'Jenkins CI correlation', included: true },
                    { label: 'Jira ticket coverage', included: true },
                    { label: 'Governance checks', included: true },
                    { label: 'Exportable reporting', included: true },
                ],
            },
            {
                name: 'Enterprise',
                description: 'For self-hosted deployments, phased rollout, priority support, and org-wide operation.',
                price: 'From $1,200',
                priceNote: 'per month',
                ctaLabel: 'Talk to Sales',
                ctaHref: '/contact',
                icon: <HiOutlineOfficeBuilding size={22} />,
                features: [
                    { label: 'Everything in Team', included: true },
                    { label: 'Unlimited users', included: true },
                    { label: 'Self-hosted or hybrid architecture', included: true },
                    { label: 'Priority support', included: true },
                    { label: 'Dedicated onboarding', included: true },
                    { label: 'Rollout advisory', included: true },
                ],
            },
        ];

    return (
        <>
            <section className="pt-32 md:pt-40 pb-16 relative overflow-hidden">
                <div className="absolute inset-0">
                    <div
                        className="absolute inset-0 opacity-[0.03]"
                        style={{
                            backgroundImage: `linear-gradient(rgba(249,115,22,0.2) 1px, transparent 1px), linear-gradient(90deg, rgba(249,115,22,0.2) 1px, transparent 1px)`,
                            backgroundSize: '40px 40px',
                        }}
                    />
                    <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[680px] h-[340px] bg-brand-500/5 rounded-full blur-3xl pointer-events-none" />
                </div>
                <Container>
                    <SectionHeader
                        badge={copy.badge}
                        title={copy.title}
                        titleAccent={copy.titleAccent}
                        description={copy.description}
                    />
                </Container>
            </section>

            <section className="pb-28">
                <Container>
                    <SectionReveal>
                        <div className="max-w-6xl mx-auto grid lg:grid-cols-3 gap-6 items-stretch">
                            {plans.map((plan) => (
                                <div
                                    key={plan.name}
                                    className={`rounded-[2rem] p-[1px] ${
                                        plan.highlighted
                                            ? 'bg-gradient-to-b from-brand-500/40 via-brand-500/20 to-transparent shadow-[0_0_30px_rgba(249,115,22,0.12)]'
                                            : 'bg-gradient-to-b from-white/10 to-transparent'
                                    }`}
                                >
                                    <div
                                        className={`h-full rounded-[31px] border p-8 md:p-9 flex flex-col ${
                                            plan.highlighted
                                                ? 'border-brand-500/20 bg-[radial-gradient(circle_at_top,rgba(249,115,22,0.12),rgba(9,9,9,0.96)_55%)]'
                                                : 'border-white/5 bg-[#090909]'
                                        }`}
                                    >
                                        <div className="flex items-center justify-between mb-8">
                                            <div className="w-12 h-12 rounded-2xl bg-white/5 border border-white/10 flex items-center justify-center text-brand-400">
                                                {plan.icon}
                                            </div>
                                            {plan.badge && (
                                                <span className="inline-flex items-center px-3 py-1 rounded-full text-[11px] font-bold uppercase tracking-widest bg-brand-500/10 text-brand-400 border border-brand-500/20">
                                                    {plan.badge}
                                                </span>
                                            )}
                                        </div>

                                        <div className="mb-8">
                                            <h2 className="text-3xl font-black text-white tracking-tight mb-3">
                                                {plan.name}
                                            </h2>
                                            <p className="text-sm text-gray-400 leading-relaxed min-h-[72px]">
                                                {plan.description}
                                            </p>
                                        </div>

                                        <div className="mb-8">
                                            <div className="text-5xl md:text-6xl font-black tracking-tight text-white leading-none">
                                                {plan.price}
                                            </div>
                                            <p className="mt-3 text-xs font-bold uppercase tracking-widest text-brand-400">
                                                {plan.priceNote}
                                            </p>
                                        </div>

                                        <div className="pt-6 border-t border-white/6 mb-8">
                                            <ul className="space-y-4">
                                                {plan.features.map((feature) => (
                                                    <li key={feature.label} className="flex items-start gap-3">
                                                        <HiOutlineCheckCircle
                                                            size={18}
                                                            className={feature.included ? 'text-brand-400 mt-0.5 shrink-0' : 'text-gray-700 mt-0.5 shrink-0'}
                                                        />
                                                        <span
                                                            className={`text-sm leading-relaxed ${
                                                                feature.included ? 'text-gray-200' : 'text-gray-600'
                                                            }`}
                                                        >
                                                            {feature.label}
                                                        </span>
                                                    </li>
                                                ))}
                                            </ul>
                                        </div>

                                        <Link
                                            href={plan.ctaHref}
                                            className={`mt-auto inline-flex items-center justify-center gap-2 rounded-xl px-5 py-4 font-bold transition-all duration-300 ${
                                                plan.highlighted
                                                    ? 'bg-brand-500 text-white shadow-[0_0_24px_rgba(249,115,22,0.22)] hover:shadow-[0_0_34px_rgba(249,115,22,0.34)]'
                                                    : 'bg-black text-white border border-white/8 hover:border-white/16 hover:bg-white/[0.03]'
                                            }`}
                                        >
                                            <HiOutlineMail size={18} />
                                            {plan.ctaLabel}
                                        </Link>
                                    </div>
                                </div>
                            ))}
                        </div>
                    </SectionReveal>

                    <SectionReveal delay={0.08}>
                        <p className="max-w-4xl mx-auto mt-10 text-center text-sm text-gray-500 leading-relaxed">
                            {copy.footerNote}
                        </p>
                    </SectionReveal>
                </Container>
            </section>
        </>
    );
}
