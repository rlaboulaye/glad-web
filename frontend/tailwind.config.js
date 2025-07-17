/** @type {import('tailwindcss').Config} */
export default {
  darkMode: "class",
  content: [
    './src/**/*.{html,js,svelte,ts}',
    './src/app.html'
  ],
  safelist: [
    "hover:underline",
    "text-green-400",
    "dark:text-green-300",
    "p-8",
    "shadow-md",
    "rounded-lg",
    "mt-8",
    "space-y-6",
  ],
  theme: {
    extend: {
      fontFamily: {
        roboto: ["Roboto", "sans-serif"],
      },
      colors: {
        primary_dark: "#8C4F2B",
        primary: "#BF8B5E",
        btn: "#BFA893",
        secondary_dark: "#0D0D0D",
        secondary: "#031626",
      },
    },
  },
  plugins: [],
};