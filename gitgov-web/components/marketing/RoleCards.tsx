'use client';

import React, { useState } from 'react';
import { SectionReveal } from '@/components/ui/SectionReveal';
import { useTranslation } from '@/lib/i18n';
import { HiOutlineCode, HiOutlineShieldCheck, HiOutlineTicket } from 'react-icons/hi';

interface RoleCardData {
    icon: React.ReactNode;
    role: string;
    painPoint: string;
    solution: string;
}

interface RoleCardsProps {
    roles: RoleCardData[];
}

export function RoleCards({ roles }: RoleCardsProps) {
    const { t } = useTranslation();
    const [activeIndex, setActiveIndex] = useState(0);

    const activeRole = roles[activeIndex];

    return (
        <div className="grid lg:grid-cols-12 gap-6 lg:gap-10 items-stretch mt-6">
            {/* Left Column: Vertical Menu */}
            <div className="lg:col-span-5 flex flex-col justify-between gap-2 relative z-10">
                {roles.map((role, i) => {
                    const isActive = i === activeIndex;
                    return (
                        <button
                            key={role.role}
                            onClick={() => setActiveIndex(i)}
                            className={`flex items-center gap-4 p-4 rounded-2xl text-left transition-all duration-300 w-full group relative overflow-hidden ${
                                isActive
                                    ? 'bg-surface-200 border-white/[0.08] shadow-[0_0_30px_rgba(249,115,22,0.1)]'
                                    : 'bg-transparent border-transparent hover:bg-white/[0.02]'
                            } border`}
                        >
                            {/* Active background glow */}
                            {isActive && (
                                <div className="absolute inset-0 bg-gradient-to-r from-brand-500/10 to-transparent opacity-50" />
                            )}
                            
                            <div className={`w-12 h-12 rounded-xl flex items-center justify-center shrink-0 transition-all duration-300 relative z-10 ${
                                isActive 
                                    ? 'bg-brand-500/20 border border-brand-500/30 text-brand-400' 
                                    : 'bg-white/[0.03] border border-white/[0.05] text-gray-500 group-hover:text-gray-300'
                            }`}>
                                {role.icon}
                            </div>
                            <span className={`font-semibold text-lg transition-colors duration-300 relative z-10 ${
                                isActive ? 'text-white' : 'text-gray-500 group-hover:text-gray-300'
                            }`}>
                                {role.role}
                            </span>
                        </button>
                    );
                })}
            </div>

            {/* Right Column: Dynamic Content Canvas */}
            <div className="lg:col-span-7 relative h-full">
                <SectionReveal key={activeIndex} className="h-full">
                    <div className="relative rounded-3xl bg-surface-200 border border-white/[0.05] p-[1px] overflow-hidden group h-full flex flex-col justify-between">
                        {/* Background structural glow */}
                        <div className="absolute top-0 right-0 w-96 h-96 bg-brand-500/10 blur-[100px] rounded-full pointer-events-none transition-all duration-700" />
                        
                        <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_top_right,rgba(255,255,255,0.02),transparent_50%)]" />

                        {/* Top Contextual SVG visualization */}
                        <div className="relative h-56 w-full border-b border-white/[0.03] overflow-hidden flex items-center justify-center bg-[#090909]">
                            {/* SVG Grid background */}
                            <svg className="absolute inset-0 w-full h-full opacity-[0.15]" stroke="url(#grid-pattern)">
                                <defs>
                                    <pattern id="grid-pattern" width="40" height="40" patternUnits="userSpaceOnUse">
                                        <path d="M 40 0 L 0 0 0 40" fill="none" stroke="currentColor" strokeWidth="1" />
                                    </pattern>
                                </defs>
                                <rect width="100%" height="100%" fill="url(#grid-pattern)" />
                            </svg>
                            
                            <div className="absolute inset-0 flex items-center justify-center">
                                {activeIndex === 0 && <SVGCompliance />}
                                {activeIndex === 1 && <SVGVisibility />}
                                {activeIndex === 2 && <SVGEnforcement />}
                                {activeIndex === 3 && <SVGAuditTrail />}
                                {activeIndex === 4 && <SVGOrgDashboard />}
                            </div>
                        </div>

                        <div className="p-8 md:p-10 relative z-10 flex-1 flex flex-col gap-8 bg-gradient-to-br from-surface-300 to-transparent backdrop-blur-sm">
                            {/* Challenge */}
                            <div>
                                <div className="flex items-center gap-2 mb-3 text-accent-400">
                                    <div className="w-1.5 h-1.5 rounded-full bg-accent-400 font-bold" />
                                    <span className="text-[10px] uppercase tracking-widest font-black">{t('challenge')}</span>
                                </div>
                                <p className="text-gray-400 leading-relaxed text-sm">
                                    {activeRole.painPoint}
                                </p>
                            </div>

                            {/* Solution */}
                            <div>
                                <div className="flex items-center gap-2 mb-3 text-brand-400">
                                    <div className="w-1.5 h-1.5 rounded-full bg-brand-400 font-bold shadow-[0_0_10px_rgba(249,115,22,0.8)]" />
                                    <span className="text-[10px] uppercase tracking-widest font-black">{t('withGitGov')}</span>
                                </div>
                                <p className="text-gray-300 leading-relaxed text-sm">
                                    {activeRole.solution}
                                </p>
                            </div>
                        </div>
                    </div>
                </SectionReveal>
            </div>
        </div>
    );
}

