'use client';

import React, { useState, useEffect } from 'react';
import Image from 'next/image';
import { usePathname } from 'next/navigation';

const PRELOADER_SESSION_KEY = 'gitgov-web-preloader-seen';

export function Preloader() {
    const pathname = usePathname();
    const shouldRenderPreloader = process.env.NODE_ENV === 'production' && pathname === '/';
    const [phase, setPhase] = useState(() => (shouldRenderPreloader ? 0 : 2));
    // 0 = fox visible + glow fades in, 1 = fade out, 2 = done

    useEffect(() => {
        if (!shouldRenderPreloader) {
            setPhase(2);
            return;
        }

        try {
            if (sessionStorage.getItem(PRELOADER_SESSION_KEY) === '1') {
                setPhase(2);
                return;
            }
            sessionStorage.setItem(PRELOADER_SESSION_KEY, '1');
        } catch {
            // If sessionStorage is unavailable, skip persistence and keep one-time render.
        }

        setPhase(0);
        const t0 = setTimeout(() => setPhase(1), 550);
        const t1 = setTimeout(() => setPhase(2), 850);
        return () => [t0, t1].forEach(clearTimeout);
    }, [shouldRenderPreloader]);

    if (phase >= 2) return null;

    return (
        <div style={{
            position: 'fixed',
            inset: 0,
            zIndex: 100,
            backgroundColor: '#000',
            overflow: 'hidden',
            opacity: phase >= 1 ? 0 : 1,
            transition: phase >= 1 ? 'opacity 0.45s ease-in' : 'none',
        }}>
            {/* Fox */}
            <div
                style={{
                    position: 'absolute',
                    top: '50%',
                    left: '50%',
                    transform: 'translate(-50%, -50%)',
                    height: '85vh',
                    width: '90vw',
                    maxHeight: '85vh',
                    maxWidth: '90vw',
                    zIndex: 5,
                    userSelect: 'none',
                    pointerEvents: 'none',
                }}
            >
                <Image
                    src="/fox.png"
                    alt=""
                    fill
                    priority
                    draggable={false}
                    sizes="(max-width: 768px) 90vw, 70vw"
                    style={{
                        objectFit: 'contain',
                    }}
                />
            </div>

            {/* Glow */}
            <div style={{
                position: 'absolute',
                top: '50%',
                left: '50%',
                transform: 'translate(-50%, -50%) scale(1.1)',
                width: '70vmin',
                height: '70vmin',
                borderRadius: '50%',
                background: 'radial-gradient(circle, rgba(249,115,22,0.45) 0%, rgba(251,191,36,0.18) 40%, transparent 68%)',
                opacity: 0.9,
                zIndex: 2,
                pointerEvents: 'none',
            }} />
        </div>
    );
}
