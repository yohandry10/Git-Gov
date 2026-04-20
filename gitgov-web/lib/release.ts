import { promises as fs } from 'node:fs';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { siteConfig } from '@/lib/config/site';

export interface ReleaseMetadata {
    version: string;
    downloadUrl: string;
    checksum: string;
    msiUrl: string | null;
    available: boolean;
}

const REMOTE_CHECK_TIMEOUT_MS = 5000;
const REMOTE_CACHE_TTL_MS = 60_000;

interface CachedLocalRelease {
    absolutePath: string;
    size: number;
    mtimeMs: number;
    result: ReleaseMetadata;
}

interface CachedRemoteRelease {
    url: string;
    checkedAt: number;
    result: ReleaseMetadata;
}

let cachedLocalRelease: CachedLocalRelease | null = null;
let cachedRemoteRelease: CachedRemoteRelease | null = null;

function isHttpUrl(value: string): boolean {
    return /^https?:\/\//i.test(value);
}

async function checkRemoteAsset(url: string): Promise<boolean> {
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), REMOTE_CHECK_TIMEOUT_MS);

    try {
        const headResponse = await fetch(url, {
            method: 'HEAD',
            redirect: 'follow',
            cache: 'no-store',
            signal: controller.signal,
        });

        if (headResponse.ok) {
            return true;
        }

        if (headResponse.status !== 405) {
            return false;
        }

        const getResponse = await fetch(url, {
            method: 'GET',
            headers: { Range: 'bytes=0-0' },
            redirect: 'follow',
            cache: 'no-store',
            signal: controller.signal,
        });

        return getResponse.ok || getResponse.status === 206;
    } catch {
        return false;
    } finally {
        clearTimeout(timer);
    }
}

export async function getReleaseMetadata(): Promise<ReleaseMetadata> {
    const msiUrl = siteConfig.downloadMsiUrl;

    // External URL mode: validate the remote asset exists before enabling CTA.
    if (isHttpUrl(siteConfig.downloadPath)) {
        if (
            cachedRemoteRelease &&
            cachedRemoteRelease.url === siteConfig.downloadPath &&
            Date.now() - cachedRemoteRelease.checkedAt < REMOTE_CACHE_TTL_MS
        ) {
            return cachedRemoteRelease.result;
        }

        const available = await checkRemoteAsset(siteConfig.downloadPath);
        const result = {
            version: siteConfig.version,
            downloadUrl: siteConfig.downloadPath,
            checksum: siteConfig.downloadChecksum,
            msiUrl,
            available,
        };

        cachedRemoteRelease = {
            url: siteConfig.downloadPath,
            checkedAt: Date.now(),
            result,
        };

        return result;
    }

    const relativePath = siteConfig.downloadPath.replace(/^\//, '');
    const absolutePath = path.join(process.cwd(), 'public', relativePath);

    try {
        const stat = await fs.stat(absolutePath);
        if (!stat.isFile()) {
            return {
                version: siteConfig.version,
                downloadUrl: siteConfig.downloadPath,
                checksum: siteConfig.downloadChecksum,
                msiUrl,
                available: false,
            };
        }

        if (
            cachedLocalRelease &&
            cachedLocalRelease.absolutePath === absolutePath &&
            cachedLocalRelease.size === stat.size &&
            cachedLocalRelease.mtimeMs === stat.mtimeMs
        ) {
            return cachedLocalRelease.result;
        }

        const buffer = await fs.readFile(absolutePath);
        const checksum = `sha256:${createHash('sha256').update(buffer).digest('hex')}`;
        const result = {
            version: siteConfig.version,
            downloadUrl: siteConfig.downloadPath,
            checksum,
            msiUrl,
            available: true,
        };

        cachedLocalRelease = {
            absolutePath,
            size: stat.size,
            mtimeMs: stat.mtimeMs,
            result,
        };

        return result;
    } catch {
        return {
            version: siteConfig.version,
            downloadUrl: siteConfig.downloadPath,
            checksum: siteConfig.downloadChecksum,
            msiUrl,
            available: false,
        };
    }
}
