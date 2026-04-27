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
    HiOutlineTerminal,
    HiOutlineDatabase,
} from 'react-icons/hi';

/* ─────────────────────────────────────────────────────────────────────────
   Shared Components
───────────────────────────────────────────────────────────────────────── */
function SectionBadge({ icon, label }: { icon: React.ReactNode; label: string }) {
    return (
        <div className="inline-flex items-center gap-2.5 px-3 py-1.5 rounded-full bg-white/[0.03] border border-white/5 mb-8">
            <span className="text-brand-400">{icon}</span>
            <span className="text-gray-300 text-[11px] font-bold tracking-widest uppercase">{label}</span>
        </div>
    );
}

function GridPattern() {
    return (
        <div
            className="absolute inset-0 pointer-events-none opacity-[0.015]"
            style={{
                backgroundImage: `linear-gradient(to right, #fff 1px, transparent 1px), linear-gradient(to bottom, #fff 1px, transparent 1px)`,
                backgroundSize: '4rem 4rem'
            }}
        />
    );
}

/* ─────────────────────────────────────────────────────────────────────────
   Hero Section
───────────────────────────────────────────────────────────────────────── */
function FeaturesHero() {
    const { t } = useTranslation();

    return (
        <section className="pt-32 pb-24 md:pt-40 md:pb-32 relative overflow-hidden bg-[#030303]">
            <GridPattern />

            {/* Ultra subtle top glow */}
            <div className="absolute top-0 left-1/2 -translate-x-1/2 w-[800px] h-[400px] bg-brand-500/10 blur-[120px] rounded-full pointer-events-none" />

            <Container className="relative z-10">
                <SectionReveal>
                    <div className="text-center max-w-4xl mx-auto flex flex-col items-center">
                        <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-brand-500/10 border border-brand-500/20 mb-8">
                            <div className="w-1.5 h-1.5 rounded-full bg-brand-400 animate-pulse" />
                            <span className="text-brand-400 text-xs font-semibold tracking-wide">
                                {t('features.badge') as string}
                            </span>
                        </div>

                        <h1 className="text-5xl md:text-7xl font-bold tracking-tight text-white mb-8 leading-[1.1]">
                            {t('features.title') as string}{' '}
                            <span className="text-transparent bg-clip-text bg-gradient-to-b from-brand-400 to-brand-600">
                                {t('features.titleAccent') as string}
                            </span>
                        </h1>

                        <p className="text-lg md:text-xl text-gray-400 max-w-2xl leading-relaxed mb-12">
                            {t('features.description') as string}
                        </p>
                    </div>
                </SectionReveal>

                {/* Minimalist Visual Representation of GitGov core loop */}
                <SectionReveal delay={0.1}>
                    <div className="relative max-w-5xl mx-auto mt-8">
                        <div className="absolute inset-0 bg-gradient-to-b from-transparent via-brand-500/5 to-transparent blur-3xl" />
                        <div className="relative h-[200px] md:h-[300px] rounded-3xl border border-white/5 bg-white/[0.01] overflow-hidden flex items-center justify-center backdrop-blur-sm">

                            {/* Abstract Node Flow */}
                            <div className="flex items-center justify-between w-full max-w-3xl px-8">
                                <div className="flex flex-col items-center gap-4">
                                    <div className="w-16 h-16 rounded-full bg-white/[0.03] border border-white/10 flex items-center justify-center text-gray-400 shadow-[0_0_30px_rgba(255,255,255,0.02)]">
                                        <HiOutlineTerminal size={24} />
                                    </div>
                                    <span className="text-xs font-mono text-gray-500 uppercase tracking-widest">Local</span>
                                </div>

                                <div className="flex-1 h-px bg-gradient-to-r from-transparent via-brand-500/50 to-transparent relative mx-4">
                                    <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 px-4 py-1 bg-[#050505] border border-white/10 rounded-full text-xs font-semibold text-brand-400 shadow-[0_0_20px_rgba(249,115,22,0.1)]">
                                        Governance
                                    </div>
                                </div>

                                <div className="flex flex-col items-center gap-4 relative z-10">
                                    <div className="w-20 h-20 rounded-full bg-brand-500/10 border border-brand-500/20 flex items-center justify-center text-brand-400 shadow-[0_0_40px_rgba(249,115,22,0.15)] relative">
                                        <div className="absolute inset-0 rounded-full border border-brand-400/30 animate-ping opacity-20" />
                                        <HiOutlineShieldCheck size={32} />
                                    </div>
                                    <span className="text-xs font-mono text-brand-400 uppercase tracking-widest font-bold">GitGov</span>
                                </div>

                                <div className="flex-1 h-px bg-gradient-to-r from-brand-500/50 via-white/20 to-transparent relative mx-4" />

                                <div className="flex flex-col items-center gap-4">
                                    <div className="w-16 h-16 rounded-full bg-white/[0.03] border border-white/10 flex items-center justify-center text-gray-400 shadow-[0_0_30px_rgba(255,255,255,0.02)]">
                                        <HiOutlineDatabase size={24} />
                                    </div>
                                    <span className="text-xs font-mono text-gray-500 uppercase tracking-widest">Remote</span>
                                </div>
                            </div>

                        </div>
                    </div>
                </SectionReveal>
            </Container>
        </section>
    );
}

/* ─────────────────────────────────────────────────────────────────────────
   Pillar 1: Workstation Capture (Linear/Vercel Enterprise Style)
───────────────────────────────────────────────────────────────────────── */
function CaptureSection() {
    const { t } = useTranslation();

    return (
        <section id="capture" className="py-32 relative bg-[#000] overflow-hidden">
            {/* Absolute minimalist background grid (Vercel style) */}
            <div className="absolute inset-0 bg-[linear-gradient(to_right,#80808012_1px,transparent_1px),linear-gradient(to_bottom,#80808012_1px,transparent_1px)] bg-[size:24px_24px] [mask-image:radial-gradient(ellipse_60%_50%_at_50%_0%,#000_70%,transparent_100%)]" />

            <Container className="relative z-10">
                {/* Hero Header */}
                <SectionReveal>
                    <div className="flex flex-col items-center text-center max-w-3xl mx-auto">
                        <div className="inline-flex items-center gap-2 px-4 py-1.5 rounded-full bg-white/[0.03] border border-white/10 mb-8">
                            <HiOutlineDesktopComputer className="text-brand-500" size={16} />
                            <span className="text-sm font-medium text-gray-300">{t('features.core.badge') as string}</span>
                        </div>
                        <h2 className="text-5xl md:text-6xl font-bold text-white mb-6 tracking-tighter leading-[1.1]">
                            {t('features.core.title') as string}
                        </h2>
                        <p className="text-lg md:text-xl text-gray-400 leading-relaxed max-w-2xl">
                            {t('features.core.description') as string}
                        </p>
                    </div>
                </SectionReveal>

                {/* The "Vercel" Window Mockup */}
                <SectionReveal delay={0.2}>
                    <div className="mt-20 relative max-w-5xl mx-auto">
                        {/* Core ambient glow */}
                        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[80%] h-[80%] bg-brand-500/20 blur-[120px] rounded-[100%] pointer-events-none" />

                        {/* Outer Glass Frame */}
                        <div className="relative p-2 md:p-3 rounded-[32px] bg-white/[0.02] border border-white/5 backdrop-blur-md shadow-2xl">
                            {/* Inner Dark Window */}
                            <div className="rounded-[24px] bg-[#050505] border border-white/10 overflow-hidden flex flex-col shadow-[inset_0_1px_0_0_rgba(255,255,255,0.05)]">

                                {/* Window Header */}
                                <div className="h-14 flex items-center justify-between px-6 border-b border-white/5 bg-[#0a0a0a]">
                                    <div className="flex gap-2">
                                        <div className="w-3 h-3 rounded-full bg-[#333]" />
                                        <div className="w-3 h-3 rounded-full bg-[#333]" />
                                        <div className="w-3 h-3 rounded-full bg-[#333]" />
                                    </div>
                                    <div className="flex items-center gap-2 text-xs font-mono text-gray-500 bg-[#000] px-3 py-1.5 rounded-md border border-white/5 shadow-inner">
                                        <HiOutlineCode size={14} /> ~/projects/gitgov-desktop
                                    </div>
                                    <div className="w-14" /> {/* Spacer */}
                                </div>

                                {/* Window Body */}
                                <div className="p-8 md:p-16 font-mono text-sm md:text-base relative bg-[#020202]">
                                    <div className="flex items-center gap-4 text-gray-300">
                                        <span className="text-brand-500">➜</span>
                                        <span>git commit -m &quot;feat: implement local queue backoff&quot;</span>
                                    </div>

                                    <div className="mt-8 pl-5 relative">
                                        {/* Connecting Line */}
                                        <div className="absolute left-0 top-2 bottom-6 w-px bg-gradient-to-b from-white/10 via-white/10 to-brand-500/50" />

                                        <div className="space-y-8">
                                            <div className="flex items-center gap-4 text-gray-500 relative">
                                                <div className="absolute -left-[23px] w-1.5 h-1.5 rounded-full bg-white/20" />
                                                Intercepting git hook...
                                            </div>
                                            <div className="flex items-center gap-4 text-gray-500 relative">
                                                <div className="absolute -left-[23px] w-1.5 h-1.5 rounded-full bg-white/20" />
                                                Capturing workstation context (author, branch, timestamp)...
                                            </div>

                                            <div className="relative mt-8">
                                                <div className="absolute -left-[27px] top-1/2 -translate-y-1/2 w-2.5 h-2.5 rounded-full bg-brand-500 shadow-[0_0_12px_rgba(249,115,22,0.8)]" />
                                                <div className="p-5 md:p-6 rounded-2xl bg-[#0a0a0a] border border-white/10 flex flex-col sm:flex-row sm:items-center justify-between gap-6 shadow-xl">
                                                    <div className="flex items-center gap-5">
                                                        <div className="w-12 h-12 rounded-full bg-brand-500/10 border border-brand-500/20 flex items-center justify-center shrink-0">
                                                            <span className="text-brand-500 font-bold text-lg">✓</span>
                                                        </div>
                                                        <div>
                                                            <div className="text-white font-sans font-medium text-base">Evidence logged locally</div>
                                                            <div className="text-xs text-gray-500 font-sans mt-1">Hash: a3f8c01b9e2 • Queued for sync</div>
                                                        </div>
                                                    </div>
                                                    <div className="text-xs font-sans text-brand-500 uppercase tracking-widest bg-brand-500/10 px-4 py-2 rounded-full border border-brand-500/20 font-semibold text-center">
                                                        Secured
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </SectionReveal>

                {/* Features Grid below */}
                <SectionReveal delay={0.4}>
                    <div className="grid md:grid-cols-2 gap-x-16 gap-y-12 mt-32 max-w-4xl mx-auto pt-16 border-t border-white/5 relative">
                        {/* Subtle glow on the top border */}
                        <div className="absolute top-0 left-1/2 -translate-x-1/2 w-1/2 h-px bg-gradient-to-r from-transparent via-brand-500/50 to-transparent" />

                        {/* Feature 1 */}
                        <div className="flex flex-col items-center text-center">
                            <div className="w-16 h-16 rounded-2xl bg-[#0a0a0a] border border-white/10 flex items-center justify-center mb-6 text-gray-300 shadow-inner">
                                <HiOutlineCode size={32} />
                            </div>
                            <h4 className="text-2xl font-semibold text-white mb-4 tracking-tight">{t('features.commit.title') as string}</h4>
                            <p className="text-gray-400 text-lg leading-relaxed">{t('features.commit.desc') as string}</p>
                        </div>

                        {/* Feature 2 */}
                        <div className="flex flex-col items-center text-center">
                            <div className="w-16 h-16 rounded-2xl bg-[#0a0a0a] border border-white/10 flex items-center justify-center mb-6 text-gray-300 shadow-inner">
                                <HiOutlineWifi size={32} />
                            </div>
                            <h4 className="text-2xl font-semibold text-white mb-4 tracking-tight">{t('features.offline.title') as string}</h4>
                            <p className="text-gray-400 text-lg leading-relaxed">{t('features.offline.desc') as string}</p>
                        </div>
                    </div>
                </SectionReveal>
            </Container>
        </section>
    );
}

/* ─────────────────────────────────────────────────────────────────────────
   Pillar 2: Governance Engine (Bleeding UI Layout)
───────────────────────────────────────────────────────────────────────── */
function GovernanceSection() {
    const { t } = useTranslation();

    return (
        <section id="governance" className="py-24 md:py-32 relative bg-[#000] overflow-hidden">
            <Container className="relative z-10">
                {/* Premium Card Container */}
                <SectionReveal>
                    <div className="relative w-full rounded-[32px] border border-white/10 bg-gradient-to-br from-[#0a0a0a] to-[#020202] shadow-[0_0_80px_rgba(0,0,0,0.8)] overflow-hidden grid lg:grid-cols-2">

                        {/* Left: Text Content */}
                        <div className="p-10 md:p-16 lg:p-20 flex flex-col justify-center relative z-20">
                            {/* Subtle background glow */}
                            <div className="absolute top-0 left-0 w-full h-full bg-brand-500/5 blur-[100px] pointer-events-none" />

                            <div className="relative">
                                <div className="inline-flex items-center gap-2 px-4 py-1.5 rounded-full bg-white/[0.03] border border-white/10 mb-8">
                                    <HiOutlineShieldCheck className="text-brand-500" size={16} />
                                    <span className="text-sm font-medium text-gray-300">{t('features.policy.badge') as string}</span>
                                </div>
                                <h2 className="text-4xl md:text-5xl font-bold text-white mb-6 tracking-tighter leading-tight">
                                    {t('features.policy.title') as string}
                                </h2>
                                <p className="text-lg text-gray-400 leading-relaxed mb-12">
                                    {t('features.policy.description') as string}
                                </p>

                                <div className="p-6 rounded-2xl bg-black/40 border border-white/10 backdrop-blur-md">
                                    <div className="flex items-center gap-4 mb-4">
                                        <div className="w-10 h-10 rounded-xl bg-white/[0.05] border border-white/10 flex items-center justify-center">
                                            <HiOutlineClipboardCheck className="text-brand-500" size={20} />
                                        </div>
                                        <h4 className="text-lg text-white font-semibold">{t('features.policy.check.title') as string}</h4>
                                    </div>
                                    <p className="text-sm text-gray-400 leading-relaxed">{t('features.policy.check.desc') as string}</p>
                                </div>
                            </div>
                        </div>

                        {/* Right: Bleeding Visual */}
                        <div className="relative bg-[#020202] border-t lg:border-t-0 lg:border-l border-white/5 min-h-[400px] lg:min-h-full overflow-hidden">
                            {/* Diagonal striped background */}
                            <div className="absolute inset-0 opacity-[0.03]" style={{ backgroundImage: 'repeating-linear-gradient(45deg, #fff 0, #fff 1px, transparent 0, transparent 50%)', backgroundSize: '20px 20px' }} />

                            {/* The Bleeding Mockup Container - Pinned to right and bottom */}
                            <div className="absolute top-12 left-8 md:left-16 lg:left-20 right-[-10px] bottom-[-10px]">
                                <div className="w-full h-full rounded-tl-3xl border-t border-l border-white/10 bg-[#050505] shadow-[-20px_-20px_60px_rgba(0,0,0,0.5)] p-8 md:p-10 flex flex-col">

                                    <div className="flex items-center justify-between mb-10 pb-6 border-b border-white/5">
                                        <div className="flex items-center gap-4">
                                            <div className="w-10 h-10 rounded-lg bg-brand-500/10 flex items-center justify-center border border-brand-500/20 shadow-[0_0_15px_rgba(249,115,22,0.1)]">
                                                <HiOutlineShieldCheck className="text-brand-500" size={20} />
                                            </div>
                                            <h3 className="text-lg text-white font-semibold tracking-wide">Branch Policies</h3>
                                        </div>
                                        <div className="px-3 py-1.5 rounded-full bg-brand-500/10 text-brand-500 text-[10px] font-bold uppercase tracking-widest border border-brand-500/20">Active</div>
                                    </div>

                                    <div className="space-y-6 flex-1">
                                        {/* Rule 1 */}
                                        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 p-6 rounded-2xl bg-[#0a0a0a] border border-white/5">
                                            <div>
                                                <div className="text-sm font-semibold text-white">Require Jira Ticket</div>
                                                <div className="text-xs text-gray-500 mt-1.5">Check commit messages.</div>
                                            </div>
                                            <div className="flex bg-[#000] rounded-xl p-1.5 border border-white/10 shrink-0">
                                                <div className="px-4 py-2 text-xs text-gray-500 rounded-lg font-medium transition-colors hover:text-white cursor-pointer">Off</div>
                                                <div className="px-4 py-2 text-xs text-brand-500 bg-brand-500/10 border border-brand-500/20 rounded-lg shadow-sm font-bold tracking-wide">Block</div>
                                            </div>
                                        </div>
                                        {/* Rule 2 */}
                                        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 p-6 rounded-2xl bg-[#0a0a0a] border border-white/5">
                                            <div>
                                                <div className="text-sm font-semibold text-white">Quality Gate</div>
                                                <div className="text-xs text-gray-500 mt-1.5">SonarQube must be green.</div>
                                            </div>
                                            <div className="flex bg-[#000] rounded-xl p-1.5 border border-white/10 shrink-0">
                                                <div className="px-4 py-2 text-xs text-white bg-white/10 border border-white/10 rounded-lg shadow-sm font-medium">Warn</div>
                                                <div className="px-4 py-2 text-xs text-gray-500 rounded-lg font-medium transition-colors hover:text-white cursor-pointer">Block</div>
                                            </div>
                                        </div>
                                        {/* Skeleton Rule 3 to show depth */}
                                        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 p-6 rounded-2xl bg-[#0a0a0a] border border-white/5 opacity-50">
                                            <div>
                                                <div className="h-4 w-32 bg-white/10 rounded mb-2" />
                                                <div className="h-3 w-48 bg-white/5 rounded" />
                                            </div>
                                            <div className="h-8 w-24 bg-white/10 rounded-xl" />
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </SectionReveal>
            </Container>
        </section>
    );
}

/* ─────────────────────────────────────────────────────────────────────────
   Pillar 3: Integrations & Correlation
───────────────────────────────────────────────────────────────────────── */
function IntegrationsSection() {
    const { t } = useTranslation();

    return (
        <section id="correlation" className="py-24 md:py-32 relative bg-[#050505]">
            <Container>
                <div className="text-center max-w-3xl mx-auto mb-20">
                    <SectionBadge icon={<HiOutlinePuzzle />} label={t('features.integrations.badge') as string} />
                    <h2 className="text-4xl md:text-5xl font-bold text-white mb-6">
                        {t('features.integrations.title') as string}
                    </h2>
                    <p className="text-lg text-gray-400">
                        {t('features.integrations.description') as string}
                    </p>
                </div>

                <SectionReveal>
                    <div className="grid md:grid-cols-3 gap-6 max-w-5xl mx-auto">
                        {/* Git Card */}
                        <div className="p-8 rounded-2xl border border-white/5 bg-[#0a0a0a] hover:bg-white/[0.02] transition-colors group">
                            <div className="w-14 h-14 rounded-xl bg-white/[0.03] border border-white/5 flex items-center justify-center mb-6 group-hover:scale-110 transition-transform">
                                <HiOutlineCode className="text-gray-300" size={28} />
                            </div>
                            <h3 className="text-xl font-semibold text-white mb-3">{t('features.github.title') as string}</h3>
                            <p className="text-sm text-gray-500 leading-relaxed">{t('features.github.desc') as string}</p>
                        </div>
                        {/* CI Card */}
                        <div className="p-8 rounded-2xl border border-white/5 bg-[#0a0a0a] hover:bg-white/[0.02] transition-colors group relative overflow-hidden">
                            <div className="absolute inset-0 bg-gradient-to-br from-brand-500/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity" />
                            <div className="relative z-10">
                                <div className="w-14 h-14 rounded-xl bg-brand-500/10 border border-brand-500/20 flex items-center justify-center mb-6 group-hover:scale-110 transition-transform">
                                    <HiOutlineLightningBolt className="text-brand-400" size={28} />
                                </div>
                                <h3 className="text-xl font-semibold text-white mb-3">{t('features.jenkins.title') as string}</h3>
                                <p className="text-sm text-gray-500 leading-relaxed">{t('features.jenkins.desc') as string}</p>
                            </div>
                        </div>
                        {/* Jira Card */}
                        <div className="p-8 rounded-2xl border border-white/5 bg-[#0a0a0a] hover:bg-white/[0.02] transition-colors group">
                            <div className="w-14 h-14 rounded-xl bg-white/[0.03] border border-white/5 flex items-center justify-center mb-6 group-hover:scale-110 transition-transform">
                                <HiOutlineDocumentSearch className="text-gray-300" size={28} />
                            </div>
                            <h3 className="text-xl font-semibold text-white mb-3">{t('features.jira.title') as string}</h3>
                            <p className="text-sm text-gray-500 leading-relaxed">{t('features.jira.desc') as string}</p>
                        </div>
                    </div>
                </SectionReveal>
            </Container>
        </section>
    );
}

/* ─────────────────────────────────────────────────────────────────────────
   Pillar 4: Reporting & Dashboard (Wide Top-to-Bottom Layout)
───────────────────────────────────────────────────────────────────────── */
function ReportingSection() {
    const { t } = useTranslation();

    const auditItems = [
        t('features.risk.audit.item1') as string,
        t('features.risk.audit.item2') as string,
        t('features.risk.audit.item3') as string,
    ];

    return (
        <section id="reporting" className="py-24 md:py-32 relative bg-[#000] overflow-hidden">
            <Container className="relative z-10">

                {/* Header Block */}
                <SectionReveal>
                    <div className="flex flex-col lg:flex-row items-end justify-between gap-12 mb-20 max-w-6xl mx-auto">
                        <div className="max-w-2xl">
                            <div className="inline-flex items-center gap-2 px-4 py-1.5 rounded-full bg-white/[0.03] border border-white/10 mb-8">
                                <HiOutlineTrendingUp className="text-brand-500" size={16} />
                                <span className="text-sm font-medium text-gray-300">{t('features.risk.badge') as string}</span>
                            </div>
                            <h2 className="text-4xl md:text-5xl font-bold text-white mb-6 tracking-tighter leading-tight">
                                {t('features.risk.title') as string}
                            </h2>
                            <p className="text-lg text-gray-400 leading-relaxed">
                                {t('features.risk.description') as string}
                            </p>
                        </div>

                        <div className="flex-1 w-full lg:w-auto bg-[#050505] p-6 rounded-3xl border border-white/5 relative overflow-hidden">
                            <div className="absolute top-0 right-0 w-32 h-32 bg-brand-500/5 blur-[50px] rounded-full" />
                            <div className="space-y-4 relative z-10">
                                {auditItems.map((item, idx) => (
                                    <div key={idx} className="flex items-center gap-4 text-gray-300">
                                        <div className="w-8 h-8 rounded-full bg-brand-500/10 flex items-center justify-center border border-brand-500/20 shrink-0">
                                            <HiOutlineLockClosed className="text-brand-500" size={14} />
                                        </div>
                                        <p className="text-sm font-medium">{item}</p>
                                    </div>
                                ))}
                            </div>
                        </div>
                    </div>
                </SectionReveal>

                {/* Massive Dashboard Visual */}
                <SectionReveal delay={0.2}>
                    <div className="relative max-w-6xl mx-auto">
                        {/* Glow under dashboard */}
                        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[90%] h-[50%] bg-brand-500/10 blur-[100px] rounded-full pointer-events-none" />

                        {/* Dashboard Container */}
                        <div className="w-full rounded-[32px] border border-white/10 bg-gradient-to-br from-[#0a0a0a] to-[#020202] shadow-[0_0_80px_rgba(0,0,0,0.6)] p-6 md:p-10 relative z-10 overflow-hidden">

                            {/* Dashboard Header */}
                            <div className="flex flex-col md:flex-row md:items-center justify-between mb-10 pb-6 border-b border-white/5 gap-6">
                                <div>
                                    <div className="text-2xl text-white font-bold tracking-tight">Live Compliance</div>
                                    <div className="text-sm text-gray-500 mt-1 font-mono">Control Plane Monitoring</div>
                                </div>
                                <div className="flex items-center gap-3">
                                    <div className="flex items-center gap-2 px-4 py-2 rounded-xl bg-[#050505] border border-white/10 shadow-inner">
                                        <div className="w-2 h-2 rounded-full bg-green-500 animate-pulse" />
                                        <span className="text-xs font-bold text-gray-300 uppercase tracking-widest">Real-time sync</span>
                                    </div>
                                </div>
                            </div>

                            {/* Dashboard Metrics Grid */}
                            <div className="grid grid-cols-1 md:grid-cols-3 gap-6">

                                {/* Large Circular Metric */}
                                <div className="md:col-span-1 p-8 rounded-2xl bg-[#050505] border border-white/5 shadow-inner flex flex-col items-center justify-center text-center">
                                    <div className="relative w-32 h-32 mb-6">
                                        <svg className="w-full h-full -rotate-90 drop-shadow-[0_0_12px_rgba(249,115,22,0.4)]" viewBox="0 0 36 36">
                                            <circle cx="18" cy="18" r="15.9155" fill="none" stroke="#111" strokeWidth="3" />
                                            <path
                                                className="text-brand-500 transition-all duration-1000 ease-out"
                                                strokeDasharray="94, 100"
                                                stroke="currentColor"
                                                strokeWidth="3"
                                                strokeLinecap="round"
                                                fill="none"
                                                d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831"
                                            />
                                        </svg>
                                        <div className="absolute inset-0 flex flex-col items-center justify-center">
                                            <span className="text-4xl font-bold text-white tracking-tighter">94%</span>
                                        </div>
                                    </div>
                                    <div className="text-sm font-bold text-gray-400 uppercase tracking-widest">Ticket Coverage</div>
                                </div>

                                {/* Stacked Metrics */}
                                <div className="md:col-span-2 grid grid-rows-2 gap-6">
                                    {/* Alert Metric */}
                                    <div className="p-6 md:p-8 rounded-2xl bg-[#050505] border border-white/5 flex flex-col sm:flex-row sm:items-center justify-between gap-6 shadow-inner">
                                        <div>
                                            <div className="flex items-center gap-3 mb-2">
                                                <div className="w-8 h-8 rounded-full bg-yellow-500/10 flex items-center justify-center border border-yellow-500/20">
                                                    <span className="text-yellow-500 text-sm font-bold">!</span>
                                                </div>
                                                <div className="text-xs text-gray-500 uppercase tracking-widest font-bold">Risk Results</div>
                                            </div>
                                            <div className="text-xl text-white font-medium mt-4">3 Pipelines flagged for audit</div>
                                        </div>
                                        <div className="px-6 py-3 rounded-xl bg-yellow-500/10 text-yellow-500 text-sm font-bold border border-yellow-500/20 hover:bg-yellow-500/20 transition-colors cursor-pointer text-center">
                                            Review
                                        </div>
                                    </div>

                                    {/* Stats Grid */}
                                    <div className="grid grid-cols-2 gap-6">
                                        <div className="p-6 rounded-2xl bg-[#050505] border border-white/5 shadow-inner">
                                            <div className="text-xs text-gray-500 uppercase tracking-widest mb-2 font-bold">Monitored Repos</div>
                                            <div className="text-3xl font-bold text-white">12</div>
                                        </div>
                                        <div className="p-6 rounded-2xl bg-[#050505] border border-white/5 shadow-inner">
                                            <div className="text-xs text-gray-500 uppercase tracking-widest mb-2 font-bold">Audit Exports</div>
                                            <div className="text-3xl font-bold text-brand-500">2,408</div>
                                        </div>
                                    </div>
                                </div>

                            </div>
                        </div>
                    </div>
                </SectionReveal>
            </Container>
        </section>
    );
}

/* ─────────────────────────────────────────────────────────────────────────
   Main Component
───────────────────────────────────────────────────────────────────────── */
export function FeaturesClient() {
    const { t } = useTranslation();

    return (
        <div className="bg-[#030303] min-h-screen selection:bg-brand-500/30">
            <FeaturesHero />
            <CaptureSection />
            <GovernanceSection />
            <IntegrationsSection />
            <ReportingSection />
            <CTASection
                title={t('features.cta.title') as string}
                titleAccent={t('features.cta.titleAccent') as string}
                description={t('features.cta.desc') as string}
                primaryCta={{ label: t('features.cta.primary') as string, href: '/download' }}
                secondaryCta={{ label: t('features.cta.secondary') as string, href: '/docs' }}
            />
        </div>
    );
}
