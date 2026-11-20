import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { editionName } from "./lib/edition";
import "./styles/globals.css";

document.title = editionName();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
