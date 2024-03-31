"use client";

import NextImage from "next/image";
import { useEffect, useState, WheelEventHandler } from "react";

import { invoke } from "@tauri-apps/api/tauri";
import { appWindow } from "@tauri-apps/api/window";
import { readBinaryFile } from "@tauri-apps/api/fs";

const IMAGE_PADDING = 8;

type Dimensions = {
    width: number,
    height: number
}

export default function RoseImage() {
    const [image, setImage] = useState<[string, [number, number]] | null>(null);
    const [zoom_position, setZoomPosition] = useState({ x: 0, y: 0, scale: 1 });
    const [image_bounds, setImageBounds] = useState<Dimensions | null>(null);
    const [window_size, setWindowSize] = useState<Dimensions | null>(null);

    const on_scroll: WheelEventHandler = (event) => {
        const delta = event.deltaY * -0.005 * (zoom_position.scale / 2);
        const new_scale = zoom_position.scale + delta;

        const ratio = 1 - new_scale / zoom_position.scale;

        if (new_scale < 0) return;

        console.log(delta, new_scale);

        setZoomPosition({
            scale: new_scale,
            x: zoom_position.x + (event.clientX - zoom_position.x) * ratio,
            y: zoom_position.y + (event.clientY - zoom_position.y) * ratio
        });
    };

    useEffect(() => {
        appWindow.innerSize().then(
            (size) => setWindowSize({width: size.width, height: size.height})
        );
    });

    useEffect(() => {
        if (window_size !== null) {
            const width = window_size.width - IMAGE_PADDING * 2;
            const height = window_size.height - IMAGE_PADDING * 2;

            if (width !== image_bounds?.width || height !== image_bounds?.height || image_bounds == null) {
                setZoomPosition({ x: 0, y: 0, scale: 1 });
                setImageBounds({width: width, height: height});
            }
        }
    }, [image_bounds, window_size]);

    useEffect(() => {
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
                }
            }
        ).catch(console.error);
    }, []);

    //document.getElementById("image-dev")?.addEventListener("contextmenu", event => event.preventDefault());

    return (
        <div 
            id="image-dev" onWheelCapture={on_scroll}
            className="select-none cursor-default relative">

            <div 
                className="flex items-center justify-center h-screen transition-transform duration-500" 
                style={{
                    padding: IMAGE_PADDING, 

                    transformOrigin: "0 0", 
                    transform: `translate(${zoom_position.x}px, ${zoom_position.y}px) scale(${zoom_position.scale})`
                }}>
                {
                    image === null ? 
                        <h1 className="font-dosis font-medium text-white text-5xl">🌹</h1> : 
                        <figure className="rounded-lg size-max overflow-hidden">
                            <NextImage
                                className="w-auto h-auto transition-all duration-1000 delay-500"
                                style={{
                                    maxHeight: `${image_bounds?.height}px`, 
                                    maxWidth: `${image_bounds?.width}px`, 
                                }}
                                src={image[0]} width={image[1][0]} height={image[1][1]} alt=""/>
                        </figure>
                }
            </div>
        </div>
    );
}