import dynamic from "next/dynamic";

const DynamicRoseImage = dynamic(() => import("./image"), {
    ssr: false,
});

export default DynamicRoseImage;