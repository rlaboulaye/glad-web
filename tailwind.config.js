/** @type {import('tailwindcss').Config} */
export default {
  darkMode: "class",
  content: {
    files: [
      "tailwind.config.js",
      "./site/*.html",
      "./site/src/**/*.rs",
      "*.html",
      "./src/**/*.rs",
    ],
  },
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
      extend: {
        fontFamily: {
          roboto: ["Roboto", "sans-serif"],
        },
      },
    },
  },
  plugins: [],
};
