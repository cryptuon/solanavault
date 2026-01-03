/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        'vault-dark': '#1a1a2e',
        'vault-darker': '#16213e',
        'vault-accent': '#0f3460',
        'vault-cyan': '#00d9ff',
        'vault-green': '#00ff88',
        'vault-yellow': '#ffcc00',
        'vault-red': '#ff4444',
      },
    },
  },
  plugins: [],
}
