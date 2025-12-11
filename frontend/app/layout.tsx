import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "NSE Options Analyzer",
  description: "Advanced NSE F&O Options Analysis Platform",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body className="min-h-screen">
        <div className="relative min-h-screen">
          {/* Main Content */}
          <div className="relative z-10">{children}</div>
        </div>
      </body>
    </html>
  );
}

