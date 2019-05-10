import React from "react";
import ReactDOM from "react-dom";
import EventEmitter from "events";


function connectWebSocket(path: string): Promise<EventEmitter> {
  return new Promise((resolve, reject) => {
    const ws = new WebSocket(`ws://${location.host}/${path}`);

    ws.onerror = function error(err) {
      reject(err);
    };

    ws.onopen = function open() {
      ws.onopen = () => {};

      const ee = new EventEmitter();

      ws.onerror = (err) => {
        ee.emit("error", err);
        ee.removeAllListeners("send");
      };

      ws.onclose = () => {
        ee.emit("close");
        ee.removeAllListeners("send");
      };

      ws.onmessage = (response) => {
        const { data } = response;

        ee.emit("receive", data);
      };

      ee.on("send", (message: string) => {
        ws.send(message);
      });

      resolve(ee);
    };
  });
}

connectWebSocket("ws").then((emitter) => {
  console.log("connected");
  emitter.on("receive", (msg) => console.log("msg", msg));

  global.ag = emitter;
});

function App() {
  return <h1>hi world</h1>;
}

ReactDOM.render(<App />, document.getElementById("root"));
