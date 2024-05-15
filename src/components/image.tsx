import React from "react";
import { useEffect, useRef, useState, WheelEventHandler } from "react";

import { appWindow, PhysicalSize } from "@tauri-apps/api/window";
import { Event } from "@tauri-apps/api/event";

import { Image } from "../App";

const IMAGE_PADDING = 8;

type Dimensions = {
    width: number,
    height: number
}

type Props = {
    image: Image
}

export default function RoseImage(props: Props) {
    const last_zoom_event = useRef(Date.now());

    const [zoom_position, setZoomPosition] = useState({ x: 0, y: 0, scale: 1 });
    const [image_bounds, setImageBounds] = useState<Dimensions>({width: props.image.width, height: props.image.height});
    const [image_will_change, warnImageChange] = useState<string>("unset");
    const [window_size, setWindowSize] = useState<Dimensions | null>(null);

    const on_scroll: WheelEventHandler<HTMLDivElement> = (event) => {
        last_zoom_event.current = Date.now();
        warnImageChange("transform");

        const delta = event.deltaY * -0.005 * (zoom_position.scale / 2);
        const new_scale = zoom_position.scale + delta;

        const ratio = 1 - new_scale / zoom_position.scale;

        const rect = event.currentTarget.getBoundingClientRect();

        setZoomPosition({
            scale: new_scale,
            x: zoom_position.x + ((event.clientX - rect.left) - zoom_position.x) * ratio,
            y: zoom_position.y + ((event.clientY - rect.top) - zoom_position.y) * ratio
        });
    };

    // Set's "will-change" css attribute back to "unset" and zoom scale 
    // to 1 if exceeded the zoom out boundaries when the user is no longer zooming in/out.
    useEffect(() => {
        const id = setInterval(() => {
            if (last_zoom_event.current + 700 < Date.now() && image_will_change !== "unset") {
                warnImageChange("unset");

                if (zoom_position.scale < 1) {
                    setZoomPosition({ x: 0, y: 0, scale: 1 });
                }
            }
        }, 200);

        return () => clearInterval(id);
    }, [zoom_position, image_will_change]);

    // Set window size on first time application launch.
    useEffect(() => {
        appWindow.innerSize().then(
            (size) => setWindowSize({width: size.width, height: size.height})
        );
    }, []);

    // Sets the window size on window resize event.
    useEffect(() => {
        const unlisten = appWindow.listen(
            "tauri://resize", 
            (event: Event<PhysicalSize>) => {
                const size = event.payload;

                if (window_size == null || size.width !== window_size.width || size.height !== window_size.height) {
                    setWindowSize({width: size.width, height: size.height});
                }
            }
        );

        return () => {
            unlisten.then(f => f());
        };
    }, [window_size]);

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
            <div className="transition-transform duration-500 delay-0"
                style={{
                    padding: IMAGE_PADDING, 

                    transformOrigin: "0 0", 
                    transform: `translate(${zoom_position.x}px, ${zoom_position.y}px) scale(${zoom_position.scale})`,
                    willChange: image_will_change
                }}>

                <figure className="rounded-lg size-max overflow-hidden">
                    <img className="w-auto h-auto transition-all duration-1000 delay-500"
                        style={{
                            maxHeight: `${image_bounds?.height}px`, 
                            maxWidth: `${image_bounds?.width}px`, 
                        }}
                        src={props.image.url} width={props.image.width} height={props.image.height} alt=""/>
                </figure>
            </div>
        </div>
    );
}