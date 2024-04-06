import React from "react";
import { useState } from "react";

export default function Rose() {
    const [tip_text_opacity, setTextOpacity] = useState<number>(0);

    setInterval(
        () => {
            setTextOpacity(1);
        }, 3000
    );

    return (
        <div className="cursor-pointer">
            <div className="relative py-3">
                <h1 className="flex items-center justify-center font-dosis font-medium text-white text-5xl">🌹</h1>
            </div>

            {/* TODO: Add a good light mode text colour to this. */}
            <h3 className="dark:text-red-400 font-semibold transition-opacity ease-in-out duration-[2s]" style={{opacity: tip_text_opacity}}>click to select an image</h3>
        </div>
    );
}