import React from "react";
import ReactDOM from "react-dom/client";

import "./index.css";
import App from "./App.tsx";

import { initTheme } from "../cirrus/tauri_frontend";

initTheme();

ReactDOM.createRoot(document.getElementById("root")!).render(
    <React.StrictMode>
        <App />
    </React.StrictMode>,
);