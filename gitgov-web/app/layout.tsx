import type { Metadata, Viewport } from 'next';
import { JetBrains_Mono, Plus_Jakarta_Sans } from 'next/font/google';
import { generatePageMetadata } from '@/lib/seo/metadata';
import { ClientLayout } from './client-layout';
import './globals.css';

const plusJakartaSans = Plus_Jakarta_Sans({
    subsets: ['latin'],
    weight: ['300', '400', '500', '600', '700', '800'],
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
        icon: '/logo-192.png',
        apple: '/logo-192.png',
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
            <body className={`${plusJakartaSans.variable} ${jetBrainsMono.variable} font-sans min-h-[100dvh] bg-surface-300 text-white antialiased`}>
                <ClientLayout>{children}</ClientLayout>
            </body>
        </html>
    );
}
