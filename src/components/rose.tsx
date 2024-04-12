import React from "react";

type Props = {
    image_loading: boolean, 
}

export default function Rose(props: Props) {

    return (
        <div className="select-none">
            {
                props.image_loading === true ? 
                    <div className="relative py-3">
                        <h1 className="flex items-center justify-center font-dosis font-medium text-white text-5xl">🌹</h1>
                    </div> : 
                    <div className="cursor-pointer relative py-3">
                        <h1 className="flex items-center justify-center font-dosis font-medium text-white text-5xl">🌹</h1>
                    </div>
            }

            {/* TODO: Add a good light mode text colour to this. */}
            {
                props.image_loading === true ? 
                    <h3 className="dark:text-red-400 font-semibold">Loading...</h3> : 
                    <h3 className="dark:text-red-400 font-semibold">
                        click me to select an image
                    </h3>
            }

        </div>
    );
}