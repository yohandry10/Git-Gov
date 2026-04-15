import fs from 'fs';
import path from 'path';
import matter from 'gray-matter';

const docsDirectory = path.join(process.cwd(), 'content', 'docs');
const docsDirectoryResolved = path.resolve(docsDirectory);
const SUPPORTED_DOC_LOCALES = ['en', 'es'] as const;
const SAFE_DOC_SLUG = /^[a-z0-9][a-z0-9-]*$/i;
type DocLocale = (typeof SUPPORTED_DOC_LOCALES)[number];
type LocalizedDocPathMap = {
    en: string;
    es?: string;
};

export interface DocPage {
    slug: string;
    title: string;
    description: string;
    order: number;
    contentMarkdown: string;
}

export interface DocMeta {
    slug: string;
    title: string;
    description: string;
    order: number;
}

function normalizeDocLocale(locale: string): string {
    const normalized = locale.trim().toLowerCase();
    if ((SUPPORTED_DOC_LOCALES as readonly string[]).includes(normalized)) {
        return normalized;
    }
    return 'en';
}

function normalizeDocSlug(slug: string): string | null {
    const normalized = slug.trim();
    if (!normalized || !SAFE_DOC_SLUG.test(normalized)) {
        return null;
    }
    return normalized;
}

function readDocPathIndex(): Map<string, LocalizedDocPathMap> {
    const index = new Map<string, LocalizedDocPathMap>();
    let entries: fs.Dirent[];
    try {
        entries = fs.readdirSync(docsDirectoryResolved, { withFileTypes: true });
    } catch {
        return index;
    }

    for (const entry of entries) {
        if (!entry.isFile() || !entry.name.endsWith('.md')) {
            continue;
        }

        const slug = normalizeDocSlug(entry.name.replace(/\.md$/, ''));
        if (!slug) {
            continue;
        }

        const localizedPaths: LocalizedDocPathMap = {
            en: path.join(docsDirectoryResolved, entry.name),
        };

        const nonDefaultLocales = SUPPORTED_DOC_LOCALES.filter((locale) => locale !== 'en');
        for (const locale of nonDefaultLocales) {
            const candidatePath = path.join(docsDirectoryResolved, locale, entry.name);
            if (fs.existsSync(candidatePath)) {
                localizedPaths[locale] = candidatePath;
            }
        }

        index.set(slug, localizedPaths);
    }

    return index;
}

function pickPathForLocale(entry: LocalizedDocPathMap, locale: DocLocale): string {
    if (locale === 'en') {
        return entry.en;
    }
    return entry[locale] ?? entry.en;
}

export function getDocsSlugs(): string[] {
    return Array.from(readDocPathIndex().keys()).sort((a, b) => a.localeCompare(b));
}

export function getDocsMeta(locale: string = 'en'): DocMeta[] {
    const normalizedLocale = normalizeDocLocale(locale) as DocLocale;
    const docs = Array.from(readDocPathIndex().entries()).map(([slug, paths]) => {
        const fullPath = pickPathForLocale(paths, normalizedLocale);
        const fileContents = fs.readFileSync(fullPath, 'utf-8');
        const { data } = matter(fileContents);
        return {
            slug,
            title: (data.title as string) || slug,
            description: (data.description as string) || '',
            order: (data.order as number) || 99,
        };
    });
    return docs.sort((a, b) => a.order - b.order);
}

export async function getDocBySlug(slug: string, locale: string = 'en'): Promise<DocPage | null> {
    try {
        const normalizedSlug = normalizeDocSlug(slug);
        if (!normalizedSlug) {
            return null;
        }

        const docPathIndex = readDocPathIndex();
        const docPaths = docPathIndex.get(normalizedSlug);
        if (!docPaths) {
            return null;
        }

        const normalizedLocale = normalizeDocLocale(locale) as DocLocale;
        const fullPath = pickPathForLocale(docPaths, normalizedLocale);
        const fileContents = fs.readFileSync(fullPath, 'utf-8');
        const { data, content } = matter(fileContents);

        return {
            slug: normalizedSlug,
            title: (data.title as string) || normalizedSlug,
            description: (data.description as string) || '',
            order: (data.order as number) || 99,
            contentMarkdown: content,
        };
    } catch {
        return null;
    }
}
