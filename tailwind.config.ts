import type { Config } from "tailwindcss";

import * as CTKTailwind from "./cirrus/tailwind";

const config: Config = {
    plugins: [
        CTKTailwind.Colours
    ],
    content: [
        "./src/pages/**/*.{js,ts,jsx,tsx,mdx}",
        "./src/components/**/*.{js,ts,jsx,tsx,mdx}",
        "./src/app/**/*.{js,ts,jsx,tsx,mdx}",
    ],
    theme: {},
};
export default config;