import React from "react";
import { useEffect, useState } from "react";

import { invoke } from "@tauri-apps/api/tauri";
import { readBinaryFile } from "@tauri-apps/api/fs";

import Rose from "./components/rose";
import RoseImage from "./components/image";

export type Image = {
    url: string,
    width: number,
    height: number
}

export default function Home() {
    const [image_loading, setImageLoading] = useState(false);
    const [no_image_available, setNoImageAvailable] = useState(false);

    const [image_opacity, setImageOpacity] = useState<number>(0);
    const [image, setImage] = useState<Image | null>(null);

    function load_image() {
        if (image_loading == false && image == null && no_image_available == false) {
            setImageLoading(true);

            invoke<[string, [number, number]] | null>("get_image").then(
                image_data => {
                    if (image_data !== null) {
                        const path = image_data[0];
                        const dimensions = image_data[1];

                        readBinaryFile(path).then(
                            (contents) => {
                                const blob = new Blob([contents], { type: "image/png" });
                                const url = URL.createObjectURL(blob);
                                setImage({
                                    url: url,
                                    width: dimensions[0],
                                    height: dimensions[1]
                                });

                                setTimeout(() => {
                                    setImageLoading(false);
                                    setImageOpacity(1);
                                }, 1000);
                            }
                        ).catch(console.error);
                    } else {
                        setNoImageAvailable(true);
                        setImageLoading(false);
                    }
                }
            ).catch(console.error);
        }
    }

    useEffect(() => load_image());

    return (
        <div className="relative">
            <div className="flex items-center justify-center h-screen">
                {
                    image === null ? 
                        <div onClick={() => {
                            if (!image_loading) {
                                invoke("select_image").then(() => {
                                    setNoImageAvailable(false);
                                    load_image();
                                });
                            }
                        }}>
                            <Rose image_loading={image_loading}></Rose>
                        </div> : 
                        <div className="opacity-0 transition-opacity duration-300" style={{opacity: image_opacity}}>
                            <RoseImage image={image}></RoseImage>
                        </div>
                }
            </div>
        </div>
    );
}