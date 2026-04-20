'use client';

import React from 'react';
import { motion } from 'framer-motion';
import { SectionReveal } from '@/components/ui/SectionReveal';
import { HiOutlineDesktopComputer, HiOutlineServer, HiOutlineLink, HiOutlineArrowNarrowRight } from 'react-icons/hi';
import { useTranslation } from '@/lib/i18n';

interface StepConfig {
    icon: React.ReactNode;
    labelKey: string;
    descKey: string;
    step: string;
}

const stepConfigs: StepConfig[] = [
    {
        icon: <HiOutlineDesktopComputer size={22} />,
        labelKey: 'howItWorks.desktop',
        descKey: 'howItWorks.desktopDesc',
        step: '01',
    },
    {
        icon: <HiOutlineServer size={22} />,
        labelKey: 'howItWorks.controlPlane',
        descKey: 'howItWorks.controlPlaneDesc',
        step: '02',
    },
    {
        icon: <HiOutlineLink size={22} />,
        labelKey: 'howItWorks.integrations',
        descKey: 'howItWorks.integrationsDesc',
        step: '03',
    },
];

export function FlowDiagram() {
    const { t } = useTranslation();

    const steps = stepConfigs.map((s) => ({
        ...s,
        label: t(s.labelKey as any) as string,
        description: t(s.descKey as any) as string,
    }));

    return (
        <SectionReveal>
            {/* Horizontal numbered timeline */}
            <div className="relative max-w-4xl mx-auto">

                {/* Desktop */}
                <div className="hidden md:block">
                    {/* The line */}
                    <div className="absolute top-[28px] left-[80px] right-[80px] h-[2px] bg-white/[0.06] z-0">
                        <motion.div
                            className="h-full bg-gradient-to-r from-brand-500 via-brand-400 to-accent-400"
                            initial={{ scaleX: 0 }}
                            whileInView={{ scaleX: 1 }}
                            viewport={{ once: true }}
                            transition={{ duration: 1.5, delay: 0.3, ease: 'easeOut' }}
                            style={{ transformOrigin: 'left' }}
                        />
                    </div>

                    <div className="grid grid-cols-3 relative z-10">
                        {steps.map((step, i) => (
                            <motion.div
                                key={step.labelKey}
                                className="flex flex-col items-center"
                                initial={{ opacity: 0, y: 15 }}
                                whileInView={{ opacity: 1, y: 0 }}
                                viewport={{ once: true }}
                                transition={{ delay: 0.4 + i * 0.2, duration: 0.5 }}
                            >
                                {/* Step circle */}
                                <div className="w-14 h-14 rounded-full bg-surface-200 border-2 border-brand-500/40 flex items-center justify-center text-brand-400 mb-4 relative">
                                    {step.icon}
                                    <span className="absolute -top-2 -right-2 text-[10px] font-black text-brand-400 bg-surface-500 border border-brand-500/30 rounded-full w-6 h-6 flex items-center justify-center">
                                        {step.step}
                                    </span>
                                </div>

                                <h3 className="text-sm font-bold text-white mb-1 text-center">{step.label}</h3>
                                <p className="text-xs text-gray-500 max-w-[180px] text-center leading-relaxed">{step.description}</p>
                            </motion.div>
                        ))}
                    </div>
                </div>

                {/* Mobile: compact vertical */}
                <div className="md:hidden space-y-0">
                    {steps.map((step, i) => (
                        <div key={step.labelKey} className="flex items-start gap-4">
                            <div className="flex flex-col items-center">
                                <div className="w-10 h-10 rounded-full bg-surface-200 border border-brand-500/30 flex items-center justify-center text-brand-400 shrink-0">
                                    {step.icon}
                                </div>
                                {i < steps.length - 1 && (
                                    <div className="w-px h-8 bg-gradient-to-b from-brand-500/30 to-transparent" />
                                )}
                            </div>
                            <div className="pt-1.5 pb-4">
                                <h3 className="text-sm font-bold text-white">{step.label}</h3>
                                <p className="text-xs text-gray-500 mt-0.5">{step.description}</p>
                            </div>
                        </div>
                    ))}
                </div>

            </div>
        </SectionReveal>
    );
}
