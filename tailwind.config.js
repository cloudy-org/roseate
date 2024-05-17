/** @type {import('tailwindcss').Config} */
import * as CTKTailwind from "./cirrus/tailwind";

const config = {
    darkMode: "selector",
    plugins: [
        CTKTailwind.Colours
    ],
    content: [
        "./src/**/*.{js,ts,jsx,tsx,mdx}",
        "./src/components/**/*.{js,ts,jsx,tsx,mdx}",
    ],
    theme: {},
};

export default config;