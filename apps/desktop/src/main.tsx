import React from "react";
import ReactDOM from "react-dom/client";
import { App } from "./app/app";
import { installConsoleBridge } from "@feature/log";

// 前端 console 日志接入统一日志面板（source=react）。
installConsoleBridge();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
