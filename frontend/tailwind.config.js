/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./app/**/*.{js,ts,jsx,tsx,mdx}",
    "./pages/**/*.{js,ts,jsx,tsx,mdx}",
    "./components/**/*.{js,ts,jsx,tsx,mdx}",
  ],
  theme: {
    extend: {
      colors: {
        nse: {
          primary: "#1E293B",     // slate-800
          secondary: "#3B82F6",   // blue-500
          accent: "#10B981",      // emerald-500
          surface: "#334155",     // slate-700
        },
      },
      fontFamily: {
        body: ['Inter', 'Arial', 'Helvetica', 'sans-serif'],
        mono: ['JetBrains Mono', 'monospace'],
      },
    },
  },
  plugins: [],
};