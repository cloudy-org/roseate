import React from "react";
import { useEffect, useState, useRef } from "react";

import { invoke, convertFileSrc } from "@tauri-apps/api/tauri";

import { listen } from '@tauri-apps/api/event';

import { initWindow } from "../cirrus/tauri_typescript";

import Rose from "./components/rose";
import RoseImage from "./components/image";

export type Image = {
    url: string,
    width: number,
    height: number
}

export default function Home() {
    const image_load_called = useRef(false);

    const [image_loading, setImageLoading] = useState(false);
    const [no_image_available, setNoImageAvailable] = useState(false);
    const [listen_running, setListenRunning] = useState(false);

    const [image_opacity, setImageOpacity] = useState<number>(0);
    const [image_display, setImageDisplay] = useState<string>("hidden");

    const [image, setImage] = useState<Image | null>(null);

    useEffect(() => initWindow(), []);

    function load_image() {
        if (image_load_called.current == false && image == null && no_image_available == false) {
            image_load_called.current = true;
            setImageLoading(true);

            console.debug("Attempting to load image from backend...");

            invoke<[string, [number, number]] | null>("get_image").then(
                image_data => {
                    if (image_data !== null) {
                        const path = image_data[0];
                        const dimensions = image_data[1];

                        const url = convertFileSrc(path);

                        console.debug("Setting image...");
                        setImage({
                            url: url,
                            width: dimensions[0],
                            height: dimensions[1]
                        });

                        // WHY THE FUCK DOES A 20 MILLISECOND TIMEOUT FIX MY ANIMATION PROBLEMS!!!
                        setTimeout(() => setImageDisplay("unset"), 20);

                        setTimeout(() => {
                            console.debug("Displaying image...");
                            setImageLoading(false);
                            setImageOpacity(1);
                        }, 50);
                    } else {
                        console.debug("No image found in backend.");
                        setNoImageAvailable(true);
                        setImageLoading(false);
                    }
                }
            ).catch(console.error);
        }
    }

    useEffect(() => load_image());

    useEffect(() => {
        const unlisten = listen('tauri://file-drop', event => {
            image_load_called.current = false;

            console.log(event.payload);

            invoke("set_image_drag_drop", {path: event.payload})
            load_image()
        });

        return () => {
            console.log("return");
            unlisten.then(f => f());
        };
      }, []);
    
    return (
        <div className="relative">
            <div className="flex items-center justify-center h-screen">
                {
                    image === null ? 
                        <div onClick={() => {
                            if (!image_loading) {
                                invoke("select_image").then(() => {
                                    image_load_called.current = false;
                                    setNoImageAvailable(false);
                                    load_image();
                                });
                            }
                        }}>
                            <Rose image_loading={image_loading}></Rose>
                        </div> : 
                        <div className="hidden opacity-0 transition-opacity duration-1000" style={{display: image_display, opacity: image_opacity}}>
                            <RoseImage image={image}></RoseImage>
                        </div>
                }
            </div>
        </div>
    );
}