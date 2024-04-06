import React from "react";
import { useEffect, useRef, useState } from "react";

import { invoke } from "@tauri-apps/api/tauri";
import { readBinaryFile } from "@tauri-apps/api/fs";

import Rose from "./components/rose";
import RoseImage from "./components/image";

export default function Home() {
    console.log("jeff!");

    const is_image_loaded = useRef(false);

    const [image_opacity, setImageOpacity] = useState<number>(0);
    const [image, setImage] = useState<[string, [number, number]] | null>(null);

    function load_image() {
        if (is_image_loaded.current == false) {
            invoke<[string, [number, number]] | null>("get_image").then(
                image_data => {
                    if (image_data !== null) {
                        const path = image_data[0];
                        const dimensions = image_data[1];

                        readBinaryFile(path).then(
                            (contents) => {
                                const blob = new Blob([contents], { type: "image/png" });
                                setImage([URL.createObjectURL(blob), dimensions]);

                                setTimeout(() => setImageOpacity(1), 1000);
                            }
                        ).catch(console.error);
                    }
                }
            ).catch(console.error);

            is_image_loaded.current = true;
        }
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
                        <div className="opacity-0 transition-opacity duration-300" style={{opacity: image_opacity}}>
                            <RoseImage url={image[0]} width={image[1][0]} height={image[1][1]}></RoseImage>
                        </div>
                }
            </div>
        </div>
    );
}