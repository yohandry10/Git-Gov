'use client';

import React, { createContext, useContext, useState, useCallback, useEffect } from 'react';
import { type Locale, type TranslationKey, translations } from './translations';

interface I18nContextValue {
    locale: Locale;
    setLocale: (locale: Locale) => void;
    t: (key: TranslationKey) => string | string[];
}

const I18nContext = createContext<I18nContextValue>({
    locale: 'en',
    setLocale: () => { },
    t: (key) => {
        const entry = translations[key];
        if (!entry) return key;
        return (entry as Record<string, string | string[]>)['en'] ?? key;
    },
});

function resolveInitialLocale(): Locale {
    if (typeof window === 'undefined') {
        return 'en';
    }

    const saved = window.localStorage.getItem('gitgov-locale');
    if (saved === 'en' || saved === 'es') {
        return saved;
    }

    const browserLocale = window.navigator.language.toLowerCase();
    return browserLocale.startsWith('es') ? 'es' : 'en';
}

export function I18nProvider({ children }: { children: React.ReactNode }) {
    const [locale, setLocaleState] = useState<Locale>('en');

    const setLocale = useCallback((newLocale: Locale) => {
        setLocaleState(newLocale);
    }, []);

    // Resolve the real locale after hydration to avoid SSR/client mismatch
    useEffect(() => {
        setLocaleState(resolveInitialLocale());
    }, []);

    useEffect(() => {
        window.localStorage.setItem('gitgov-locale', locale);
        document.documentElement.lang = locale;
    }, [locale]);

    const t = useCallback(
        (key: TranslationKey): string | string[] => {
            const entry = translations[key];
            if (!entry) return key;
            return (entry as Record<string, string | string[]>)[locale] ?? (entry as Record<string, string | string[]>)['en'] ?? key;
        },
        [locale]
    );

    return (
        <I18nContext.Provider value={{ locale, setLocale, t }}>
            {children}
        </I18nContext.Provider>
    );
}

export function useTranslation() {
    return useContext(I18nContext);
}

export { type Locale };
