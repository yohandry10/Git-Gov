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
                'Precios simples y transparentes para equipos que quieren trazabilidad operativa sin improvisar el rollout.',
            footerNote:
                'Precios en USD. Team requiere mínimo 10 seats. Enterprise se cotiza según despliegue, soporte y alcance organizacional.',
        }
        : {
            badge: 'Pricing',
            title: 'Plans &',
            titleAccent: 'Pricing',
            description:
                'Simple, transparent pricing for teams that need operational traceability without improvising the rollout.',
            footerNote:
                'Prices shown in USD. Team requires a 10-seat minimum. Enterprise is scoped around deployment, support, and org-wide rollout.',
        };

    const plans: Plan[] = isEs
        ? [
            {
                name: 'Starter',
                description: 'Empieza con gobernanza Git para equipos pequeños y evaluación inicial.',
                price: 'Gratis',
                priceNote: 'Hasta 5 desarrolladores',
                ctaLabel: 'Empezar',
                ctaHref: '/download',
                icon: <HiOutlineUser size={22} />,
                features: [
                    { label: 'Captura de operaciones Git', included: true },
                    { label: 'Timeline local de evidencia', included: true },
                    { label: 'Hasta 5 usuarios', included: true },
                    { label: 'Correlación Jenkins CI', included: false },
                    { label: 'Cobertura de tickets Jira', included: false },
                    { label: 'Reportes exportables', included: false },
                ],
            },
            {
                name: 'Team',
                description: 'Gobernanza completa para equipos de ingeniería en crecimiento.',
                price: '$24',
                priceNote: 'por desarrollador / mes',
                ctaLabel: 'Contactar por Precios',
                ctaHref: '/contact',
                highlighted: true,
                badge: 'Más popular',
                icon: <HiOutlineLightningBolt size={22} />,
                features: [
                    { label: 'Todo en Starter', included: true },
                    { label: 'Logs de auditoría inmutables', included: true },
                    { label: 'Correlación Jenkins CI', included: true },
                    { label: 'Cobertura de tickets Jira', included: true },
                    { label: 'Verificaciones de gobernanza', included: true },
                    { label: 'Reportes exportables', included: true },
                ],
            },
            {
                name: 'Enterprise',
                description: 'Control total para despliegues regulados y rollout organizacional.',
                price: 'Desde $2,500',
                priceNote: 'por mes',
                ctaLabel: 'Hablar con Ventas',
                ctaHref: '/contact',
                icon: <HiOutlineOfficeBuilding size={22} />,
                features: [
                    { label: 'Todo en Team', included: true },
                    { label: 'Usuarios ilimitados', included: true },
                    { label: 'Soporte prioritario', included: true },
                    { label: 'Onboarding dedicado', included: true },
                    { label: 'Arquitectura híbrida o self-hosted', included: true },
                    { label: 'Rollout de políticas a nivel organización', included: true },
                ],
            },
        ]
        : [
            {
                name: 'Starter',
                description: 'Start Git governance for small teams and early evaluation.',
                price: 'Free',
                priceNote: 'Up to 5 developers',
                ctaLabel: 'Get Started',
                ctaHref: '/download',
                icon: <HiOutlineUser size={22} />,
                features: [
                    { label: 'Git operation capture', included: true },
                    { label: 'Local evidence timeline', included: true },
                    { label: 'Up to 5 users', included: true },
                    { label: 'Jenkins CI correlation', included: false },
                    { label: 'Jira ticket coverage', included: false },
                    { label: 'Exportable reporting', included: false },
                ],
            },
            {
                name: 'Team',
                description: 'Full governance coverage for growing engineering teams.',
                price: '$24',
                priceNote: 'per developer / month',
                ctaLabel: 'Contact for Pricing',
                ctaHref: '/contact',
                highlighted: true,
                badge: 'Most Popular',
                icon: <HiOutlineLightningBolt size={22} />,
                features: [
                    { label: 'Everything in Starter', included: true },
                    { label: 'Immutable audit logs', included: true },
                    { label: 'Jenkins CI correlation', included: true },
                    { label: 'Jira ticket coverage', included: true },
                    { label: 'Governance checks', included: true },
                    { label: 'Exportable reporting', included: true },
                ],
            },
            {
                name: 'Enterprise',
                description: 'Full control for regulated rollouts and org-wide deployment.',
                price: 'From $2,500',
                priceNote: 'per month',
                ctaLabel: 'Talk to Sales',
                ctaHref: '/contact',
                icon: <HiOutlineOfficeBuilding size={22} />,
                features: [
                    { label: 'Everything in Team', included: true },
                    { label: 'Unlimited users', included: true },
                    { label: 'Priority support', included: true },
                    { label: 'Dedicated onboarding', included: true },
                    { label: 'Hybrid or self-hosted architecture', included: true },
                    { label: 'Org-wide policy rollout', included: true },
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
                                            <p className="text-sm text-gray-400 leading-relaxed min-h-[48px]">
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
