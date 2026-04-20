'use client';

import React from 'react';
import { Hero, SectionHeader, CTASection, RoleCards, FAQSection, CapabilitiesSection } from '@/components/marketing';
import { ProblemSection } from './ProblemSection';
import { TrustSection } from './TrustSection';
import { Container } from '@/components/layout';
import { SectionReveal } from '@/components/ui';
import { useTranslation } from '@/lib/i18n';

import {
    FaUserTie,
    FaUserShield,
    FaUserCog,
} from 'react-icons/fa';
import {
    HiOutlineClipboardCheck,
    HiOutlineChartBar,
} from 'react-icons/hi';

export function HomeClient() {
    const { t } = useTranslation();

    return (
        <>
            {/* ═══ Hero ═══ */}
            <Hero />

            {/* ═══ The Problem / Solution ═══ */}
            <ProblemSection />

            {/* ═══ Who It's For ═══ */}
            <section className="py-16 md:py-24" id="who-its-for">
                <Container>
                    <SectionHeader
                        badge={t('roles.badge') as string}
                        title={t('roles.title') as string}
                        titleAccent={t('roles.titleAccent') as string}
                        description={t('roles.description') as string}
                    />

                    <RoleCards
                        roles={[
                            {
                                icon: <FaUserShield className="text-brand-400" size={24} />,
                                role: t('roles.cto.role') as string,
                                painPoint: t('roles.cto.pain') as string,
                                solution: t('roles.cto.solution') as string,
                            },
                            {
                                icon: <FaUserTie className="text-accent-400" size={24} />,
                                role: t('roles.em.role') as string,
                                painPoint: t('roles.em.pain') as string,
                                solution: t('roles.em.solution') as string,
                            },
                            {
                                icon: <FaUserCog className="text-brand-300" size={24} />,
                                role: t('roles.devops.role') as string,
                                painPoint: t('roles.devops.pain') as string,
                                solution: t('roles.devops.solution') as string,
                            },
                            {
                                icon: <HiOutlineClipboardCheck className="text-accent-300" size={24} />,
                                role: t('roles.auditor.role') as string,
                                painPoint: t('roles.auditor.pain') as string,
                                solution: t('roles.auditor.solution') as string,
                            },
                            {
                                icon: <HiOutlineChartBar className="text-brand-400" size={24} />,
                                role: t('roles.vpe.role') as string,
                                painPoint: t('roles.vpe.pain') as string,
                                solution: t('roles.vpe.solution') as string,
                            },
                        ]}
                    />
                </Container>
            </section>

            {/* ═══ Trust / Architecture ═══ */}
            <TrustSection />


            {/* ═══ Key Capabilities ═══ */}
            <CapabilitiesSection />

            {/* ═══ FAQ ═══ */}
            <FAQSection maxItems={3} />

            {/* ═══ CTA ═══ */}
            <CTASection
                title={t('cta.title') as string}
                titleAccent={t('cta.titleAccent') as string}
                description={t('cta.description') as string}
                primaryCta={{ label: t('hero.cta') as string, href: '/contact' }}
                secondaryCta={{ label: t('cta.primary') as string, href: '/download' }}
            />
        </>
    );
}
