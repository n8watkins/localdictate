import React from "react";
import ReactDOM from "react-dom/client";

const root = ReactDOM.createRoot(document.getElementById("root") as HTMLElement);
const isPillView =
  new URLSearchParams(window.location.search).get("view") === "pill";

if (isPillView) {
  void import("./PillApp").then(({ default: PillApp }) => {
    root.render(
      <React.StrictMode>
        <PillApp />
      </React.StrictMode>,
    );
  });
} else {
  void import("./App").then(({ default: App }) => {
    root.render(
      <React.StrictMode>
        <App />
      </React.StrictMode>,
    );
  });
}
