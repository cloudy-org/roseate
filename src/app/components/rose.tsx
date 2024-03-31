"use client";

export default function Rose() {
    return (
        <div className="cursor-pointer">
            <div className="relative py-3">
                <h1 className="flex items-center justify-center font-dosis font-medium text-white text-5xl">🌹</h1>
            </div>

            {/* TODO: Add a good light mode text colour to this. */}
            <h3 className="dark:text-red-400 font-semibold">click to select an image</h3>
        </div>
    );
}