/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        app: "var(--bg-app)",
        surface: "var(--bg-surface)",
        muted: "var(--bg-muted)",
        border: "var(--border-color)",
        'border-subtle': "var(--border-subtle)",
        accent: "var(--accent)",
        'accent-hover': "var(--accent-hover)",
        'accent-subtle': "var(--accent-subtle)",
      },
      textColor: {
        primary: "var(--text-primary)",
        secondary: "var(--text-secondary)",
        muted: "var(--text-muted)",
      }
    },
  },
  plugins: [],
}
