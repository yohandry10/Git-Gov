'use client';

import React from 'react';
import { Container } from '@/components/layout/Container';
import { SectionHeader } from '@/components/marketing/SectionHeader';
import { SectionReveal } from '@/components/ui/SectionReveal';
import { useTranslation } from '@/lib/i18n';
import {
    HiOutlineShieldCheck,
    HiOutlineDocumentSearch,
    HiOutlineLightningBolt,
    HiOutlineEye,
} from 'react-icons/hi';

export function CapabilitiesSection() {
    const { t } = useTranslation();

    const capabilities = [
        {
            icon: <HiOutlineShieldCheck size={28} />,
            titleKey: 'capabilities.governance.title',
            descKey: 'capabilities.governance.desc',
        },
        {
            icon: <HiOutlineDocumentSearch size={28} />,
            titleKey: 'capabilities.audit.title',
            descKey: 'capabilities.audit.desc',
        },
        {
            icon: <HiOutlineLightningBolt size={28} />,
            titleKey: 'capabilities.ci.title',
            descKey: 'capabilities.ci.desc',
            badge: 'Jenkins',
        },
        {
            icon: <HiOutlineEye size={28} />,
            titleKey: 'capabilities.ticket.title',
            descKey: 'capabilities.ticket.desc',
            badge: 'Jira',
        },
    ];

    return (
        <section className="section-gap bg-surface-100/30 relative overflow-hidden" id="capabilities">
            {/* Ambient Background */}
            <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-full max-w-[1200px] h-[500px] pointer-events-none">
                <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_center,rgba(249,115,22,0.05),transparent_70%)]" />
            </div>

            <Container>
                <SectionHeader
                    badge={t('capabilities.badge') as string}
                    title={t('capabilities.title') as string}
                    titleAccent={t('capabilities.titleAccent') as string}
                    description={t('capabilities.description') as string}
                />

                <div className="mt-20 relative max-w-6xl mx-auto">
                    
                    {/* SVG Connecting Line (Desktop) */}
                    <div className="hidden lg:block absolute top-[120px] left-[10%] w-[80%] h-px border-t-2 border-dashed border-white/[0.1] z-0">
                        <div className="absolute top-0 left-0 h-[2px] w-[20%] bg-gradient-to-r from-transparent via-brand-500 to-transparent -translate-y-[2px] animate-[slide-up_4s_linear_infinite] opacity-50" style={{ animationDirection: 'normal', animationName: 'slideRight' }} />
                    </div>
                    
                    <style dangerouslySetInnerHTML={{__html: `
                        @keyframes slideRight {
                            0% { left: -20%; opacity: 0; }
                            10% { opacity: 1; }
                            90% { opacity: 1; }
                            100% { left: 100%; opacity: 0; }
                        }
                    `}} />

                    <div className="grid lg:grid-cols-4 gap-8 lg:gap-6 relative z-10">
                        {capabilities.map((cap, idx) => (
                            <SectionReveal key={cap.titleKey} delay={idx * 0.15}>
                                <div className="group relative flex flex-col items-center lg:items-start text-center lg:text-left h-full">
                                    
                                    {/* Abstract glowing connector drops */}
                                    <div className="hidden lg:block absolute left-1/2 -translate-x-1/2 top-[120px] w-4 h-4 rounded-full bg-[#141414] border-2 border-white/[0.1] group-hover:border-brand-500 group-hover:shadow-[0_0_15px_rgba(249,115,22,0.5)] transition-all duration-500 z-10" />

                                    {/* Icon Box */}
                                    <div className="w-24 h-24 rounded-2xl bg-gradient-to-tl from-surface-300 to-[#1a1a1a] border border-white/[0.08] shadow-2xl flex items-center justify-center text-brand-400 group-hover:-translate-y-2 group-hover:border-brand-500/50 group-hover:shadow-[0_10px_40px_rgba(249,115,22,0.15)] transition-all duration-500 mb-8 relative">
                                        <div className="absolute inset-0 bg-brand-500/10 opacity-0 group-hover:opacity-100 rounded-2xl transition-opacity duration-500" />
                                        <div className="relative z-10 transform group-hover:scale-110 transition-transform duration-500 delay-100">
                                            {cap.icon}
                                        </div>
                                    </div>

                                    {/* Text Content */}
                                    <div className="relative lg:pt-16">
                                        <div className="flex flex-col lg:flex-row items-center lg:items-start gap-3 mb-3">
                                            <h3 className="text-xl font-bold text-white tracking-tight">
                                                {t(cap.titleKey as any) as string}
                                            </h3>
                                            {cap.badge && (
                                                <span className="px-2 py-0.5 rounded text-[10px] font-bold tracking-widest uppercase bg-brand-500/10 text-brand-400 border border-brand-500/20">
                                                    {cap.badge}
                                                </span>
                                            )}
                                        </div>
                                        <p className="text-gray-400 text-sm leading-relaxed max-w-[280px]">
                                            {t(cap.descKey as any) as string}
                                        </p>
                                    </div>
                                    
                                </div>
                            </SectionReveal>
                        ))}
                    </div>
                </div>
            </Container>
        </section>
    );
}
