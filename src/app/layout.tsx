import type { Metadata } from "next";
import { Inter } from "next/font/google";

import "./globals.css";

const inter = Inter({ subsets: ["latin"] });

export const metadata: Metadata = {
    title: "Roseate",
    description: "A small and simple but fancy image viewer built with Rust & Typescript that's cross platform.",
};

export default function RootLayout({
    children,
}: Readonly<{
    children: React.ReactNode;
}>) {
    return (
        <html lang="en" className="bg-cloudyLight dark:bg-cloudyDark h-screen overflow-hidden">
            <body className={inter.className}>{children}</body>
        </html>
    );
}