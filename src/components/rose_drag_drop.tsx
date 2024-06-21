import React, { useEffect, useState } from "react";
import { resolveResource } from '@tauri-apps/api/path'
import { convertFileSrc } from "@tauri-apps/api/tauri";

type Props = {
    show: boolean
}

export default function RoseDragDropDialog(props: Props) {
    const [filePath, setFilePath] = useState<string>();

    useEffect(() => {
        resolveResource("../public/osaka.png").then((value) => {
            setFilePath(convertFileSrc(value));
        });
    }, []);

    if (!props.show) {
        return null;
    }

    return (
        <div className="select-none">
            <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-2/4 max-w-screen-sm">
                <div className="relative border-dashed border-2 border-goldyPink 
                dark:bg-goldyDarky bg-rose-100 text-white text-center rounded-lg">
                    <img src={filePath} alt="osaka" className="mx-auto w-36 rounded-full mt-2"></img>

                    <div className="bg-rose-500 dark:bg-transparent min-w-fit">
                        <h3 className="m-2 text-white dark:text-red-400 font-extrabold">Drag and Drop</h3>
                    </div>
                </div>
            </div>
        </div>
    );
}