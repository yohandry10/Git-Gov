'use client';

import React from 'react';
import { Container } from '@/components/layout';
import { CTASection } from '@/components/marketing';
import { SectionReveal } from '@/components/ui';
import { useTranslation } from '@/lib/i18n';
import {
    HiOutlineShieldCheck,
    HiOutlineDocumentSearch,
    HiOutlineLightningBolt,
    HiOutlineEye,
    HiOutlineLockClosed,
    HiOutlineClipboardCheck,
    HiOutlinePuzzle,
    HiOutlineTrendingUp,
    HiOutlineWifi,
    HiOutlineDesktopComputer,
    HiOutlineCheckCircle,
    HiOutlineArrowRight,
    HiOutlineCode,
} from 'react-icons/hi';

/* ─────────────────────────────────────────────────────────────────────────
   Section label divider
───────────────────────────────────────────────────────────────────────── */
function SectionLabel({
    icon, label, iconCls, textCls, lineCls,
}: {
    icon: React.ReactNode; label: string;
    iconCls: string; textCls: string; lineCls: string;
}) {
    return (
        <SectionReveal>
            <div className="flex items-center gap-4 mb-16">
                <div className={`w-8 h-8 rounded-lg border flex items-center justify-center shrink-0 ${iconCls}`}>
                    {icon}
                </div>
                <span className={`text-[11px] font-black tracking-[0.25em] uppercase ${textCls}`}>{label}</span>
                <div className={`flex-1 h-px bg-gradient-to-r ${lineCls} to-transparent`} />
            </div>
        </SectionReveal>
    );
}

