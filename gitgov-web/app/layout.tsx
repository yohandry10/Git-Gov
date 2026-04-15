import type { Metadata, Viewport } from 'next';
import { JetBrains_Mono, Outfit } from 'next/font/google';
import { generatePageMetadata } from '@/lib/seo/metadata';
import { ClientLayout } from './client-layout';
import './globals.css';

const outfit = Outfit({
    subsets: ['latin'],
    weight: ['300', '400', '500', '600', '700', '800', '900'],
    display: 'swap',
    variable: '--font-sans',
});

const jetBrainsMono = JetBrains_Mono({
    subsets: ['latin'],
    weight: ['400', '500', '600'],
    display: 'swap',
    variable: '--font-mono',
});

export const metadata: Metadata = {
    ...generatePageMetadata(),
    manifest: '/manifest.json',
    icons: {
        icon: '/logo.png',
        apple: '/logo.png',
    },
};

export const viewport: Viewport = {
    themeColor: '#090909',
    width: 'device-width',
    initialScale: 1,
};

export default function RootLayout({
    children,
}: {
    children: React.ReactNode;
}) {
    return (
        <html lang="en" className="dark" suppressHydrationWarning>
            <body className={`${outfit.variable} ${jetBrainsMono.variable} min-h-[100dvh] bg-surface-300 text-white antialiased`}>
                <ClientLayout>{children}</ClientLayout>
            </body>
        </html>
    );
}
