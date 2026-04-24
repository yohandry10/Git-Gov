import path from 'path'
import { fileURLToPath } from 'url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))

/** @type {import('next').NextConfig} */
const nextConfig = {
    reactStrictMode: true,
    outputFileTracingRoot: __dirname,
    experimental: {
        optimizePackageImports: ['react-icons/hi', 'react-icons/fa', 'framer-motion'],
    },
    images: {
        formats: ['image/avif', 'image/webp'],
        remotePatterns: [],
    },
    poweredByHeader: false,
};

export default nextConfig;
