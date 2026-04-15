import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

/** @type {import('next').NextConfig} */
const nextConfig = {
    reactStrictMode: true,
    images: {
        formats: ['image/avif', 'image/webp'],
        // No remote images are required; keep optimizer scope local-only.
        remotePatterns: [],
    },
    poweredByHeader: false,
    outputFileTracingRoot: __dirname,
    outputFileTracingIncludes: {
        '/docs/[[...slug]]': ['./content/**/*'],
    },
};

export default nextConfig;