// Highly stylized SVG components to represent each role's persona

const SVGCompliance = () => (
    <div className="w-full h-full flex items-center justify-center animate-fade-in relative pt-4">
        <svg viewBox="0 0 400 150" className="w-full max-w-[300px] mix-blend-screen opacity-80" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path d="M50 75 h100 l30 -30 h100 l40 40 h50" stroke="#f97316" strokeWidth="2" strokeOpacity="0.3" strokeDasharray="4 4" className="animate-[slide-up_2s_ease-out_infinite]"/>
            <path d="M50 75 h100 l30 -30 h100 l40 40 h50" stroke="#f97316" strokeWidth="2" strokeDashoffset="100" strokeDasharray="100 100" className="animate-[pulseGlow_3s_infinite]"/>
            {/* Compliance node */}
            <circle cx="150" cy="75" r="15" fill="#141414" stroke="#f97316" strokeWidth="3" />
            <circle cx="280" cy="45" r="15" fill="#141414" stroke="#fbbf24" strokeWidth="3" />
            <path d="M142 75 l6 6 l10 -10" stroke="#f97316" strokeWidth="2" strokeLinecap="round" />
            <path d="M272 45 l6 6 l10 -10" stroke="#fbbf24" strokeWidth="2" strokeLinecap="round" />
            <rect x="130" y="20" width="40" height="20" rx="4" fill="#f97316" fillOpacity="0.1" stroke="#f97316" strokeWidth="1" />
            <text x="150" y="34" fill="#f97316" fontSize="10" textAnchor="middle" fontWeight="bold" letterSpacing="1">SECURE</text>
        </svg>
    </div>
);

const SVGVisibility = () => (
    <div className="w-full h-full flex items-center justify-center animate-fade-in relative pt-4">
         <svg viewBox="0 0 400 150" className="w-full max-w-[300px] mix-blend-screen opacity-80" fill="none" xmlns="http://www.w3.org/2000/svg">
            <circle cx="100" cy="75" r="30" stroke="#f97316" strokeWidth="1" strokeOpacity="0.2" className="animate-[spin_4s_linear_infinite]" strokeDasharray="10 5" />
            <circle cx="200" cy="75" r="40" stroke="#f97316" strokeWidth="1" strokeOpacity="0.5" className="animate-[spin_6s_linear_infinite]" strokeDasharray="5 15" />
            <circle cx="300" cy="75" r="30" stroke="#f97316" strokeWidth="1" strokeOpacity="0.2" className="animate-[spin_4s_linear_infinite_reverse]" strokeDasharray="10 5" />
            <path d="M130 75 Q165 50 200 75 T270 75" stroke="#fbbf24" strokeWidth="2" fill="none" opacity="0.6"/>
            <path d="M130 75 Q165 100 200 75 T270 75" stroke="#fbbf24" strokeWidth="2" fill="none" opacity="0.6" strokeDasharray="4 4" />
            <rect x="175" y="60" width="50" height="30" rx="8" fill="#141414" stroke="#f97316" strokeWidth="2" />
            <path d="M190 75 h20" stroke="#f97316" strokeWidth="2" strokeLinecap="round" />
            <path d="M190 68 h20" stroke="#f97316" strokeWidth="2" strokeLinecap="round" opacity="0.5" />
            <path d="M190 82 h10" stroke="#f97316" strokeWidth="2" strokeLinecap="round" opacity="0.5" />
         </svg>
    </div>
);

const SVGEnforcement = () => (
    <div className="w-full h-full flex items-center justify-center animate-fade-in relative pt-4">
        <svg viewBox="0 0 400 150" className="w-full max-w-[300px] mix-blend-screen opacity-80" fill="none" xmlns="http://www.w3.org/2000/svg">
             <path d="M0 75 h150" stroke="#f97316" strokeWidth="3" opacity="0.3" />
             <path d="M0 75 h150" stroke="#f97316" strokeWidth="3" className="animate-[pulseGlow_2s_infinite]"/>
             
             {/* Developer push */}
             <rect x="50" y="55" width="40" height="40" rx="10" fill="#141414" stroke="#f97316" strokeWidth="2" />
             <path d="M60 75 l10 -10 l10 10 M70 65 v20" stroke="#f97316" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
             
             {/* Gateway block */}
             <rect x="200" y="40" width="10" height="70" rx="5" fill="#f97316" />
             
             <path d="M210 75 h150" stroke="#f97316" strokeWidth="1" strokeDasharray="4 4" opacity="0.2" />
             
             <rect x="160" y="20" width="90" height="20" rx="4" fill="#fbbf24" fillOpacity="0.1" stroke="#fbbf24" strokeWidth="1" />
             <text x="205" y="34" fill="#fbbf24" fontSize="10" textAnchor="middle" fontWeight="bold" letterSpacing="1">POLICY GUARD</text>
        </svg>
    </div>
);

