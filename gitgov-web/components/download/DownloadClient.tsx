'use client';

import React from 'react';
import { Container } from '@/components/layout';
import { SectionHeader } from '@/components/marketing';
import { SectionReveal } from '@/components/ui';
import { siteConfig } from '@/lib/config/site';
import { FaWindows } from 'react-icons/fa';
import {
    HiOutlineDownload,
    HiOutlineShieldCheck,
    HiOutlineClipboard,
    HiOutlineCheck,
    HiOutlineInformationCircle,
    HiOutlineLightningBolt,
    HiOutlineClipboardCheck,
} from 'react-icons/hi';
import { useTranslation } from '@/lib/i18n';
import type { ReleaseMetadata } from '@/lib/release';

interface DownloadClientProps {
    release: ReleaseMetadata;
}

export function DownloadClient({ release }: DownloadClientProps) {
    const { t } = useTranslation();
    const [copied, setCopied] = React.useState(false);
    const desktopCoverage = [
        {
            title: t('download.side.h1title') as string,
            description: t('download.side.h1desc') as string,
            icon: <HiOutlineClipboardCheck size={16} />,
        },
        {
            title: t('download.side.h2title') as string,
            description: t('download.side.h2desc') as string,
            icon: <HiOutlineLightningBolt size={16} />,
        },
        {
            title: t('download.side.h3title') as string,
            description: t('download.side.h3desc') as string,
            icon: <HiOutlineShieldCheck size={16} />,
        },
        {
            title: t('download.side.h4title') as string,
            description: t('download.side.h4desc') as string,
            icon: <HiOutlineInformationCircle size={16} />,
        },
    ];

    const exeFileName =
        release.downloadUrl.split('/').pop() ?? siteConfig.downloadFileName;

    function handleCopyChecksum() {
        navigator.clipboard.writeText(release.checksum).then(() => {
            setCopied(true);
            setTimeout(() => setCopied(false), 2000);
        });
    }
    const hasPendingChecksum = release.checksum.includes('pending');

    return (
        <>
            {/* Hero */}
            <section className="pt-32 md:pt-40 pb-8 relative overflow-hidden text-center">
                <div className="absolute inset-0">
                    <div
                        className="absolute inset-0 opacity-[0.03]"
                        style={{
                            backgroundImage: `linear-gradient(rgba(249,115,22,0.2) 1px, transparent 1px), linear-gradient(90deg, rgba(249,115,22,0.2) 1px, transparent 1px)`,
                            backgroundSize: '40px 40px',
                        }}
                    />
                </div>
                <Container className="relative z-10">
                    <SectionReveal>
                        <div className="inline-flex items-center px-4 py-1.5 rounded-full text-xs font-bold uppercase tracking-widest bg-brand-500/10 text-brand-500 border border-brand-500/20 mb-8 shadow-inner">
                            {t('download.badge') as string}
                        </div>
                        <h1 className="text-5xl md:text-7xl font-bold tracking-tighter text-white mb-6">
                            {t('download.title') as string} <span className="text-brand-500">{t('download.titleAccent') as string}</span>
                        </h1>
                        <p className="text-lg md:text-xl text-gray-400 max-w-2xl mx-auto leading-relaxed">
                            {t('download.description') as string}
                        </p>
                    </SectionReveal>
                </Container>
            </section>

            {/* Centered Minimalist Layout */}
            <section className="pb-32 relative z-10">
                <Container>
                    <SectionReveal>
                        <div className="max-w-3xl mx-auto">
                            {/* Main Download Card */}
                            <div className="rounded-[32px] p-px bg-gradient-to-b from-white/10 to-white/[0.02] shadow-[0_0_80px_rgba(249,115,22,0.1)] relative">
                                <div className="absolute top-0 left-1/2 -translate-x-1/2 w-[60%] h-px bg-gradient-to-r from-transparent via-brand-500/50 to-transparent" />

                                <div className="rounded-[31px] bg-[#050505] p-8 md:p-14 flex flex-col items-center text-center relative overflow-hidden">
                                    <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[80%] h-[80%] bg-brand-500/10 blur-[100px] pointer-events-none rounded-full" />

                                    <div className="w-24 h-24 rounded-3xl bg-gradient-to-b from-[#1a1a1a] to-[#0a0a0a] border border-white/10 flex items-center justify-center text-white mb-8 shadow-2xl relative z-10">
                                        <FaWindows size={42} className="drop-shadow-[0_0_15px_rgba(255,255,255,0.3)]" />
                                    </div>

                                    <div className="inline-flex items-center px-4 py-1.5 rounded-full text-xs font-mono font-semibold bg-white/[0.03] text-gray-400 border border-white/10 mb-12 relative z-10 shadow-inner">
                                        Windows x64 • Version {release.version}
                                    </div>

                                    {/* Download Buttons */}
                                    <div className="w-full max-w-sm space-y-4 relative z-10 mb-12">
                                        {release.available ? (
                                            <>
                                                <a
                                                    href={release.downloadUrl}
                                                    className="group relative flex items-center justify-center gap-3 w-full py-5 px-8 rounded-2xl font-bold text-white overflow-hidden shadow-[0_0_30px_rgba(249,115,22,0.3)] hover:shadow-[0_0_50px_rgba(249,115,22,0.5)] transition-all duration-300"
                                                >
                                                    <div className="absolute inset-0 bg-brand-500 transition-transform duration-300 group-hover:scale-[1.02]" />
                                                    <div className="absolute inset-0 bg-gradient-to-b from-white/20 to-transparent opacity-50" />
                                                    <HiOutlineDownload size={24} className="relative z-10" />
                                                    <span className="relative z-10 text-xl tracking-wide">{t('download.button') as string}</span>
                                                </a>
                                                {release.msiUrl && (
                                                    <a
                                                        href={release.msiUrl}
                                                        className="flex items-center justify-center gap-2 w-full py-3.5 px-6 rounded-xl text-sm font-semibold bg-[#111] text-gray-300 border border-white/5 hover:bg-[#1a1a1a] hover:text-white transition-all duration-300"
                                                    >
                                                        <HiOutlineDownload size={18} />
                                                        {t('download.buttonMsi') as string}
                                                    </a>
                                                )}
                                            </>
                                        ) : (
                                            <div className="flex items-center justify-center gap-3 w-full py-5 px-8 rounded-2xl font-bold bg-[#111] text-gray-500 border border-white/5 cursor-not-allowed">
                                                <HiOutlineDownload size={24} />
                                                <span className="text-xl tracking-wide">{t('download.button') as string}</span>
                                            </div>
                                        )}
                                    </div>

                                    {/* Metadata Grid */}
                                    <div className="w-full grid grid-cols-1 sm:grid-cols-2 gap-6 text-left border-t border-white/5 pt-10 relative z-10">
                                        <div>
                                            <div className="text-[10px] uppercase tracking-widest text-gray-500 font-bold mb-2">{t('download.file') as string}</div>
                                            <div className="text-sm font-mono text-gray-300 truncate pr-4">{exeFileName}</div>
                                        </div>
                                        <div>
                                            <div className="text-[10px] uppercase tracking-widest text-gray-500 font-bold mb-2">{t('download.checksum') as string}</div>
                                            <div className="flex items-center gap-2 group cursor-pointer" onClick={handleCopyChecksum}>
                                                <div className="text-sm font-mono text-gray-300 truncate max-w-[120px] sm:max-w-[160px] group-hover:text-white transition-colors">{release.checksum}</div>
                                                {copied ? <HiOutlineCheck size={16} className="text-brand-500 shrink-0" /> : <HiOutlineClipboard size={16} className="text-gray-500 group-hover:text-white transition-colors shrink-0" />}
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>

                            {/* Requirements snippet */}
                            <div className="mt-10 flex flex-wrap items-center justify-center gap-x-6 gap-y-3 text-xs font-mono text-gray-500 uppercase tracking-widest">
                                <span className="flex items-center gap-2"><div className="w-1.5 h-1.5 rounded-full bg-brand-500/50" /> Windows 10/11</span>
                                <span className="hidden sm:block text-white/10">•</span>
                                <span>x64 Architecture</span>
                                <span className="hidden sm:block text-white/10">•</span>
                                <span>~15 MB</span>
                                <span className="hidden sm:block text-white/10">•</span>
                                <span>No dependencies</span>
                            </div>
                        </div>

                        {/* Minimal Features 3-Column */}
                        <div className="max-w-5xl mx-auto mt-28 grid md:grid-cols-3 gap-10 md:gap-16 pt-16 border-t border-white/5 relative">
                            <div className="absolute top-0 left-1/2 -translate-x-1/2 w-1/4 h-px bg-gradient-to-r from-transparent via-brand-500/30 to-transparent" />

                            <div className="flex flex-col items-center text-center">
                                <div className="w-14 h-14 rounded-2xl bg-[#050505] border border-white/10 flex items-center justify-center text-brand-500 mb-6 shadow-inner">
                                    <HiOutlineShieldCheck size={24} />
                                </div>
                                <h4 className="text-xl font-bold text-white mb-3">{t('download.value.security.title') as string}</h4>
                                <p className="text-gray-400 leading-relaxed">{t('download.value.security.desc') as string}</p>
                            </div>

                            <div className="flex flex-col items-center text-center">
                                <div className="w-14 h-14 rounded-2xl bg-[#050505] border border-white/10 flex items-center justify-center text-brand-500 mb-6 shadow-inner">
                                    <HiOutlineLightningBolt size={24} />
                                </div>
                                <h4 className="text-xl font-bold text-white mb-3">{t('download.value.zeroOverhead.title') as string}</h4>
                                <p className="text-gray-400 leading-relaxed">{t('download.value.zeroOverhead.desc') as string}</p>
                            </div>

                            <div className="flex flex-col items-center text-center">
                                <div className="w-14 h-14 rounded-2xl bg-[#050505] border border-white/10 flex items-center justify-center text-brand-500 mb-6 shadow-inner">
                                    <HiOutlineClipboardCheck size={24} />
                                </div>
                                <h4 className="text-xl font-bold text-white mb-3">{t('download.value.offline.title') as string}</h4>
                                <p className="text-gray-400 leading-relaxed">{t('download.value.offline.desc') as string}</p>
                            </div>
                        </div>

                        {/* Terminal Installation steps */}
                        <div className="max-w-4xl mx-auto mt-32">
                            <div className="flex items-center justify-between mb-8">
                                <h4 className="text-2xl font-bold text-white tracking-tight">{t('download.installNotes') as string}</h4>
                                {!hasPendingChecksum && (
                                    <div className="text-xs font-mono font-semibold text-brand-500 flex items-center gap-2 bg-brand-500/10 px-3 py-1.5 rounded-full border border-brand-500/20">
                                        <HiOutlineShieldCheck size={16} /> SHA256 Verified
                                    </div>
                                )}
                            </div>

                            <div className="rounded-[24px] border border-white/10 bg-[#020202] shadow-2xl overflow-hidden">
                                <div className="h-12 bg-[#0a0a0a] border-b border-white/5 flex items-center px-6 gap-2">
                                    <div className="w-3 h-3 rounded-full bg-[#333]" />
                                    <div className="w-3 h-3 rounded-full bg-[#333]" />
                                    <div className="w-3 h-3 rounded-full bg-[#333]" />
                                </div>
                                <div className="p-8 md:p-12 font-mono text-sm md:text-base leading-relaxed text-gray-400 space-y-8">
                                    <div className="flex gap-6">
                                        <span className="text-brand-500 font-bold shrink-0">1.</span>
                                        <span className="text-gray-300" dangerouslySetInnerHTML={{ __html: t('download.step1') as string }} />
                                    </div>
                                    <div className="flex gap-6">
                                        <span className="text-brand-500 font-bold shrink-0">2.</span>
                                        <span className="text-gray-300" dangerouslySetInnerHTML={{ __html: t('download.step2') as string }} />
                                    </div>
                                    {!hasPendingChecksum && (
                                        <div className="pl-10 my-6">
                                            <div className="text-xs text-gray-600 mb-3 uppercase tracking-widest font-bold"># Optional: Verify checksum before running</div>
                                            <div className="p-4 bg-[#0a0a0a] border border-white/5 rounded-xl text-brand-400 break-all shadow-inner relative">
                                                <div className="absolute left-0 top-0 bottom-0 w-1 bg-brand-500/50 rounded-l-xl" />
                                                Get-FileHash .\{exeFileName} -Algorithm SHA256
                                            </div>
                                        </div>
                                    )}
                                    <div className="flex gap-6">
                                        <span className="text-brand-500 font-bold shrink-0">3.</span>
                                        <span className="text-gray-300" dangerouslySetInnerHTML={{ __html: t('download.step3') as string }} />
                                    </div>
                                    <div className="flex gap-6">
                                        <span className="text-brand-500 font-bold shrink-0">4.</span>
                                        <span className="text-gray-300" dangerouslySetInnerHTML={{ __html: t('download.step4') as string }} />
                                    </div>
                                </div>
                            </div>
                        </div>

                    </SectionReveal>
                </Container>
            </section>
        </>
    );
}