/* ─────────────────────────────────────────────────────────────────────────
   Main
───────────────────────────────────────────────────────────────────────── */
export function FeaturesClient() {
    const { t } = useTranslation();
    const navigation = [
        { label: t('features.nav.capture') as string, href: '#capture' },
        { label: t('features.nav.governance') as string, href: '#governance' },
        { label: t('features.nav.correlation') as string, href: '#correlation' },
        { label: t('features.nav.reporting') as string, href: '#reporting' },
    ];
    const governanceModes = [
        t('features.policy.mode.off') as string,
        t('features.policy.mode.warn') as string,
        t('features.policy.mode.block') as string,
    ];
    const reportingSurface = [
        {
            title: t('features.dashboard.surface.pipeline') as string,
            description: t('features.dashboard.surface.pipelineDesc') as string,
        },
        {
            title: t('features.dashboard.surface.coverage') as string,
            description: t('features.dashboard.surface.coverageDesc') as string,
        },
        {
            title: t('features.dashboard.surface.risk') as string,
            description: t('features.dashboard.surface.riskDesc') as string,
        },
        {
            title: t('features.dashboard.surface.export') as string,
            description: t('features.dashboard.surface.exportDesc') as string,
        },
    ];
    const auditItems = [
        t('features.risk.audit.item1') as string,
        t('features.risk.audit.item2') as string,
        t('features.risk.audit.item3') as string,
    ];

    return (
        <>
            {/* ══════════════════════════════════════════════════════
                HERO
            ══════════════════════════════════════════════════════ */}
            <section className="pt-32 md:pt-44 pb-0 relative overflow-hidden">
                <div
                    className="absolute inset-0 pointer-events-none"
                    style={{
                        opacity: 0.025,
                        backgroundImage: `linear-gradient(rgba(249,115,22,0.6) 1px, transparent 1px),
                                          linear-gradient(90deg, rgba(249,115,22,0.6) 1px, transparent 1px)`,
                        backgroundSize: '72px 72px',
                    }}
                />
                <div
                    className="absolute top-0 left-1/2 -translate-x-1/2 w-[1400px] h-[700px] pointer-events-none"
                    style={{
                        background:
                            'radial-gradient(ellipse at 50% 0%, rgba(249,115,22,0.1) 0%, rgba(249,115,22,0.04) 35%, transparent 70%)',
                    }}
                />

                <Container>
                    <div className="text-center max-w-5xl mx-auto">
                        <SectionReveal>
                            <div className="inline-flex items-center gap-2.5 px-4 py-2 rounded-full bg-brand-500/10 border border-brand-500/25 mb-10">
                                <div className="w-1.5 h-1.5 rounded-full bg-brand-400 animate-pulse" />
                                <span className="text-brand-400 text-[11px] font-bold tracking-widest uppercase">
                                    {t('features.badge') as string}
                                </span>
                            </div>

                            <h1 className="font-semibold tracking-tight leading-[1] mb-8">
                                <span className="block text-white text-4xl md:text-5xl lg:text-6xl">
                                    {t('features.title') as string}
                                </span>
                                <span className="block gradient-text text-4xl md:text-5xl lg:text-6xl">
                                    {t('features.titleAccent') as string}
                                </span>
                            </h1>

                            <p className="text-lg md:text-xl text-gray-400 leading-relaxed max-w-2xl mx-auto mb-14">
                                {t('features.description') as string}
                            </p>
                        </SectionReveal>

                        {/* Anchor navigation */}
                        <SectionReveal delay={0.1}>
                            <div className="flex flex-wrap justify-center gap-3 mb-20">
                                {navigation.map((p) => (
                                    <a
                                        key={p.label}
                                        href={p.href}
                                        className="flex items-center gap-2 px-5 py-2.5 rounded-full text-sm font-semibold border transition-all duration-200 text-brand-400 border-brand-500/30 bg-brand-500/10 hover:bg-brand-500/20"
                                    >
                                        {p.label}
                                        <HiOutlineArrowRight size={13} />
                                    </a>
                                ))}
                            </div>
                        </SectionReveal>
                    </div>
                </Container>
            </section>

            {/* ══════════════════════════════════════════════════════
                1. WORKSTATION CAPTURE
            ══════════════════════════════════════════════════════ */}
            <section id="capture" className="py-28 relative">
                <div
                    className="absolute top-1/2 left-[-200px] -translate-y-1/2 w-[600px] h-[600px] rounded-full pointer-events-none"
                    style={{ background: 'radial-gradient(ellipse, rgba(249,115,22,0.07) 0%, transparent 65%)' }}
                />

                <Container>
                    <SectionLabel
                        icon={<HiOutlineDesktopComputer size={16} className="text-brand-400" />}
                        label={t('features.core.badge') as string}
                        iconCls="text-brand-400 bg-brand-500/10 border-brand-500/25"
                        textCls="text-brand-400"
                        lineCls="from-brand-500/30"
                    />

                    <div className="mb-12 max-w-3xl">
                         <h2 className="text-4xl md:text-5xl font-black text-white mb-4 leading-tight">
                             {t('features.core.title') as string}{' '}
                             <span className="gradient-text">{t('features.core.titleAccent') as string}</span>
                         </h2>
                         <p className="text-gray-400 text-lg leading-relaxed">
                             {t('features.core.description') as string}
                         </p>
                    </div>

                    <div className="grid lg:grid-cols-5 gap-6">
                        {/* Dominant Terminal Canvas */}
                        <SectionReveal className="lg:col-span-3">
                            <div className="glass-card rounded-3xl p-10 h-full border-white/5 bg-gradient-to-br from-white/5 to-transparent relative overflow-hidden group hover:border-brand-500/30 transition-colors duration-500">
                                <div className="absolute top-0 right-0 w-64 h-64 bg-brand-500/10 rounded-full blur-[80px] -translate-y-1/2 translate-x-1/2 group-hover:bg-brand-500/20 transition-colors duration-500" />
                                
                                <div className="mb-10 relative z-10 flex items-center justify-between">
                                    <div>
                                        <div className="w-14 h-14 rounded-2xl bg-brand-500/15 border border-brand-500/30 flex items-center justify-center text-brand-400 mb-6 shadow-[0_0_20px_rgba(249,115,22,0.15)] group-hover:scale-110 transition-transform duration-500">
                                            <HiOutlineCode size={28} />
                                        </div>
                                        <h3 className="text-2xl font-bold text-white mb-2">{t('features.commit.title') as string}</h3>
                                        <p className="text-gray-400 text-sm max-w-sm">
                                            {t('features.commit.desc') as string}
                                        </p>
                                    </div>
                                </div>
                                
                                {/* Visual Representation Canvas */}
                                <div className="w-full bg-[#0d0d0d] rounded-xl border border-white/10 p-6 font-mono text-sm relative z-10 shadow-2xl overflow-hidden group-hover:border-brand-500/20 transition-colors duration-500">
                                     <div className="absolute top-0 left-0 w-1 h-full bg-brand-500/50" />
                                    <div className="flex gap-2 mb-6 ml-2">
                                        <div className="w-3 h-3 rounded-full bg-white/10 border border-white/20" />
                                        <div className="w-3 h-3 rounded-full bg-white/10 border border-white/20" />
                                        <div className="w-3 h-3 rounded-full bg-white/10 border border-white/20" />
                                    </div>
                                    <div className="text-gray-500 ml-2">$ git commit -m &quot;feat: login system&quot;</div>
                                    <div className="text-brand-400 mt-2 font-semibold ml-2">› Intercepting git hook...</div>
                                    <div className="text-gray-400 mt-1 ml-2">› Capturing workstation state...</div>
                                    <div className="text-gray-300 mt-2 font-mono bg-white/5 inline-block px-2 py-1 rounded ml-2 border border-white/10">✓ Evidence logged locally [a3f8c01]</div>
                                </div>
                            </div>
                        </SectionReveal>

                        {/* Offline Sync Canvas */}
                        <SectionReveal delay={0.1} className="lg:col-span-2">
                            <div className="glass-card rounded-3xl p-10 h-full border-white/5 bg-gradient-to-br from-white/5 to-transparent relative overflow-hidden group hover:border-brand-500/30 transition-colors duration-500 flex flex-col">
                                 <div className="absolute top-0 right-0 w-64 h-64 bg-brand-500/10 rounded-full blur-[80px] -translate-y-1/2 translate-x-1/2 group-hover:bg-brand-500/20 transition-colors duration-500" />
                                
                                <div className="mb-10 relative z-10">
                                    <div className="w-14 h-14 rounded-2xl bg-brand-500/15 border border-brand-500/30 flex items-center justify-center text-brand-400 mb-6 shadow-[0_0_20px_rgba(249,115,22,0.15)] group-hover:scale-110 transition-transform duration-500">
                                        <HiOutlineWifi size={28} />
                                    </div>
                                    <h3 className="text-xl md:text-2xl font-bold text-white mb-2">{t('features.offline.title') as string}</h3>
                                    <p className="text-gray-400 text-sm">
                                        {t('features.offline.desc') as string}
                                    </p>
                                </div>

                                {/* Visual Representation Canvas */}
                                <div className="w-full h-40 mt-auto rounded-xl border border-white/10 flex items-center justify-center relative z-10 overflow-hidden bg-black/40 shadow-inner group-hover:border-brand-500/20 transition-colors duration-500">
                                     <div className="flex flex-col sm:flex-row items-center justify-between w-full px-6 opacity-90 gap-4">
                                         {/* Dev Machine */}
                                         <div className="w-14 h-14 shrink-0 rounded-xl border border-white/20 bg-white/5 flex items-center justify-center relative shadow-lg">
                                             <HiOutlineDesktopComputer size={28} className="text-gray-300" />
                                         </div>
                                         
                                         {/* Broken Network Link */}
                                         <div className="hidden sm:block flex-1 h-px border-t-2 border-dashed border-gray-600 relative mx-4 group-hover:border-brand-500/30 transition-colors duration-500">
                                            <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 bg-black/80 text-gray-400 text-[9px] font-bold px-2 py-0.5 rounded border border-white/10 shadow-inner">OFFLINE</div>
                                         </div>
                                         
                                         {/* Local Queue Vault */}
                                         <div className="w-32 h-16 shrink-0 rounded-xl border border-brand-500/40 bg-brand-500/10 flex flex-col justify-center px-4 relative overflow-hidden shadow-[0_0_20px_rgba(249,115,22,0.1)] group-hover:border-brand-500/60 transition-colors duration-500">
                                             <div className="absolute top-0 right-0 w-16 h-16 bg-brand-500/20 blur-xl group-hover:bg-brand-500/30 transition-colors duration-500" />
                                             <div className="text-[9px] text-brand-400 font-bold mb-1.5 tracking-widest leading-none shrink-0 relative z-10">LOCAL QUEUE</div>
                                             <div className="flex gap-1 grayscale-0 relative z-10">
                                                 <div className="w-1/3 h-1 bg-brand-400 rounded-sm shadow-[0_0_8px_rgba(249,115,22,0.6)]" />
                                                 <div className="w-1/3 h-1 bg-brand-500/60 rounded-sm" />
                                                 <div className="w-1/3 h-1 bg-brand-500/30 rounded-sm" />
                                             </div>
                                         </div>
                                     </div>
                                </div>
                            </div>
                        </SectionReveal>
                    </div>
                </Container>
            </section>

             {/* ══════════════════════════════════════════════════════
                2. GOVERNANCE ENGINE
            ══════════════════════════════════════════════════════ */}
            <section id="governance" className="py-28 relative bg-surface-100/30">
                <Container>
                    <SectionLabel
                        icon={<HiOutlineShieldCheck size={16} className="text-brand-400" />}
                        label={t('features.policy.badge') as string}
                        iconCls="text-brand-400 bg-brand-500/10 border-brand-500/25"
                        textCls="text-brand-400"
                        lineCls="from-brand-500/30"
                    />

                    <div className="mb-12">
                         <h2 className="text-3xl md:text-4xl font-black text-white mb-3">
                             {t('features.policy.title') as string}{' '}
                             <span className="text-transparent bg-clip-text bg-gradient-to-r from-brand-400 to-brand-400">{t('features.policy.titleAccent') as string}</span>
                         </h2>
                         <p className="text-gray-400 max-w-xl text-sm">
                             {t('features.policy.description') as string}
                         </p>
                    </div>

                    <div className="grid lg:grid-cols-3 gap-6">
                        <SectionReveal className="lg:col-span-2">
                            <div className="glass-card rounded-2xl p-8 hover:-translate-y-1 transition-transform duration-300 h-full border-white/5">
                                <div className="w-12 h-12 rounded-xl bg-brand-500/15 border border-brand-500/30 flex items-center justify-center text-brand-400 mb-6">
                                    <HiOutlineClipboardCheck size={24} />
                                </div>
                                <h3 className="text-xl font-bold text-white mb-3">{t('features.policy.check.title') as string}</h3>
                                <p className="text-gray-400 text-sm leading-relaxed">
                                    {t('features.policy.check.desc') as string}
                                </p>
                            </div>
                        </SectionReveal>

                        <SectionReveal delay={0.1}>
                             <div className="glass-card rounded-2xl p-8 hover:-translate-y-1 transition-transform duration-300 h-full border-white/5 flex flex-col justify-center">
                                 <div className="space-y-4">
                                    {governanceModes.map((mode, i) => (
                                        <div key={i} className="flex items-center gap-3">
                                            <HiOutlineCheckCircle className="text-brand-400" size={18} />
                                            <span className="text-sm font-semibold text-gray-200">{mode}</span>
                                        </div>
                                    ))}
                                 </div>
                             </div>
                        </SectionReveal>
                    </div>
                </Container>
            </section>

             {/* ══════════════════════════════════════════════════════
                3. INTEGRATIONS & CORRELATION
            ══════════════════════════════════════════════════════ */}
            <section id="correlation" className="py-28 relative">
                 <div
                    className="absolute top-1/2 right-[-200px] -translate-y-1/2 w-[600px] h-[600px] rounded-full pointer-events-none"
                    style={{ background: 'radial-gradient(ellipse, rgba(59,130,246,0.07) 0%, transparent 65%)' }}
                />
                <Container>
                    <SectionLabel
                        icon={<HiOutlinePuzzle size={16} className="text-brand-400" />}
                        label={t('features.integrations.badge') as string}
                        iconCls="text-brand-400 bg-brand-500/10 border-brand-500/25"
                        textCls="text-brand-400"
                        lineCls="from-brand-500/30"
                    />

                    <div className="mb-12">
                         <h2 className="text-3xl md:text-4xl font-black text-white mb-3">
                             {t('features.integrations.title') as string}{' '}
                             <span className="text-transparent bg-clip-text bg-gradient-to-r from-brand-400 to-brand-400">{t('features.integrations.titleAccent') as string}</span>
                         </h2>
                         <p className="text-gray-400 max-w-xl text-sm">
                             {t('features.integrations.description') as string}
                         </p>
                    </div>

                    <SectionReveal>
                        <div className="w-full glass-card rounded-3xl p-8 lg:p-12 h-full border-white/5 bg-gradient-to-r from-transparent via-brand-500/5 to-transparent relative overflow-hidden group">
                            
                            {/* Visual Pipeline Canvas */}
                            <div className="w-full h-48 md:h-64 rounded-2xl border border-white/5 bg-black/40 mb-12 flex items-center justify-between px-6 md:px-16 relative overflow-hidden shadow-inner hidden md:flex">
                                <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_center,rgba(249,115,22,0.1)_0%,transparent_70%)]" />
                                
                                {/* Node 1: Git */}
                                <div className="flex flex-col items-center relative z-10 w-24">
                                    <div className="w-16 h-16 rounded-2xl bg-white/5 border border-white/10 flex items-center justify-center mb-4 shadow-[0_0_15px_rgba(255,255,255,0.05)] text-gray-400 group-hover:text-white transition-colors duration-500">
                                        <HiOutlineCode size={32} />
                                    </div>
                                    <div className="text-[10px] font-bold text-gray-500 uppercase tracking-widest text-center">Git Commit</div>
                                </div>

                                {/* Link 1 */}
                                <div className="flex-1 h-px border-t border-brand-500/30 relative overflow-hidden mx-4">
                                     <div className="absolute top-1/2 left-0 w-12 h-[2px] bg-brand-500 shadow-[0_0_10px_#f97316] -translate-y-1/2" style={{ animation: 'pipeline-flow 2s linear infinite' }} />
                                </div>

                                {/* Node 2: CI (Jenkins/Actions) */}
                                <div className="flex flex-col items-center relative z-10 w-24">
                                    <div className="w-16 h-16 rounded-2xl bg-brand-500/10 border border-brand-500/30 flex items-center justify-center mb-4 shadow-[0_0_20px_rgba(249,115,22,0.15)] text-brand-400 group-hover:scale-110 transition-transform duration-500">
                                        <HiOutlineLightningBolt size={32} />
                                    </div>
                                    <div className="text-[10px] font-bold text-brand-400 uppercase tracking-widest text-center">CI Pipeline</div>
                                </div>

                                {/* Link 2 */}
                                <div className="flex-1 h-px border-t border-brand-500/30 relative overflow-hidden mx-4">
                                     <div className="absolute top-1/2 left-0 w-12 h-[2px] bg-brand-500 shadow-[0_0_10px_#f97316] -translate-y-1/2" style={{ animation: 'pipeline-flow 2s linear infinite', animationDelay: '1s' }} />
                                </div>

                                {/* Node 3: Ticket (Jira) */}
                                <div className="flex flex-col items-center relative z-10 w-24">
                                    <div className="w-16 h-16 rounded-2xl bg-white/5 border border-white/10 flex items-center justify-center mb-4 shadow-[0_0_15px_rgba(255,255,255,0.05)] text-gray-400 group-hover:text-white transition-colors duration-500">
                                        <HiOutlineDocumentSearch size={32} />
                                    </div>
                                    <div className="text-[10px] font-bold text-gray-500 uppercase tracking-widest text-center">Jira Ticket</div>
                                </div>
                            </div>
                            
                            {/* Descriptive Text Split */}
                            <div className="flex flex-col md:flex-row gap-10 lg:gap-20">
                                <div className="flex-1">
                                    <h3 className="text-xl md:text-2xl font-bold text-white mb-3 flex items-center gap-3">
                                        <HiOutlineLightningBolt className="text-brand-400" /> {t('features.jenkins.title') as string}
                                    </h3>
                                    <p className="text-gray-400 text-sm leading-relaxed">
                                        {t('features.jenkins.desc') as string}
                                    </p>
                                </div>
                                <div className="hidden md:block w-px bg-white/10" />
                                <div className="flex-1">
                                    <h3 className="text-xl md:text-2xl font-bold text-white mb-3 flex items-center gap-3">
                                        <HiOutlineDocumentSearch className="text-gray-300" /> {t('features.jira.title') as string}
                                    </h3>
                                    <p className="text-gray-400 text-sm leading-relaxed">
                                        {t('features.jira.desc') as string}
                                    </p>
                                </div>
                            </div>

                            <div className="mt-10 pt-6 border-t border-white/8">
                                <h3 className="text-lg md:text-xl font-bold text-white mb-2 flex items-center gap-3">
                                    <HiOutlinePuzzle className="text-brand-400" /> {t('features.github.title') as string}
                                </h3>
                                <p className="text-gray-400 text-sm leading-relaxed max-w-3xl">
                                    {t('features.github.desc') as string}
                                </p>
                            </div>
                        </div>
                    </SectionReveal>
                </Container>
            </section>

             {/* ══════════════════════════════════════════════════════
                4. RISK, READINESS & REPORTING
            ══════════════════════════════════════════════════════ */}
             <section id="reporting" className="py-28 relative bg-surface-100/30">
                <Container>
                    <SectionLabel
                        icon={<HiOutlineTrendingUp size={16} className="text-brand-400" />}
                        label={t('features.risk.badge') as string}
                        iconCls="text-brand-400 bg-brand-500/10 border-brand-500/25"
                        textCls="text-brand-400"
                        lineCls="from-brand-500/30"
                    />

                    <div className="mb-12">
                         <h2 className="text-3xl md:text-4xl font-black text-white mb-3">
                             {t('features.risk.title') as string}{' '}
                             <span className="text-transparent bg-clip-text bg-gradient-to-r from-brand-400 to-brand-400">{t('features.risk.titleAccent') as string}</span>
                         </h2>
                         <p className="text-gray-400 max-w-xl text-sm">
                             {t('features.risk.description') as string}
                         </p>
                    </div>

                    <div className="grid md:grid-cols-3 gap-6">
                        {/* Status Matrix */}
                        <SectionReveal className="md:col-span-2">
                            <div className="glass-card rounded-3xl p-8 h-full border-white/5 bg-gradient-to-br from-white/5 to-transparent relative overflow-hidden group">
                                <div className="absolute top-0 right-0 w-64 h-64 bg-brand-500/10 rounded-full blur-[80px] -translate-y-1/2 translate-x-1/2" />
                                
                                <div className="flex flex-col md:flex-row gap-8 relative z-10">
                                    <div className="flex-1">
                                        <div className="w-12 h-12 rounded-xl bg-brand-500/15 border border-brand-500/30 flex items-center justify-center text-brand-400 mb-6 shadow-lg">
                                            <HiOutlineEye size={24} />
                                        </div>
                                        <h3 className="text-xl font-bold text-white mb-3">{t('features.centralized.title') as string}</h3>
                                        <p className="text-gray-400 text-sm leading-relaxed max-w-sm">
                                            {t('features.centralized.desc') as string}
                                        </p>
                                    </div>
                                    
                                    {/* Reporting Surface */}
                                    <div className="flex-1 min-w-[240px] bg-black/40 border border-white/10 rounded-2xl p-5 shadow-inner">
                                        <div className="text-[10px] uppercase tracking-widest text-gray-500 mb-4 font-bold">
                                            {t('features.dashboard.surface.title') as string}
                                        </div>
                                        <div className="grid gap-3">
                                            {reportingSurface.map((item) => (
                                                <div key={item.title} className="rounded-xl border border-white/8 bg-white/[0.03] p-3">
                                                    <div className="flex items-center gap-2 mb-1.5">
                                                        <span className="h-1.5 w-1.5 rounded-full bg-brand-400" />
                                                        <span className="text-[11px] font-semibold tracking-wide text-white">
                                                            {item.title}
                                                        </span>
                                                    </div>
                                                    <p className="text-[10px] leading-relaxed text-gray-400">
                                                        {item.description}
                                                    </p>
                                                </div>
                                            ))}
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </SectionReveal>

                        {/* Audit Trail Vertical Card */}
                        <SectionReveal delay={0.1}>
                             <div className="glass-card rounded-3xl p-8 h-full border-white/5 bg-gradient-to-b from-brand-500/10 to-transparent flex flex-col relative overflow-hidden group">
                                 <div className="w-12 h-12 rounded-xl bg-brand-500/20 border border-brand-500/40 flex items-center justify-center text-brand-400 mb-6 shadow-[0_0_20px_rgba(249,115,22,0.2)]">
                                    <HiOutlineLockClosed size={24} />
                                </div>
                                <h3 className="text-xl font-bold text-white mb-3">{t('features.risk.audit.title') as string}</h3>
                                <p className="text-gray-400 text-sm leading-relaxed mb-6">
                                    {t('features.risk.audit.desc') as string}
                                </p>
                                
                                <div className="mt-auto space-y-2">
                                     {auditItems.map((item) => (
                                         <div key={item} className="flex items-center gap-3 text-[10px] font-mono border-b border-white/[0.03] pb-2 last:border-0 last:pb-0">
                                             <span className="h-1.5 w-1.5 rounded-full bg-brand-400 shrink-0" />
                                             <span className="text-gray-400">{item}</span>
                                         </div>
                                     ))}
                                </div>
                             </div>
                        </SectionReveal>
                    </div>
                </Container>
            </section>

            {/* ══════════════════════════════════════════════════════
                CTA
            ══════════════════════════════════════════════════════ */}
            <CTASection
                title={t('features.cta.title') as string}
                titleAccent={t('features.cta.titleAccent') as string}
                description={t('features.cta.desc') as string}
                primaryCta={{ label: t('features.cta.primary') as string, href: '/download' }}
                secondaryCta={{ label: t('features.cta.secondary') as string, href: '/docs' }}
            />
        </>
    );
}
