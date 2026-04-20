'use client';

import React from 'react';
import Link from 'next/link';
import { Container } from '@/components/layout';
import { CTASection, SectionHeader } from '@/components/marketing';
import { SectionReveal } from '@/components/ui';
import { useTranslation } from '@/lib/i18n';
import {
    HiOutlineCloud,
    HiOutlineShieldCheck,
    HiOutlineOfficeBuilding,
    HiOutlineCheckCircle,
    HiOutlineArrowRight,
    HiOutlineClipboardCheck,
} from 'react-icons/hi';

interface DeploymentOption {
    title: string;
    description: string;
    icon: React.ReactNode;
}

interface ProcessStep {
    title: string;
    description: string;
}

export function PricingClient() {
    const { t } = useTranslation();

    const deploymentOptions: DeploymentOption[] = [
        {
            title: t('pricing.deployment.self.title') as string,
            description: t('pricing.deployment.self.desc') as string,
            icon: <HiOutlineShieldCheck size={22} />,
        },
        {
            title: t('pricing.deployment.managed.title') as string,
            description: t('pricing.deployment.managed.desc') as string,
            icon: <HiOutlineCloud size={22} />,
        },
        {
            title: t('pricing.deployment.hybrid.title') as string,
            description: t('pricing.deployment.hybrid.desc') as string,
            icon: <HiOutlineOfficeBuilding size={22} />,
        },
    ];

    const qualificationPoints = [
        t('pricing.fit.item1') as string,
        t('pricing.fit.item2') as string,
        t('pricing.fit.item3') as string,
    ];

    const processSteps: ProcessStep[] = [
        {
            title: t('pricing.process.step1.title') as string,
            description: t('pricing.process.step1.desc') as string,
        },
        {
            title: t('pricing.process.step2.title') as string,
            description: t('pricing.process.step2.desc') as string,
        },
        {
            title: t('pricing.process.step3.title') as string,
            description: t('pricing.process.step3.desc') as string,
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
                    <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[640px] h-[320px] bg-brand-500/5 rounded-full blur-3xl pointer-events-none" />
                </div>
                <Container>
                    <SectionHeader
                        badge={t('pricing.badge') as string}
                        title={t('pricing.title') as string}
                        titleAccent={t('pricing.titleAccent') as string}
                        description={t('pricing.description') as string}
                    />
                </Container>
            </section>

            <section className="pb-32">
                <Container>
                    <div className="max-w-6xl mx-auto space-y-8">
                        <SectionReveal>
                            <div className="grid lg:grid-cols-[1.3fr_0.9fr] gap-8">
                                <div className="glass-card rounded-[2rem] p-8 md:p-10 border border-white/5 relative overflow-hidden">
                                    <div className="absolute inset-0 bg-[radial-gradient(circle_at_10%_0%,rgba(249,115,22,0.08),transparent_55%)] pointer-events-none" />
                                    <div className="relative z-10">
                                        <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-brand-500/10 border border-brand-500/20 text-brand-400 text-[11px] font-bold uppercase tracking-widest mb-6">
                                            <HiOutlineClipboardCheck size={14} />
                                            {t('pricing.story.badge') as string}
                                        </div>
                                        <h2 className="text-3xl md:text-4xl font-black text-white tracking-tight mb-4">
                                            {t('pricing.story.title') as string}
                                        </h2>
                                        <p className="text-sm md:text-base text-gray-400 leading-relaxed max-w-2xl mb-8">
                                            {t('pricing.story.desc') as string}
                                        </p>
                                        <Link
                                            href="/contact"
                                            className="inline-flex items-center gap-2 px-5 py-3 rounded-xl bg-brand-500 text-white font-bold shadow-[0_0_20px_rgba(249,115,22,0.18)] hover:shadow-[0_0_30px_rgba(249,115,22,0.32)] transition-all duration-300"
                                        >
                                            {t('pricing.cta.primary') as string}
                                            <HiOutlineArrowRight size={16} />
                                        </Link>
                                    </div>
                                </div>

                                <div className="rounded-[2rem] p-[1px] bg-gradient-to-b from-white/10 to-transparent">
                                    <div className="h-full rounded-[31px] bg-[#090909] p-8 md:p-10 border border-white/5">
                                        <h3 className="text-xl font-bold text-white mb-3">{t('pricing.fit.title') as string}</h3>
                                        <p className="text-sm text-gray-400 leading-relaxed mb-6">
                                            {t('pricing.fit.desc') as string}
                                        </p>
                                        <ul className="space-y-4">
                                            {qualificationPoints.map((item) => (
                                                <li key={item} className="flex items-start gap-3">
                                                    <HiOutlineCheckCircle size={18} className="text-brand-400 mt-0.5 shrink-0" />
                                                    <span className="text-sm text-gray-300 leading-relaxed">{item}</span>
                                                </li>
                                            ))}
                                        </ul>
                                    </div>
                                </div>
                            </div>
                        </SectionReveal>

                        <SectionReveal delay={0.08}>
                            <div className="rounded-[2rem] p-[1px] bg-gradient-to-b from-white/10 to-transparent">
                                <div className="rounded-[31px] bg-[#090909] p-8 md:p-10 border border-white/5">
                                    <div className="max-w-2xl mb-8">
                                        <h2 className="text-2xl md:text-3xl font-black text-white tracking-tight mb-3">
                                            {t('pricing.deployment.title') as string}
                                        </h2>
                                        <p className="text-sm text-gray-400 leading-relaxed">
                                            {t('pricing.deployment.desc') as string}
                                        </p>
                                    </div>

                                    <div className="grid md:grid-cols-3 gap-6">
                                        {deploymentOptions.map((option, index) => (
                                            <div
                                                key={option.title}
                                                className={`rounded-2xl p-7 border transition-all duration-300 ${
                                                    index === 1
                                                        ? 'border-brand-500/25 bg-brand-500/5 shadow-[0_0_24px_rgba(249,115,22,0.08)]'
                                                        : 'border-white/5 bg-surface-300/50'
                                                }`}
                                            >
                                                <div className="w-11 h-11 rounded-xl bg-white/5 border border-white/10 flex items-center justify-center text-brand-400 mb-5">
                                                    {option.icon}
                                                </div>
                                                <h3 className="text-lg font-bold text-white mb-2">{option.title}</h3>
                                                <p className="text-sm text-gray-400 leading-relaxed">{option.description}</p>
                                            </div>
                                        ))}
                                    </div>
                                </div>
                            </div>
                        </SectionReveal>

                        <SectionReveal delay={0.16}>
                            <div className="rounded-[2rem] p-[1px] bg-gradient-to-b from-white/10 to-transparent">
                                <div className="rounded-[31px] bg-[#090909] p-8 md:p-10 border border-white/5">
                                    <div className="max-w-2xl mb-8">
                                        <h2 className="text-2xl md:text-3xl font-black text-white tracking-tight mb-3">
                                            {t('pricing.process.title') as string}
                                        </h2>
                                    </div>

                                    <div className="grid md:grid-cols-3 gap-6">
                                        {processSteps.map((step) => (
                                            <div key={step.title} className="rounded-2xl border border-white/5 bg-surface-300/50 p-7">
                                                <h3 className="text-base font-bold text-white mb-3">{step.title}</h3>
                                                <p className="text-sm text-gray-400 leading-relaxed">{step.description}</p>
                                            </div>
                                        ))}
                                    </div>
                                </div>
                            </div>
                        </SectionReveal>
                    </div>
                </Container>
            </section>

            <CTASection
                title={t('pricing.title') as string}
                titleAccent={t('pricing.titleAccent') as string}
                description={t('pricing.description') as string}
                primaryCta={{ label: t('pricing.cta.primary') as string, href: '/contact' }}
                secondaryCta={{ label: t('pricing.cta.secondary') as string, href: '/docs' }}
            />
        </>
    );
}
