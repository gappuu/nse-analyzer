
import type { Metadata } from "next";
import "./globals.css";

// --------------------
// SVG GRID BACKGROUND
// --------------------
const gridPattern = `
<svg width='20' height='20' xmlns='http://www.w3.org/2000/svg'>
  <defs>
    <pattern id='grid' width='20' height='20' patternUnits='userSpaceOnUse'>
      <path d='M 20 0 L 0 0 0 20' fill='none' stroke='#374151' stroke-width='0.5' />
    </pattern>
  </defs>
  <rect width='100%' height='100%' fill='url(#grid)'/>
</svg>
`;

const encodedGridPattern = `bg-[url("data:image/svg+xml,${encodeURIComponent(
  gridPattern
)}")]`;

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
          
          {/* Background container */}
          <div className="absolute inset-0 bg-gradient-to-br from-nse-primary via-slate-900 to-nse-primary">
            
            {/* SVG Grid Pattern Layer */}
            <div className={`absolute inset-0 opacity-20 ${encodedGridPattern}`} />
          </div>

          {/* Main Content */}
          <div className="relative z-10">{children}</div>
        </div>
      </body>
    </html>
  );
}