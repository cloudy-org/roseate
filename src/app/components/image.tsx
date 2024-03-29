"use client";

import NextImage from "next/image";
import { useEffect, useState } from "react";

import { invoke } from "@tauri-apps/api/tauri";
import { readBinaryFile } from "@tauri-apps/api/fs";
import { appWindow, LogicalSize } from "@tauri-apps/api/window";

export default function RoseImage() {
    const image_padding = 8;
    const [image, setImage] = useState<[string, [number, number]] | null>(null);

    useEffect(() => {
        invoke<[string, [number, number]] | null>("get_image").then(
            image => {
                if (image !== null) {
                    const path = image[0];
                    const dimensions = image[1];

                    const window_minimal_size = new LogicalSize(
                        dimensions[0] + image_padding * 2, 
                        dimensions[1] + image_padding * 2
                    );

                    appWindow.setMinSize(window_minimal_size)
                        .catch(console.error);

                    readBinaryFile(path).then(
                        (contents) => {
                            const blob = new Blob([contents], { type: "image/png" });
                            setImage([URL.createObjectURL(blob), dimensions]);
                        }
                    ).catch(console.error);
                }
            }
        ).catch(console.error);
    }, []);

    document.getElementById("image-dev")?.addEventListener(
        "contextmenu", event => event.preventDefault()
    );

    return (
        <div id="image-dev" className="select-none cursor-default relative">
            <div style={{padding: image_padding}} className="flex items-center justify-center h-screen">
                {
                    image === null ? 
                        <h1 className="font-dosis font-medium text-white text-2xl">🌹 no image</h1> : 
                        <figure className="rounded-lg size-max overflow-hidden">
                            <NextImage src={image[0]} width={image[1][0]} height={image[1][1]} alt=""/>
                        </figure>
                }
            </div>
        </div>
    );
}