const SVGAuditTrail = () => (
    <div className="w-full h-full flex items-center justify-center animate-fade-in relative pt-4">
        <svg viewBox="0 0 400 150" className="w-full max-w-[300px] mix-blend-screen opacity-80" fill="none" xmlns="http://www.w3.org/2000/svg">
            {/* Stacked document layers */}
            <rect x="60" y="25" width="80" height="100" rx="6" fill="#141414" stroke="#f97316" strokeWidth="1" strokeOpacity="0.3" />
            <rect x="70" y="20" width="80" height="100" rx="6" fill="#141414" stroke="#f97316" strokeWidth="1" strokeOpacity="0.5" />
            <rect x="80" y="15" width="80" height="100" rx="6" fill="#141414" stroke="#f97316" strokeWidth="2" />
            
            {/* Document lines */}
            <path d="M95 35 h50" stroke="#f97316" strokeWidth="2" strokeLinecap="round" opacity="0.6" />
            <path d="M95 48 h40" stroke="#f97316" strokeWidth="1.5" strokeLinecap="round" opacity="0.3" />
            <path d="M95 58 h45" stroke="#f97316" strokeWidth="1.5" strokeLinecap="round" opacity="0.3" />
            <path d="M95 68 h30" stroke="#f97316" strokeWidth="1.5" strokeLinecap="round" opacity="0.3" />
            
            {/* Checkmark seal */}
            <circle cx="140" cy="95" r="12" fill="#141414" stroke="#fbbf24" strokeWidth="2" />
            <path d="M133 95 l5 5 l9 -9" stroke="#fbbf24" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />

            {/* Arrow to output */}
            <path d="M180 65 h40" stroke="#f97316" strokeWidth="2" strokeDasharray="4 4" className="animate-[pulseGlow_2s_infinite]"/>
            <path d="M215 60 l10 5 l-10 5" stroke="#f97316" strokeWidth="2" fill="none" />

            {/* SOC2 badge */}
            <rect x="240" y="40" width="100" height="50" rx="8" fill="#141414" stroke="#fbbf24" strokeWidth="2" />
            <text x="290" y="60" fill="#fbbf24" fontSize="11" textAnchor="middle" fontWeight="bold" letterSpacing="1">SOC2</text>
            <text x="290" y="78" fill="#f97316" fontSize="9" textAnchor="middle" opacity="0.7">AUDIT READY</text>
        </svg>
    </div>
);

const SVGOrgDashboard = () => (
    <div className="w-full h-full flex items-center justify-center animate-fade-in relative pt-4">
        <svg viewBox="0 0 400 150" className="w-full max-w-[300px] mix-blend-screen opacity-80" fill="none" xmlns="http://www.w3.org/2000/svg">
            {/* Dashboard frame */}
            <rect x="50" y="15" width="300" height="120" rx="10" fill="#141414" stroke="#f97316" strokeWidth="2" />
            
            {/* Top bar */}
            <rect x="50" y="15" width="300" height="25" rx="10" fill="#1a1a1a" stroke="#f97316" strokeWidth="1" strokeOpacity="0.3" />
            <circle cx="70" cy="27.5" r="4" fill="#f97316" opacity="0.5" />
            <circle cx="84" cy="27.5" r="4" fill="#fbbf24" opacity="0.3" />
            <circle cx="98" cy="27.5" r="4" fill="#f97316" opacity="0.2" />

            {/* Bar chart */}
            <rect x="75" y="100" width="20" height="25" rx="2" fill="#f97316" fillOpacity="0.4" />
            <rect x="105" y="85" width="20" height="40" rx="2" fill="#f97316" fillOpacity="0.6" />
            <rect x="135" y="70" width="20" height="55" rx="2" fill="#f97316" fillOpacity="0.8" />
            <rect x="165" y="60" width="20" height="65" rx="2" fill="#fbbf24" className="animate-pulse" />

            {/* Metrics panel */}
            <rect x="210" y="50" width="120" height="30" rx="4" fill="#1a1a1a" stroke="#f97316" strokeWidth="1" strokeOpacity="0.3" />
            <text x="220" y="63" fill="#f97316" fontSize="8" opacity="0.6">COMPLIANCE</text>
            <text x="310" y="63" fill="#fbbf24" fontSize="10" textAnchor="end" fontWeight="bold">97.2%</text>

            <rect x="210" y="90" width="120" height="30" rx="4" fill="#1a1a1a" stroke="#f97316" strokeWidth="1" strokeOpacity="0.3" />
            <text x="220" y="103" fill="#f97316" fontSize="8" opacity="0.6">TEAMS</text>
            <text x="310" y="103" fill="#fbbf24" fontSize="10" textAnchor="end" fontWeight="bold">12 / 12</text>
        </svg>
    </div>
);
