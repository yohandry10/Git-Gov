'use client';

import React, { useRef, useState } from 'react';
import { motion, useScroll, useTransform, useSpring, useMotionValue } from 'framer-motion';
import Link from 'next/link';
import { Container } from '@/components/layout/Container';
import { siteConfig } from '@/lib/config/site';
import { HiOutlineArrowRight, HiOutlineShieldCheck, HiOutlineDatabase } from 'react-icons/hi';
import { useTranslation } from '@/lib/i18n';

/* ═══════════════════════════════════════════════════════
   THE SPATIAL HOLOGRAM (Stanford Elite Level)
   Asymmetrical cinematic layout. Left: God-tier Typography.
   Right: Absolute vector-math Governance Hologram.
   ═══════════════════════════════════════════════════════ */

export function Hero() {
    const { t } = useTranslation();
    const { scrollYProgress } = useScroll();
    
    // Parallax & Scroll transforms for content
    const yText = useTransform(scrollYProgress, [0, 0.3], [0, -60]);
    const opacityText = useTransform(scrollYProgress, [0, 0.25], [1, 0]);

    // Mouse Tracking for 3D Tilt on Hologram
    const areaRef = useRef<HTMLElement>(null);
    const mouseX = useMotionValue(0.5);
    const mouseY = useMotionValue(0.5);
    
    const springConfig = { damping: 40, stiffness: 150, mass: 1 };
    
    // Calculate rotation limits (-15deg to 15deg) around the axes
    const rotateX = useSpring(useTransform(mouseY, [0, 1], [15, -15]), springConfig);
    const rotateY = useSpring(useTransform(mouseX, [0, 1], [-15, 15]), springConfig);

    const handleMouseMove = (e: React.MouseEvent) => {
        if (!areaRef.current) return;
        const rect = areaRef.current.getBoundingClientRect();
        // Normalize 0 to 1
        const x = (e.clientX - rect.left) / Math.max(rect.width, 1);
        const y = (e.clientY - rect.top) / Math.max(rect.height, 1);
        mouseX.set(x);
        mouseY.set(y);
    };

    return (
        <section
            ref={areaRef}
            onMouseMove={handleMouseMove}
            onMouseLeave={() => { mouseX.set(0.5); mouseY.set(0.5); }}
            className="relative min-h-[95vh] flex items-center overflow-hidden bg-[#020202] pt-20 pb-20"
            id="hero"
            style={{ perspective: '2000px' }}
        >
            {/* ── Abyssal Ambient Glow (Tied to the Hologram core) ── */}
            <div className="absolute top-1/2 right-[5%] -translate-y-1/2 w-[900px] h-[900px] pointer-events-none opacity-40 mix-blend-screen scale-150 z-0">
                <div 
                    className="absolute inset-0 rounded-full"
                    style={{
                        background: 'radial-gradient(circle at 50% 50%, rgba(249,115,22,0.15) 0%, rgba(200,80,10,0.05) 30%, transparent 60%)'
                    }}
                />
            </div>

            <Container className="relative z-10 w-full h-full">
                <div className="grid lg:grid-cols-12 gap-12 items-center h-full">
                    
                    {/* ── LEFT COLUMN: God-Tier Extreme Typography ── */}
                    <motion.div 
                        style={{ y: yText, opacity: opacityText }}
                        className="col-span-12 lg:col-span-5 flex flex-col z-20 pt-10"
                    >
                        <motion.div
                            initial={{ opacity: 0, x: -20 }}
                            animate={{ opacity: 1, x: 0 }}
                            transition={{ duration: 0.8, ease: [0.16, 1, 0.3, 1] }}
                            className="w-fit"
                        >
                            <div className="inline-flex items-center gap-3 px-4 py-1.5 rounded-full border border-white/10 bg-white/[0.02] backdrop-blur-md">
                                <span className="relative flex h-2 w-2">
                                  <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-brand-400 opacity-80" />
                                  <span className="relative inline-flex rounded-full h-2 w-2 bg-brand-500 shadow-[0_0_10px_#f97316]" />
                                </span>
                                <span className="text-[10px] font-bold tracking-[0.25em] text-white/50 uppercase font-mono">
                                    {t('hero.badge')}
                                </span>
                            </div>
                        </motion.div>

                        <motion.h1
                            className="mt-8 text-6xl sm:text-7xl lg:text-[5.5rem] font-bold tracking-[-0.03em] leading-[0.95]"
                            initial={{ opacity: 0, y: 20 }}
                            animate={{ opacity: 1, y: 0 }}
                            transition={{ duration: 1, delay: 0.1, ease: [0.16, 1, 0.3, 1] }}
                        >
                            <span className="text-white">{t('hero.title1')}</span>
                            <br />
                            <span className="text-transparent bg-clip-text bg-gradient-to-r from-brand-400 via-accent-300 to-white pb-2 relative z-10 inline-block drop-shadow-[0_4px_16px_rgba(249,115,22,0.2)]">
                                {t('hero.title2')}
                            </span>
                        </motion.h1>

                        <motion.p
                            className="mt-8 text-lg sm:text-[1.35rem] text-[#8a8a93] leading-relaxed max-w-lg font-medium tracking-tight text-balance"
                            initial={{ opacity: 0, y: 20 }}
                            animate={{ opacity: 1, y: 0 }}
                            transition={{ duration: 1, delay: 0.2, ease: [0.16, 1, 0.3, 1] }}
                        >
                            <span className="opacity-90">{t('hero.subtitle')}</span>
                        </motion.p>

                        <motion.div
                            className="mt-14 flex flex-col sm:flex-row items-center gap-5 w-full sm:w-auto"
                            initial={{ opacity: 0, y: 20 }}
                            animate={{ opacity: 1, y: 0 }}
                            transition={{ duration: 1, delay: 0.3, ease: [0.16, 1, 0.3, 1] }}
                        >
                            <Link href="/contact" className="group relative w-full sm:w-auto flex items-center justify-center gap-3 px-8 py-4 bg-white text-black rounded-lg font-bold tracking-tight overflow-hidden hover:scale-105 active:scale-95 transition-all duration-300 shadow-[0_0_40px_rgba(255,255,255,0.15)] hover:shadow-[0_0_60px_rgba(255,255,255,0.25)]">
                                <span className="relative z-10">{t('hero.cta')}</span>
                                <HiOutlineArrowRight size={18} className="group-hover:translate-x-1 transition-transform relative z-10" />
                                {/* Internal glowing sheen swipe line */}
                                <div className="absolute inset-0 bg-gradient-to-r from-transparent via-black/10 to-transparent -translate-x-[150%] group-hover:translate-x-[150%] transition-transform duration-700" />
                            </Link>
                            
                            <Link
                                href="/docs/"
                                className="relative group w-full sm:w-auto flex items-center justify-center px-8 py-4 font-bold tracking-tight rounded-lg overflow-hidden transition-all duration-300"
                            >
                                {/* Static Border */}
                                <span className="absolute inset-0 border border-white/5 rounded-lg group-hover:border-transparent transition-colors duration-500" />
                                
                                {/* Electric Laser Spin (Appears on Hover) */}
                                <span className="absolute inset-[-1000%] animate-[spin_2s_linear_infinite] bg-[conic-gradient(from_90deg_at_50%_50%,transparent_0%,transparent_70%,#f97316_100%)] opacity-0 group-hover:opacity-100 transition-opacity duration-300" />
                                
                                {/* Inner Mask to create the border path */}
                                <span className="absolute inset-[1px] bg-[#020202] rounded-lg opacity-0 group-hover:opacity-100 transition-opacity duration-300" />
                                
                                {/* Inner Core Electric Glow */}
                                <span className="absolute inset-0 rounded-lg shadow-[inset_0_0_0px_rgba(249,115,22,0)] group-hover:shadow-[inset_0_0_20px_rgba(249,115,22,0.15)] transition-shadow duration-500" />

                                <span className="relative z-10 text-white/50 group-hover:text-white transition-colors duration-300">
                                    {t('hero.ctaSecondary')}
                                </span>
                            </Link>
                        </motion.div>

                        <motion.div
                            className="mt-16 flex flex-wrap items-center gap-4 pt-8 border-t border-white/[0.05]"
                            initial={{ opacity: 0 }}
                            animate={{ opacity: 1 }}
                            transition={{ duration: 1, delay: 0.5 }}
                        >
                            <div className="flex items-center gap-2 px-3 py-1 rounded bg-white/[0.02] border border-white/[0.05] text-xs font-medium text-gray-400">
                                <span className="w-1.5 h-1.5 rounded-full bg-brand-500 shadow-[0_0_8px_#f97316]"></span>
                                {t('hero.trust.metadata')}
                            </div>
                            <div className="flex items-center gap-2 px-3 py-1 rounded bg-white/[0.02] border border-white/[0.05] text-xs font-medium text-gray-400">
                                <HiOutlineShieldCheck size={14} className="text-brand-500" />
                                {t('hero.trust.selfhosted')}
                            </div>
                            <div className="flex items-center gap-2 px-3 py-1 rounded bg-white/[0.02] border border-white/[0.05] text-xs font-medium text-gray-400">
                                <HiOutlineDatabase size={14} className="text-brand-500" />
                                {t('hero.trust.appendonly')}
                            </div>
                        </motion.div>
                    </motion.div>
                    {/* ── RIGHT COLUMN: The Spatial Hologram Engine ── */}
                    <div className="col-span-12 lg:col-span-7 relative w-full min-h-[500px] lg:h-[800px] flex items-center justify-center lg:justify-end mt-12 lg:mt-0 z-10 pointer-events-none">
                        {/* Dedicated positioning wrapper to bypass Framer Motion overriding CSS transforms */}
                        <div className="w-[850px] h-[850px] absolute right-0 lg:right-[-250px] top-1/2 -translate-y-1/2">
                            <motion.div
                                style={{ 
                                    rotateX,
                                    rotateY,
                                    transformStyle: "preserve-3d" 
                                }}
                                className="w-full h-full pointer-events-none"
                                initial={{ opacity: 0, scale: 0.8, rotateZ: -10 }}
                                animate={{ opacity: 1, scale: 1, rotateZ: 0 }}
                                transition={{ duration: 1.5, ease: [0.16, 1, 0.3, 1] }}
                            >
                                <HolographicEngine />
                            </motion.div>
                        </div>
                    </div>

                </div>
            </Container>
        </section>
    );
}

