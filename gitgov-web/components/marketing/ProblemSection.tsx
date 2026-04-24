'use client';

import React from 'react';
import { Container } from '@/components/layout/Container';
import { SectionReveal } from '@/components/ui/SectionReveal';
import { useTranslation } from '@/lib/i18n';
import { HiOutlineLightningBolt, HiOutlineShieldCheck } from 'react-icons/hi';

export function ProblemSection() {
    const { t } = useTranslation();

    return (
        <section className="section-gap bg-surface-100/30" id="problem">
            <Container>
                <div className="grid lg:grid-cols-2 gap-16 lg:gap-24 items-center">
                    <SectionReveal>
                        <div className="space-y-6">
                            <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full border border-brand-500/20 bg-brand-500/5 text-brand-400 text-xs font-semibold tracking-wide uppercase">
                                <HiOutlineLightningBolt size={14} />
                                {t('problem.badge') as string}
                            </div>
                            
                            <h2 className="text-4xl md:text-5xl font-bold tracking-tight text-white leading-[1.1]">
                                {t('problem.title') as string}{' '}
                                <span className="text-transparent bg-clip-text bg-gradient-to-r from-brand-400 to-accent-400">
                                    {t('problem.titleAccent') as string}
                                </span>
                            </h2>
                            
                            <p className="text-lg text-gray-400 leading-relaxed max-w-xl">
                                {t('problem.description') as string}
                            </p>
                        </div>
                    </SectionReveal>

                    <div className="space-y-6">
                        <SectionReveal delay={0.1}>
                            <div className="p-8 rounded-2xl bg-white/[0.02] border border-white/[0.05] relative overflow-hidden group">
                                <div className="absolute top-0 right-0 p-4 opacity-10 blur-sm group-hover:blur-none transition-all duration-500 text-brand-400">
                                    <HiOutlineLightningBolt size={100} />
                                </div>
                                <h3 className="text-xl font-bold text-white mb-3 flex items-center gap-3">
                                    <span className="w-2 h-2 rounded-full bg-brand-400"></span>
                                    {t('problem.challenge.title') as string}
                                </h3>
                                <p className="text-gray-400 leading-relaxed relative z-10">
                                    {t('problem.challenge.desc') as string}
                                </p>
                            </div>
                        </SectionReveal>

                        <SectionReveal delay={0.2}>
                            <div className="p-8 rounded-2xl bg-gradient-to-br from-brand-500/10 to-transparent border border-brand-500/20 relative overflow-hidden group">
                                <div className="absolute bottom-0 right-0 p-4 opacity-10 blur-sm group-hover:blur-none transition-all duration-500 text-brand-500">
                                    <HiOutlineShieldCheck size={100} />
                                </div>
                                <h3 className="text-xl font-bold text-white mb-3 flex items-center gap-3">
                                    <span className="w-2 h-2 rounded-full bg-brand-500 shadow-[0_0_8px_#f97316]"></span>
                                    {t('problem.solution.title') as string}
                                </h3>
                                <p className="text-gray-300 leading-relaxed relative z-10">
                                    {t('problem.solution.desc') as string}
                                </p>
                            </div>
                        </SectionReveal>
                    </div>
                </div>
            </Container>
        </section>
    );
}
