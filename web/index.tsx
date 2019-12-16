import "@blueprintjs/icons/lib/css/blueprint-icons.css";
import "@blueprintjs/core/lib/css/blueprint.css";
import React from "react";
import ReactDOM from "react-dom";
import App from "./App";
import { Toaster } from "@blueprintjs/core";

const toaster = Toaster.create({ position: "top-right" });
ReactDOM.render(
  <App path="ws" toaster={toaster} />,
  document.getElementById("root")
);