/* ═══════════════════════════════════════════════════════
   THE HOLOGRAM CORE (Pure Math, SVG, Motion Divs)
   Orchestrating the "Evidence Pipeline" sequence
═══════════════════════════════════════════════════════ */

function HolographicEngine() {
    // Pipeline sequence duration: 12 seconds total loop
    const DURATION = 12;

    return (
        <div className="relative w-full h-full flex items-center justify-center" style={{ transformStyle: 'preserve-3d' }}>
            
            {/* 1. Core Geometric Vault (Center) */}
            <motion.div 
                className="absolute w-32 h-32 rounded-full border-[2px] border-brand-500/80 shadow-[0_0_120px_rgba(249,115,22,0.8),inset_0_0_40px_rgba(249,115,22,0.6)] flex items-center justify-center backdrop-blur-md bg-black/40 z-50 overflow-hidden"
                style={{ transform: 'translateZ(100px)' }}
                animate={{ rotate: 360 }}
                transition={{ duration: 60, repeat: Infinity, ease: 'linear' }}
            >
                <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_center,rgba(249,115,22,0.4)_0%,transparent_70%)]" />
                <motion.div 
                    className="w-full h-full opacity-30"
                    animate={{ rotate: -720 }}
                    transition={{ duration: 30, repeat: Infinity, ease: 'linear' }}
                >
                    <svg viewBox="0 0 100 100" className="w-full h-full">
                        <path d="M 50 10 L 90 50 L 50 90 L 10 50 Z" stroke="#fff" strokeWidth="1" fill="none" />
                        <circle cx="50" cy="50" r="20" stroke="#fff" strokeWidth="1" fill="none" strokeDasharray="3 3"/>
                    </svg>
                </motion.div>
                <HiOutlineShieldCheck size={32} className="text-white absolute z-10" />
            </motion.div>

             {/* 2. Middle Ring: "Policy Evaluation Grid" (Visual ONLY) */}
            <motion.div 
                className="absolute w-[450px] h-[450px] rounded-full"
                style={{ transform: 'translateZ(30px)' }}
                animate={{ rotate: -360 }}
                transition={{ duration: 80, repeat: Infinity, ease: 'linear' }}
            >
                <svg className="absolute inset-0 w-full h-full" viewBox="0 0 450 450" style={{ filter: 'drop-shadow(0 0 15px rgba(255,255,255,0.15))' }}>
                     <circle cx="225" cy="225" r="224" stroke="rgba(255,255,255,0.2)" strokeWidth="1" fill="none" strokeDasharray="1 8" />
                     <circle cx="225" cy="225" r="224" stroke="rgba(249,115,22,0.8)" strokeWidth="3" fill="none" strokeDasharray="40 120" strokeLinecap="round" />
                     <circle cx="225" cy="225" r="210" stroke="rgba(255,255,255,0.05)" strokeWidth="10" fill="none" />
                 </svg>
            </motion.div>

            {/* 3. Outer Ring: "Data Ingestion Stream (Commits/CI)" */}
            <div className="absolute w-[700px] h-[700px] rounded-full" style={{ transform: 'translateZ(-50px)' }}>
                {/* Visual Ring Background */}
                <motion.div className="absolute inset-0 pointer-events-none" animate={{ rotate: 360 }} transition={{ duration: 120, repeat: Infinity, ease: 'linear' }}>
                    <svg className="w-full h-full opacity-60" viewBox="0 0 700 700">
                         <circle cx="350" cy="350" r="349" stroke="rgba(255,255,255,0.08)" strokeWidth="2" fill="none" strokeDasharray="6 12" />
                         <circle cx="350" cy="350" r="349" stroke="rgba(245,158,11,0.6)" strokeWidth="4" fill="none" strokeDasharray="200 2000" strokeLinecap="round" style={{ filter: 'drop-shadow(0 0 20px #f59e0b)' }} />
                    </svg>
                </motion.div>

                {/* Satellite Event Train */}
                {[
                    { label: "Commit [a3f8c]", color: "border-brand-500/40 text-brand-400", pulse: true },
                    { label: "Policy: ACTIVE", color: "border-white/20 text-gray-300", pulse: false },
                    { label: "Signature Check", color: "border-white/20 text-gray-300", pulse: false },
                    { label: "CI Workflow: OK", color: "border-white/20 text-gray-300", pulse: false },
                    { label: "Jira: Linked", color: "border-accent-500/40 text-accent-400", pulse: false },
                ].map((event, index) => {
                    const startDelay = index * 2.5; // Stagger spawns
                    return (
                        <motion.div 
                            key={`train-${index}`}
                            className="absolute inset-0 z-20 pointer-events-none"
                            initial={{ rotate: -90 }} // Start rotation from the top
                            animate={{ rotate: 270 }} // Rotate one full circle
                            transition={{ duration: 25, repeat: Infinity, ease: 'linear', delay: startDelay }}
                        >
                            {/* Anchor point at 12 o'clock */}
                            <div className="absolute top-0 left-1/2 -translate-x-1/2 -translate-y-1/2">
                                {/* The Pill (Fade in and Counter-rotate) */}
                                <motion.div
                                    initial={{ opacity: 0, scale: 0.5, rotate: 90 }}
                                    animate={{ opacity: 1, scale: 1, rotate: -270 }}
                                    transition={{ 
                                        opacity: { duration: 0.8, delay: startDelay },
                                        scale: { duration: 0.8, delay: startDelay, type: 'spring' },
                                        rotate: { duration: 25, repeat: Infinity, ease: 'linear', delay: startDelay }
                                    }}
                                    style={{ transformOrigin: 'center' }}
                                >
                                    <GlassPill label={event.label} color={event.color} pulse={event.pulse} />
                                </motion.div>
                            </div>
                        </motion.div>
                    );
                })}
            </div>

            {/* Connecting Laser Beams (Flashing at 7.5s) */}
            <motion.div 
                className="absolute inset-0 pointer-events-none"
                animate={{ 
                    opacity: [0, 0, 0, 1, 0, 0],
                    scale: [0.8, 0.8, 0.8, 1.05, 0.8, 0.8]
                }}
                transition={{ 
                    duration: DURATION,
                    repeat: Infinity,
                    times: [0, 0.60, 0.62, 0.65, 0.68, 1], // Flash occurs between 7.2s and 8.1s
                    ease: "easeInOut"
                }}
            >
                <svg className="w-full h-full" viewBox="0 0 800 800">
                    <line x1="400" y1="400" x2="250" y2="150" stroke="#f97316" strokeWidth="2" strokeDasharray="5 5" style={{ filter: 'drop-shadow(0 0 10px #f97316)' }} />
                    <line x1="400" y1="400" x2="600" y2="250" stroke="#f97316" strokeWidth="2" strokeDasharray="5 5" style={{ filter: 'drop-shadow(0 0 10px #f97316)' }} />
                    <line x1="400" y1="400" x2="650" y2="650" stroke="#f97316" strokeWidth="3" style={{ filter: 'drop-shadow(0 0 15px #f97316)' }} />
                    <line x1="400" y1="400" x2="150" y2="500" stroke="#f97316" strokeWidth="2" strokeDasharray="5 5" style={{ filter: 'drop-shadow(0 0 10px #f97316)' }} />
                    <circle cx="400" cy="400" r="150" stroke="rgba(249,115,22,0.8)" strokeWidth="4" fill="none" style={{ filter: 'drop-shadow(0 0 20px #f97316)' }} />
                </svg>
            </motion.div>

            {/* 4. Connectivity Beams & Grid Geometry */}
            <div className="absolute inset-0 z-0 flex items-center justify-center opacity-30 pointer-events-none" style={{ transform: 'translateZ(-100px)' }}>
                <svg className="w-full h-full" viewBox="0 0 800 800">
                    <line x1="400" y1="0" x2="400" y2="800" stroke="rgba(255,255,255,0.15)" strokeWidth="1" strokeDasharray="4 4" />
                    <line x1="0" y1="400" x2="800" y2="400" stroke="rgba(255,255,255,0.15)" strokeWidth="1" strokeDasharray="4 4" />
                    <circle cx="400" cy="400" r="300" stroke="rgba(255,255,255,0.05)" strokeWidth="1" fill="none" />
                </svg>
            </div>

            {/* 5. Center Laser connection to left text block (simulated via diagonal line extending off left edge) */}
            <div className="absolute top-1/2 left-0 w-1/2 h-[1px] bg-gradient-to-r from-transparent via-brand-500 to-transparent -translate-y-1/2 -translate-x-full opacity-40 shadow-[0_0_10px_#f97316] z-10 pointer-events-none" />
        </div>
    );
}

function GlassPill({ label, color, pulse = false }: { label: string, color: string, pulse?: boolean }) {
    return (
        <div className={`flex items-center gap-3 px-4 py-2 rounded-xl backdrop-blur-xl border bg-black/60 shadow-2xl whitespace-nowrap ${color}`}>
            <div className="relative flex h-2 w-2 items-center justify-center">
                {pulse && <span className="animate-ping absolute inline-flex h-4 w-4 rounded-full bg-brand-400 opacity-60" />}
                <span className="relative inline-flex rounded-full h-2 w-2 shadow-[0_0_8px_currentColor]" style={{ background: 'currentColor' }} />
            </div>
            <span className="text-xs font-mono font-black uppercase tracking-widest">{label}</span>
        </div>
    );
}
