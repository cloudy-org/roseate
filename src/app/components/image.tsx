"use client";

import NextImage from "next/image";
import { useEffect, useState } from "react";

import { invoke } from "@tauri-apps/api/tauri";
import { readBinaryFile } from "@tauri-apps/api/fs";
import { appWindow } from "@tauri-apps/api/window";

const IMAGE_PADDING = 8;

export default function RoseImage() {
    const [image, setImage] = useState<[string, [number, number]] | null>(null);
    const [image_bound, setImageBounds] = useState<[number, number] | null>(null);

    useEffect(() => {
        invoke<[string, [number, number]] | null>("get_image").then(
            image => {
                if (image !== null) {
                    const path = image[0];
                    const dimensions = image[1];

                    const image_aspect_ratio = 1.0 * dimensions[1] / dimensions[0];

                    console.log(">>", image_aspect_ratio);

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

    useEffect(() => {
        appWindow.innerSize().then((size) => setImageBounds(
            [size.width - IMAGE_PADDING * 2, size.height - IMAGE_PADDING * 2]
        ));
    });

    //document.getElementById("image-dev")?.addEventListener("contextmenu", event => event.preventDefault());

    return (
        <div id="image-dev" className="select-none cursor-default relative">
            <div style={{padding: IMAGE_PADDING}} className="flex items-center justify-center h-screen">
                {
                    image === null ? 
                        <h1 className="font-dosis font-medium text-white text-5xl">🌹</h1> : 
                        <figure className="rounded-lg size-max overflow-hidden">
                            <NextImage 
                                className="w-auto h-auto transition-all duration-1000 delay-500"
                                style={{maxHeight: `${image_bound?.[1]}px`, maxWidth: `${image_bound?.[0]}px`}} 
                                src={image[0]} width={image[1][0]} height={image[1][1]} alt=""/>
                        </figure>
                }
            </div>
        </div>
    );
}