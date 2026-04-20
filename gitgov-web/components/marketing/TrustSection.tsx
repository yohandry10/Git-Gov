'use client';

import React from 'react';
import { Container } from '@/components/layout/Container';
import { SectionReveal } from '@/components/ui/SectionReveal';
import { useTranslation } from '@/lib/i18n';
import { HiOutlineLockClosed } from 'react-icons/hi';

export function TrustSection() {
    const { t } = useTranslation();

    return (
        <section className="py-12 md:py-16 bg-surface-100/30" id="trust">
            <Container>
                <SectionReveal>
                    <div className="text-center max-w-3xl mx-auto mb-10">
                        <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full border border-brand-500/20 bg-brand-500/5 text-brand-400 text-xs font-semibold tracking-wide uppercase mb-6">
                            <HiOutlineLockClosed size={14} />
                            {t('trust.badge') as string}
                        </div>
                        
                        <h2 className="text-3xl md:text-4xl font-bold tracking-tight text-white mb-6">
                            {t('trust.title') as string}{' '}
                            <span className="text-transparent bg-clip-text bg-gradient-to-r from-brand-400 to-accent-300">
                                {t('trust.titleAccent') as string}
                            </span>
                        </h2>
                    </div>
                </SectionReveal>

                {/* Asymmetric Bento Box Grid */}
                <div className="grid grid-cols-1 md:grid-cols-3 gap-4 max-w-6xl mx-auto">
                    
                    {/* Bento 1: Append Only (Large, spans 2 cols, 2 rows logically) */}
                    <SectionReveal className="md:col-span-2 md:row-span-2">
                        <div className="h-full relative rounded-2xl p-6 overflow-hidden group border border-white/[0.05] bg-surface-200 hover:border-white/[0.1] transition-colors duration-500 flex flex-col justify-end">
                            <div className="absolute inset-0 bg-gradient-to-br from-brand-500/10 via-transparent to-transparent opacity-50 relative z-0" />
                            
                            {/* SVG Abstract Background */}
                            <div className="absolute right-0 top-1/2 -translate-y-1/2 w-[60%] h-full opacity-60 group-hover:opacity-100 transition-opacity duration-700 pointer-events-none">
                                <SVGBigDataStream />
                            </div>

                            <div className="relative z-10 max-w-[50%] mt-auto">
                                <h3 className="text-2xl font-bold text-white mb-4">
                                    {t('trust.appendonly.title') as string}
                                </h3>
                                <p className="text-gray-400 leading-relaxed">
                                    {t('trust.appendonly.desc') as string}
                                </p>
                            </div>
                        </div>
                    </SectionReveal>

                    {/* Bento 2: Self Hosted */}
                    <SectionReveal delay={0.1}>
                        <div className="h-full relative rounded-2xl p-6 overflow-hidden group border border-white/[0.05] bg-surface-200 hover:border-white/[0.1] transition-colors duration-500">
                            <div className="absolute inset-0 bg-[radial-gradient(circle_at_100%_0%,rgba(249,115,22,0.08),transparent_70%)]" />
                            
                            <div className="mb-3 overflow-hidden rounded-lg bg-[#090909] aspect-[5/2] flex items-center justify-center relative border border-white/[0.02]">
                                <SVGServerNodes />
                            </div>

                            <div className="relative z-10 mt-auto">
                                <h3 className="text-lg font-bold text-white mb-2">
                                    {t('trust.selfhosted.title') as string}
                                </h3>
                                <p className="text-sm text-gray-400 leading-relaxed">
                                    {t('trust.selfhosted.desc') as string}
                                </p>
                            </div>
                        </div>
                    </SectionReveal>

                    {/* Bento 3: Encrypted */}
                    <SectionReveal delay={0.2}>
                        <div className="h-full relative rounded-2xl p-6 overflow-hidden group border border-white/[0.05] bg-surface-200 hover:border-white/[0.1] transition-colors duration-500">
                             <div className="absolute inset-0 bg-[radial-gradient(circle_at_100%_100%,rgba(251,191,36,0.08),transparent_70%)]" />
                            
                            <div className="mb-3 overflow-hidden rounded-lg bg-[#090909] aspect-[5/2] flex items-center justify-center relative border border-white/[0.02]">
                                <SVGCryptoLock />
                            </div>

                            <div className="relative z-10 mt-auto">
                                <h3 className="text-lg font-bold text-white mb-2">
                                    {t('trust.encrypted.title') as string}
                                </h3>
                                <p className="text-sm text-gray-400 leading-relaxed">
                                    {t('trust.encrypted.desc') as string}
                                </p>
                            </div>
                        </div>
                    </SectionReveal>

                    {/* Bento 4: Metadata Only (Wide span) */}
                    <SectionReveal delay={0.3} className="md:col-span-3">
                        <div className="relative rounded-2xl p-6 md:p-8 overflow-hidden group border border-white/[0.05] bg-surface-200 hover:border-white/[0.1] transition-colors duration-500 flex flex-col md:flex-row items-center gap-6">
                            
                            <div className="flex-1 relative z-10">
                                <h3 className="text-xl md:text-2xl font-bold text-white mb-3">
                                    {t('trust.metadata.title') as string}
                                </h3>
                                <p className="text-gray-400 leading-relaxed max-w-2xl">
                                    {t('trust.metadata.desc') as string}
                                </p>
                            </div>
                            
                            <div className="w-full md:w-72 h-24 relative rounded-lg bg-[#141414] border border-white/[0.03] overflow-hidden flex items-center justify-center shrink-0">
                                <SVGMetadataFilter />
                            </div>

                        </div>
                    </SectionReveal>

                </div>
            </Container>
        </section>
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// CUSTOM BENTO SVGS
// ─────────────────────────────────────────────────────────────────────────────

const SVGBigDataStream = () => (
    <svg viewBox="0 0 400 400" className="w-[150%] h-[150%] absolute right-[-20%] top-[-25%] rotate-[-15deg] mix-blend-screen" fill="none" xmlns="http://www.w3.org/2000/svg">
         <defs>
            <linearGradient id="stream-fade" x1="0" y1="0" x2="400" y2="400" gradientUnits="userSpaceOnUse">
                <stop offset="0%" stopColor="#f97316" stopOpacity="0" />
                <stop offset="50%" stopColor="#f97316" stopOpacity="0.8" />
                <stop offset="100%" stopColor="#f97316" stopOpacity="0" />
            </linearGradient>
            <filter id="glow-heavy" x="-50%" y="-50%" width="200%" height="200%">
                <feGaussianBlur stdDeviation="8" result="blur" />
                <feComposite in="SourceGraphic" in2="blur" operator="over" />
            </filter>
        </defs>
        
        {/* DB Blocks overlapping */}
        {[0,1,2,3,4].map((i) => (
            <g key={i} transform={`translate(${i * 60}, ${i * 60})`} className="animate-float" style={{ animationDelay: `${i * 0.4}s`}}>
                <path d="M50 100 L100 125 L150 100 L100 75 Z" fill="#141414" stroke="url(#stream-fade)" strokeWidth="2" />
                <path d="M50 100 L100 125 L100 150 L50 125 Z" fill="#141414" stroke="url(#stream-fade)" strokeWidth="1" strokeOpacity="0.5"/>
                <path d="M150 100 L100 125 L100 150 L150 125 Z" fill="#141414" stroke="url(#stream-fade)" strokeWidth="1" strokeOpacity="0.5"/>
                {/* Hash signature */}
                <circle cx="100" cy="100" r="4" fill="#fbbf24" filter="url(#glow-heavy)" />
            </g>
        ))}

        {/* Connection line */}
        <path d="M100 100 L340 340" stroke="#f97316" strokeWidth="2" strokeDasharray="4 8" className="animate-[slide-up_4s_linear_infinite]" opacity="0.5" />
    </svg>
);

const SVGServerNodes = () => (
    <svg viewBox="0 0 200 150" className="w-[80%] h-full mix-blend-screen opacity-90" fill="none" xmlns="http://www.w3.org/2000/svg">
        <rect x="40" y="30" width="120" height="25" rx="4" fill="#1a1a1a" stroke="#f97316" strokeWidth="1.5" strokeOpacity="0.4" />
        <rect x="40" y="65" width="120" height="25" rx="4" fill="#1a1a1a" stroke="#f97316" strokeWidth="1.5" />
        <rect x="40" y="100" width="120" height="25" rx="4" fill="#1a1a1a" stroke="#f97316" strokeWidth="1.5" strokeOpacity="0.4" />
        
        {/* Lights */}
        <circle cx="145" cy="42.5" r="3" fill="#f97316" opacity="0.3" />
        <circle cx="145" cy="77.5" r="3" fill="#fbbf24" className="animate-pulse" />
        <circle cx="145" cy="112.5" r="3" fill="#f97316" opacity="0.3" />

        {/* Data processing pulses */}
        <path d="M50 77.5 h30" stroke="#fbbf24" strokeWidth="3" strokeLinecap="round" strokeDasharray="5 10" className="animate-[slide-up_2s_linear_infinite]" />
    </svg>
);

const SVGCryptoLock = () => (
    <svg viewBox="0 0 200 150" className="w-[80%] h-full mix-blend-screen opacity-90" fill="none" xmlns="http://www.w3.org/2000/svg">
        {/* Shield background */}
        <path d="M100 20 L160 40 L150 110 C150 110 100 140 100 140 C100 140 50 110 50 110 L40 40 Z" fill="#0f0f0f" stroke="#fbbf24" strokeWidth="1" strokeOpacity="0.3" />
        
        {/* TLS waveform */}
        <path d="M30 75 Q 65 30 100 75 T 170 75" stroke="#f97316" strokeWidth="2" opacity="0.4" fill="none" />
        <path d="M30 75 Q 65 120 100 75 T 170 75" stroke="#f97316" strokeWidth="2" opacity="0.4" fill="none" />

        {/* Lock kernel */}
        <rect x="80" y="70" width="40" height="30" rx="4" fill="#141414" stroke="#fbbf24" strokeWidth="2" />
        <path d="M88 70 V 55 A 12 12 0 0 1 112 55 V 70" stroke="#fbbf24" strokeWidth="2" />
        <circle cx="100" cy="85" r="4" fill="#fbbf24" />
    </svg>
);

const SVGMetadataFilter = () => (
    <svg viewBox="0 0 400 100" className="w-full h-full mix-blend-screen" fill="none" xmlns="http://www.w3.org/2000/svg">
        {/* Source Code entering */}
        <path d="M0 30 h100 M0 50 h100 M0 70 h100" stroke="#ffffff" strokeWidth="4" strokeOpacity="0.1" strokeDasharray="10 5" />
        
        {/* The Filter (Workstation Agent) */}
        <rect x="120" y="20" width="60" height="60" rx="10" fill="#f97316" fillOpacity="0.1" stroke="#f97316" strokeWidth="2" className="animate-pulse" />
        <path d="M150 35 v30" stroke="#f97316" strokeWidth="4" strokeLinecap="round"/>
        <path d="M135 50 h30" stroke="#f97316" strokeWidth="4" strokeLinecap="round"/>
        
        {/* Output: Only metadata survives (orange glowing dots) */}
        <path d="M200 40 h200" stroke="#fbbf24" strokeWidth="2" strokeDasharray="4 8" className="animate-[slide-up_5s_linear_infinite]"/>
        <path d="M200 60 h200" stroke="#f97316" strokeWidth="2" strokeDasharray="4 8" className="animate-[slide-up_3s_linear_infinite]"/>
        
        <circle cx="250" cy="40" r="4" fill="#fbbf24" />
        <circle cx="320" cy="60" r="4" fill="#f97316" />
        <circle cx="380" cy="40" r="4" fill="#fbbf24" />
    </svg>
);
