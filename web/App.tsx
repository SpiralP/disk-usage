import React from "react";
import EventEmitter from "events";
import FileSizeWorker from "./FileSizeWorker";

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

      ws.binaryType = "arraybuffer";
      ws.onmessage = (response) => {
        const { data } = response;

        // if (typeof data === "string") {
        const parsed = JSON.parse(data);
        // } else if (data instanceof ArrayBuffer) {
        // const parsed = messagePack.decode(Buffer.from(data));
        // }

        ee.emit("receive", parsed);
      };

      ee.on("send", (message: string) => {
        ws.send(message);
      });

      resolve(ee);
    };
  });
}

type AppState =
  | {
      state: "connecting" | "closed";
    }
  | {
      state: "error";
      error: string;
    }
  | {
      state: "connected";
      emitter: EventEmitter;
    };

export default class App extends React.Component<{}, AppState> {
  constructor(props: {}) {
    super(props);

    this.state = {
      state: "connecting",
    };
  }

  componentDidMount() {
    connectWebSocket("ws")
      .then((ee) => {
        ee.on("close", () => {
          this.setState({ state: "closed" });

          window.close();
        });

        ee.on("error", (err) => {
          this.setState({ state: "error", error: err });
        });

        this.setState({ state: "connected", emitter: ee });
      })
      .catch((err) => {
        this.setState({ state: "error", error: err });
      });
  }

  componentWillUnmount() {}

  render() {
    const state = this.state;

    return (
      <div>
        {state.state === "connected" ? (
          <FileSizeWorker emitter={state.emitter} />
        ) : (
          <h2>
            {state.state === "connecting"
              ? "connecting"
              : state.state === "error"
              ? `error: ${state.error}`
              : state.state === "closed"
              ? "closed"
              : null}
          </h2>
        )}
      </div>
    );
  }
}
