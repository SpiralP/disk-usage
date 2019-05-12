import React from "react";
import ReactDOM from "react-dom";
import App from "./App";
ReactDOM.render(<App />, document.getElementById("root"));

// import EventEmitter from "events";
// import FileSizeWorker from "./FileSizeWorker";

// const chunk1 = require("./chunk1.json");

// const emitter = new EventEmitter();
// emitter.on("send", () => {
//   emitter.emit("receive", { t: "start" });

//   setTimeout(() => {
//     emitter.emit("receive", chunk1);
//     setTimeout(() => {
//       // emitter.emit("receive", chunk2);
//       setTimeout(() => {
//         // emitter.emit("receive", chunk3);
//         emitter.emit("receive", { t: "finish" });
//       }, 1000);
//     }, 1000);
//   }, 500);
// });

// ReactDOM.render(
//   <FileSizeWorker emitter={emitter} />,
//   document.getElementById("root")
// );
