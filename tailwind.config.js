/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        primary: {
          50: '#eff6ff',
          500: '#3b82f6',
          600: '#2563eb',
          700: '#1d4ed8',
          900: '#1e3a8a',
        },
        terminal: {
          bg: '#0d1117',
          text: '#c9d1d9',
          selection: '#264f78',
        },
        gray: {
          750: '#3f4751',
          850: '#1c2128',
          950: '#0c1015',
        }
      },
      fontFamily: {
        mono: ['JetBrains Mono', 'Monaco', 'Menlo', 'monospace'],
      }
    },
  },
  darkMode: 'class',
  plugins: [],
}