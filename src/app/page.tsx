"use client";

import { useEffect, useState } from "react";

import { invoke } from "@tauri-apps/api/tauri";
import { readBinaryFile } from "@tauri-apps/api/fs";

import Rose from "./components/rose";
import RoseImage from "./components/dynamic_image";

export default function Home() {
    const [image_opacity, setImageOpacity] = useState<number>(0);
    const [image, setImage] = useState<[string, [number, number]] | null>(null);

    function load_image() {
        invoke<[string, [number, number]] | null>("get_image").then(
            image => {
                if (image !== null) {
                    const path = image[0];
                    const dimensions = image[1];

                    readBinaryFile(path).then(
                        (contents) => {
                            const blob = new Blob([contents], { type: "image/png" });
                            setImage([URL.createObjectURL(blob), dimensions]);
                        }
                    ).catch(console.error);
            
                    setTimeout(() => setImageOpacity(1), 2000);
                }
            }
        ).catch(console.error);
    }

    useEffect(() => load_image(), []);

    return (
        <div className="relative">
            <div className="flex items-center justify-center h-screen">
                {
                    image === null ? 
                        <div onClick={ () => {invoke("select_image").then(() => load_image());} }>
                            <Rose></Rose>
                        </div> : 
                        <div id="opacity-div" className="opacity-0 transition-opacity duration-300" style={{opacity: image_opacity}}>
                            <RoseImage url={image[0]} width={image[1][0]} height={image[1][1]}></RoseImage>
                        </div>
                }
            </div>
        </div>
    );
}