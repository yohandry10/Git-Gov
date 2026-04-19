'use client';

import React from 'react';
import { motion } from 'framer-motion';
import {
    HiOutlineServer,
    HiOutlineShieldCheck,
    HiOutlineDesktopComputer,
    HiOutlineDatabase,
    HiOutlineDocumentReport,
    HiOutlineCheckCircle,
    HiOutlineRefresh
} from 'react-icons/hi';
import { SiGithub, SiPostgresql } from 'react-icons/si';

/* ═══════════════════════════════════════════════════════
   THE ARCHITECTURAL BENTO (Massive Refactoring)
   Injects all the deeply technical architecture details provided:
   - E2E Correlation between Client Intent and GitHub Webhook.
   - Confidence Scoring over binary violations.
   - Desktop: JSONL outbox, OAuth, SQLite, offline retries.
   - Server: FOR UPDATE SKIP LOCKED, Dead-letter queue, Idempotency, RLS.
   - Audit: Append-only, Export SHA-256, bypassing GitHub log limits.
   ═══════════════════════════════════════════════════════ */

export function ProblemSolutionBento() {
    return (
        <div className="w-full mt-16 max-w-7xl mx-auto space-y-6 relative z-10">
            {/* The Ultimate Bento Grid */}
            <div className="grid grid-cols-1 md:grid-cols-12 gap-6">

                {/* ── BENTO 1: E2E Correlation & Confidence Scoring (Col span 8) ── */}
                <motion.div
                    initial={{ opacity: 0, y: 30 }}
                    whileInView={{ opacity: 1, y: 0 }}
                    viewport={{ once: true, margin: "-100px" }}
                    transition={{ duration: 0.8 }}
                    className="md:col-span-8 relative rounded-3xl border border-white/5 bg-[#080808] overflow-hidden p-8 flex flex-col justify-between min-h-[380px] shadow-2xl group"
                >
                    <div className="absolute top-0 left-0 w-full h-1 bg-gradient-to-r from-brand-500/50 to-transparent" />

                    <div className="relative z-10 flex flex-col md:flex-row gap-8 items-start justify-between h-full">
                        {/* Text Content */}
                        <div className="max-w-md">
                            <div className="flex items-center gap-3 mb-4">
                                <div className="w-8 h-8 rounded-full bg-brand-500/10 flex items-center justify-center border border-brand-500/20">
                                    <HiOutlineShieldCheck className="text-brand-400" size={18} />
                                </div>
                                <h3 className="text-xl font-bold tracking-tight text-white">Correlación E2E y Detección de Drift</h3>
                            </div>
                            <p className="text-white/50 text-sm leading-relaxed mb-4">
                                La propuesta de GitGov no es "hacer Git bonito". El enforcement real (Branch Protection, Rulesets) vive en GitHub. Nosotros orquestamos esas reglas agregando evidencia inmutable: calculando el <strong className="text-white/80 font-medium">Confidence Scoring</strong> cruzando la intención local del desarrollador contra el webhook final recibido.
                            </p>
                            <p className="text-white/40 text-[11px] leading-relaxed">
                                Abandona el lenguaje binario de fraude. Evaluamos "rutas no autorizadas" y "telemetría faltante" con scores paramétricos (High/Medium/Low) para anular los falsos positivos reputacionales.
                            </p>
                            {/* Filler elements to pack the space */}
                            <div className="mt-6 space-y-2 font-mono text-[10px] text-white/50">
                                <div className="flex items-center gap-2 bg-white/5 p-2 rounded border border-white/5">
                                    <HiOutlineCheckCircle className="text-brand-500" size={14} /> Detección de bypass y telemetría faltante
                                </div>
                                <div className="flex items-center gap-2 bg-white/5 p-2 rounded border border-white/5">
                                    <HiOutlineCheckCircle className="text-brand-500" size={14} /> Versionado trazable de políticas (gitgov.toml)
                                </div>
                            </div>
                        </div>

                        {/* Visual: Correlation Engine */}
                        <div className="w-full md:w-[320px] bg-[#0d0d0d] border border-white/10 rounded-xl p-4 flex-shrink-0 relative overflow-hidden flex flex-col">
                            <div className="text-[10px] uppercase font-bold tracking-widest text-brand-500 mb-3 border-b border-white/5 pb-2">
                                Correlation Engine
                            </div>
                            <div className="space-y-3 font-mono text-[10px]">
                                <div className="flex justify-between items-center bg-[#151515] border border-white/5 p-2 rounded">
                                    <span className="text-white/40">1. Client Intent</span>
                                    <span className="text-white/80">commit: a3f8c</span>
                                </div>
                                <div className="flex justify-between items-center bg-[#151515] border border-white/5 p-2 rounded">
                                    <span className="text-white/40">2. GitHub Webhook</span>
                                    <span className="text-white/80">push: refs/heads/main</span>
                                </div>
                                <div className="flex justify-between items-center p-2 border-l-2 border-brand-500 bg-brand-500/5">
                                    <span className="text-white/60">Drift Status</span>
                                    <span className="text-brand-400 font-bold uppercase tracking-wider">MATCH (High Confidence)</span>
                                </div>
                            </div>

                            {/* Filler: Evaluación de Señales */}
                            <div className="mt-4 pt-4 border-t border-white/5 flex-1 flex flex-col justify-center">
                                <span className="text-[9px] uppercase tracking-widest text-white/30 block mb-2">Evaluación de Señales (Scoring)</span>
                                <div className="space-y-1.5 text-[9px] font-mono text-white/50">
                                    <div className="flex justify-between items-center"><span className="text-white/40">Bypass Route:</span> <span className="text-white/80 border border-white/10 px-1 rounded bg-[#111]">False</span></div>
                                    <div className="flex justify-between items-center"><span className="text-white/40">Untrusted Path:</span> <span className="text-white/80 border border-white/10 px-1 rounded bg-[#111]">False</span></div>
                                    <div className="flex justify-between items-center"><span className="text-white/40">Missing Telemetry:</span> <span className="text-brand-500 font-bold bg-brand-500/10 px-1.5 rounded border border-brand-500/20">0%</span></div>
                                </div>
                            </div>
                            
                            {/* Filler: Additional E2E Badges */}
                            <div className="mt-4 pt-3 border-t border-white/5 flex gap-2">
                                <span className="bg-white/5 text-white/50 text-[8px] px-2 py-1.5 rounded uppercase tracking-widest w-full text-center border border-white/5 shadow-sm">Eventos E2E</span>
                                <span className="bg-white/5 text-white/50 text-[8px] px-2 py-1.5 rounded uppercase tracking-widest w-full text-center border border-white/5 shadow-sm">Auditoría Pública</span>
                            </div>

                            {/* Animated Background Scan */}
                            <div className="absolute inset-0 bg-gradient-to-b from-transparent via-brand-500/5 to-transparent -translate-y-full group-hover:translate-y-full transition-transform duration-[1500ms] pointer-events-none" />
                        </div>
                    </div>
                </motion.div>

                {/* ── BENTO 2: Auditoría Append-Only (Col span 4) ── */}
                <motion.div
                    initial={{ opacity: 0, y: 30 }}
                    whileInView={{ opacity: 1, y: 0 }}
                    viewport={{ once: true, margin: "-100px" }}
                    transition={{ duration: 0.8, delay: 0.1 }}
                    className="md:col-span-4 relative rounded-3xl border border-white/5 bg-[#080808] overflow-hidden p-8 flex flex-col min-h-[380px] shadow-2xl group"
                >
                    <div className="relative z-10 mb-6">
                        <div className="flex items-center gap-3 mb-4">
                            <div className="w-8 h-8 rounded-full bg-white/5 flex items-center justify-center">
                                <HiOutlineDatabase className="text-white/40" size={18} />
                            </div>
                            <h3 className="text-lg font-bold tracking-tight text-white">Auditoría Inmutable</h3>
                        </div>
                        <p className="text-white/50 text-sm leading-relaxed">
                            Retenemos la evidencia de gobernanza mucho más allá del horizonte temporal limitado del audit log de GitHub. Operamos sobre tablas estrictamente <strong className="text-white/80 font-medium">append-only</strong> en servidor.
                        </p>
                    </div>

                    <div className="mt-auto bg-[#0d0d0d] border border-white/10 rounded-xl p-4">
                        <span className="text-[9px] uppercase tracking-widest text-white/30 block mb-2">Export Data (SOC 2)</span>
                        <div className="space-y-2">
                            <div className="flex items-center justify-between text-[11px] font-mono text-white/60">
                                <span>PDF / Excel / JSON</span>
                                <HiOutlineDocumentReport />
                            </div>
                            <div className="text-[9px] font-mono text-brand-500/80 bg-brand-500/10 p-1.5 rounded truncate">
                                SHA256: 8f4e2...d91c
                            </div>
                        </div>
                        {/* Filler elements to pack the space */}
                        <div className="mt-4 border-t border-white/10 pt-4 font-mono text-[10px] text-white/40 space-y-1">
                            <div className="flex justify-between"><span>Retención Config:</span> <span className="text-white/80">+5 Años Mínimo</span></div>
                            <div className="flex justify-between"><span>Truncamientos:</span> <span className="text-brand-400 font-bold">Bloqueados</span></div>
                        </div>
                    </div>
                </motion.div>

                {/* ── BENTO 3: Desktop Agent Resiliencia (Col span 4) ── */}
                <motion.div
                    initial={{ opacity: 0, y: 30 }}
                    whileInView={{ opacity: 1, y: 0 }}
                    viewport={{ once: true, margin: "-100px" }}
                    transition={{ duration: 0.8, delay: 0.2 }}
                    className="md:col-span-4 relative rounded-3xl border border-white/5 bg-[#080808] overflow-hidden p-8 flex flex-col justify-between min-h-[380px] shadow-2xl group"
                >
                    <div className="relative z-10">
                        <div className="flex items-center gap-3 mb-4">
                            <div className="w-8 h-8 rounded-full bg-white/5 flex items-center justify-center">
                                <HiOutlineDesktopComputer className="text-white/40" size={18} />
                            </div>
                            <h3 className="text-lg font-bold tracking-tight text-white">Desktop Agent Offline</h3>
                        </div>
                        <p className="text-white/50 text-sm leading-relaxed mb-6">
                            Guía al desarrollador <strong className="text-brand-300 font-medium">sin tocar el código fuente</strong>. Auditoría local SQLite, validaciones de nomenclatura y autenticación GitHub OAuth.
                        </p>
                    </div>

                    <div className="flex flex-col gap-3 relative z-10">
                        {/* Terminal Window */}
                        <div className="bg-[#0a0a0a] border border-white/10 rounded-xl font-mono text-[10px] overflow-hidden shadow-[inset_0_0_20px_rgba(0,0,0,0.5)]">
                            <div className="bg-[#111] p-2 flex justify-between items-center border-b border-white/5">
                                <span className="text-white/50 font-bold uppercase tracking-widest text-[9px]">outbox.jsonl</span>
                                <span className="bg-brand-500/10 border border-brand-500/20 text-brand-400 px-1 py-0.5 rounded text-[8px] uppercase tracking-widest">limit: 500</span>
                            </div>
                            <div className="p-4 text-white/40 space-y-2 flex flex-col justify-center">
                                <p>{"{"}"event": "stage_files", "count": 500{"}"}</p>
                                <p className="text-brand-400/80 border-l border-brand-400/20 pl-2">Network Error. Queued.</p>
                                <p className="text-white/80 pt-1 flex items-center gap-1.5"><HiOutlineRefresh className="text-brand-500" /> Retry policy (Exponential)</p>
                            </div>
                        </div>

                        {/* Badges */}
                        <div className="flex gap-2 text-[9px] uppercase tracking-widest text-white/50 font-mono text-center">
                            <div className="bg-[#111] border border-white/5 rounded-lg p-2.5 w-full flex items-baseline justify-center gap-1.5 hover:border-brand-500/30 transition-colors">
                                Local SQLite
                            </div>
                            <div className="bg-[#111] border border-white/5 rounded-lg p-2.5 w-full flex items-baseline justify-center gap-1.5 hover:border-brand-500/30 transition-colors">
                                Ignore Rules
                            </div>
                        </div>
                    </div>
                </motion.div>

                {/* ── BENTO 4: Backend de Producción Madura (Col span 8) ── */}
                <motion.div
                    initial={{ opacity: 0, y: 30 }}
                    whileInView={{ opacity: 1, y: 0 }}
                    viewport={{ once: true, margin: "-100px" }}
                    transition={{ duration: 0.8, delay: 0.3 }}
                    className="md:col-span-8 relative rounded-3xl border border-white/5 bg-[#080808] overflow-hidden p-8 flex flex-col md:flex-row gap-6 items-center shadow-2xl group"
                >
                    <div className="flex-1">
                        <div className="flex items-center gap-3 mb-4">
                            <div className="w-8 h-8 rounded-full bg-white/5 flex items-center justify-center">
                                <HiOutlineServer className="text-white/40" size={18} />
                            </div>
                            <h3 className="text-lg font-bold tracking-tight text-white">Backend Operacional de Grado Producción</h3>
                        </div>
                        <p className="text-white/50 text-sm leading-relaxed mb-3">
                            La arquitectura del servidor no es un mock. Desplegamos ingesta batch asíncrona de eventos con idempotencia, versionado automático de políticas y multi-tenancy absoluto con Row-Level Security (RLS) sobre Supabase.
                        </p>
                        <ul className="grid grid-cols-2 gap-2 text-[11px] font-mono text-white/40 mt-4">
                            <li className="flex items-center gap-1.5"><HiOutlineCheckCircle className="text-brand-500" /> PostgreSQL <code>SKIP LOCKED</code></li>
                            <li className="flex items-center gap-1.5"><HiOutlineCheckCircle className="text-brand-500" /> Dead-letter Queues</li>
                            <li className="flex items-center gap-1.5"><HiOutlineCheckCircle className="text-brand-500" /> Backoff Exponencial</li>
                            <li className="flex items-center gap-1.5"><HiOutlineCheckCircle className="text-brand-500" /> Stale Worker Reset</li>
                        </ul>
                        {/* Filler text to pack the box */}
                        <div className="mt-4 bg-white/5 border border-white/10 p-3 rounded-lg text-[10px] text-white/50">
                            <strong className="text-white/80 block mb-1">Ingesta Batch Idempotente</strong>
                            La deduplicación ocurre por combinación de organización y tipo de evento, asegurando consistencia transaccional. Respaldado por aislamiento multi-tenant estricto mediante Supabase Row-Level Security (RLS).
                        </div>
                    </div>

                    <div className="w-full md:w-[220px] flex-shrink-0 flex items-center justify-center p-6 bg-[#0a0a0a] border border-white/10 rounded-2xl relative">
                        {/* Abstract Representation of RLS / PostgreSQL */}
                        <SiPostgresql className="text-white/10 text-8xl absolute" />
                        <div className="relative z-10 flex flex-col items-center gap-2">
                            <div className="w-full h-1.5 bg-brand-500/20 rounded shadow-[0_0_10px_rgba(249,115,22,0.3)]"></div>
                            <div className="w-full h-1.5 bg-brand-500/40 rounded shadow-[0_0_10px_rgba(249,115,22,0.3)]"></div>
                            <div className="w-full h-1.5 bg-brand-500/80 rounded shadow-[0_0_10px_rgba(249,115,22,0.3)] animate-pulse"></div>
                            <span className="text-[9px] uppercase tracking-widest text-brand-500 font-bold mt-2">Job Queue (Active)</span>
                        </div>
                    </div>
                </motion.div>

            </div>
        </div>
    );
}
