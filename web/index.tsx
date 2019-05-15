import React from "react";
import ReactDOM from "react-dom";
import "@blueprintjs/icons/lib/css/blueprint-icons.css";
import "@blueprintjs/core/lib/css/blueprint.css";

// import App from "./App";
// ReactDOM.render(<App />, document.getElementById("root"));

import EventEmitter from "events";
import FileSizeWorker from "./FileSizeWorker";
const data = require("./data.json");
const emitter = new EventEmitter();

ReactDOM.render(
  <FileSizeWorker emitter={emitter} />,
  document.getElementById("root")
);

async function wait(sec: number) {
  return new Promise((resolve) => setTimeout(resolve, sec * 1000));
}
(async () => {
  for (let index = 0; index < data.length; index++) {
    const message = data[index];
    await wait(0.5);
    emitter.emit("receive", message);
  }
})();
emitter.on("send", console.log);
