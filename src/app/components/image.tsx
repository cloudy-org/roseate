"use client";

import NextImage from "next/image";
import { useEffect, useState, WheelEventHandler } from "react";

import { appWindow } from "@tauri-apps/api/window";

const IMAGE_PADDING = 8;

type Dimensions = {
    width: number,
    height: number
}

type Props = {
    url: string,
    width: number,
    height: number
}

export default function RoseImage(props: Props) {
    const [zoom_position, setZoomPosition] = useState({ x: 0, y: 0, scale: 1 });
    const [image_bounds, setImageBounds] = useState<Dimensions>({width: props.width, height: props.height});
    const [window_size, setWindowSize] = useState<Dimensions | null>(null);

    const on_scroll: WheelEventHandler = (event) => {
        const delta = event.deltaY * -0.005 * (zoom_position.scale / 2);
        const new_scale = zoom_position.scale + delta;

        const ratio = 1 - new_scale / zoom_position.scale;

        if (new_scale < 1) {
            setTimeout(() => setZoomPosition({ x: 0, y: 0, scale: 1 }), 1000);
        }

        console.log(delta, new_scale);

        setZoomPosition({
            scale: new_scale,
            x: zoom_position.x + (event.clientX - zoom_position.x) * ratio,
            y: zoom_position.y + (event.clientY - zoom_position.y) * ratio
        });
    };

    // Keeps track of tauri window size.
    useEffect(() => {
        appWindow.innerSize().then(
            (size) => setWindowSize({width: size.width, height: size.height})
        );
    });

    // Scales image on window resize and reset's zoom.
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

    return (
        <div onWheelCapture={on_scroll} className="select-none cursor-default">
            <div className="transition-transform duration-700 delay-0" 
                style={{
                    padding: IMAGE_PADDING, 

                    transformOrigin: "0 0", 
                    transform: `translate(${zoom_position.x}px, ${zoom_position.y}px) scale(${zoom_position.scale})`,
                }}>

                <figure className="rounded-lg size-max overflow-hidden">
                    <NextImage
                        className="w-auto h-auto transition-all duration-1000 delay-500"
                        style={{
                            maxHeight: `${image_bounds?.height}px`, 
                            maxWidth: `${image_bounds?.width}px`, 
                        }}
                        src={props.url} width={props.width} height={props.height} alt=""/>
                </figure>
            </div>
        </div>
    );
}