import React from "react";
import ReactDOM from "react-dom";
import App from "./App";

ReactDOM.render(<App />, document.getElementById("root"));

// import EventEmitter from "events";
// import FileSizeWorker from "./FileSizeWorker";

// const chunk1 = require("./chunk1.json");
// const chunk2 = require("./chunk2.json");
// const chunk3 = require("./chunk3.json");

// const emitter = new EventEmitter();
// emitter.on("send", () => {
//   emitter.emit("receive", JSON.stringify({ t: "start" }));

//   setTimeout(() => {
//     emitter.emit("receive", JSON.stringify(chunk1));
//     setTimeout(() => {
//       emitter.emit("receive", JSON.stringify(chunk2));
//       setTimeout(() => {
//         emitter.emit("receive", JSON.stringify(chunk3));
//         emitter.emit("receive", JSON.stringify({ t: "finish" }));
//       }, 1000);
//     }, 1000);
//   }, 300);
// });

// ReactDOM.render(
//   <FileSizeWorker emitter={emitter} />,
//   document.getElementById("root")
// );
