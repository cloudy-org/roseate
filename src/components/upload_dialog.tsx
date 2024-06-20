import React, { useEffect, useState } from "react";
import { resolveResource } from '@tauri-apps/api/path'
import { convertFileSrc } from "@tauri-apps/api/tauri";

type Props = {
    show: boolean, 
}

export default function RoseDialog(props: Props) {
    const [filePath, setFilePath] = useState<string>();

    useEffect(() => {
        resolveResource("resources/osaka.png").then((value) => {
            setFilePath(convertFileSrc(value));
        });
    }, []);

    if (!props.show) {
        return null;
    }

    return (
        <div className="select-none">
            <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-2/4 max-w-screen-sm">
                <div className="relative dark:bg-goldyDarky bg-goldyPink text-white text-center rounded-lg">
                    <img src={filePath} alt="osaka" className="mx-auto"></img>

                    <strong>Drop and Drop</strong>
                </div>
            </div>
        </div>
    );
}