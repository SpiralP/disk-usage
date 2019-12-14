import React from "react";

interface WebSocketConnectionProps {
  path: string;
  onOpen?: (ws: WebSocket, event: Event) => void;
  onError?: (event: Event) => void;
  onClose?: (event: CloseEvent) => void;
  onMessage?: (event: MessageEvent) => void;
}

interface WebSocketConnectionState {
  state: "connecting" | "open" | "closed";
}

export default class WebSocketConnection extends React.Component<
  WebSocketConnectionProps,
  WebSocketConnectionState
> {
  ws?: WebSocket;

  state: WebSocketConnectionState = {
    state: "connecting",
  };

  componentDidMount() {
    const { path, onOpen, onClose, onError, onMessage } = this.props;

    try {
      const ws = new WebSocket(`ws://${location.host}/${path}`);

      ws.onopen = (event) => {
        if (onOpen) {
          onOpen(ws, event);
        }
        this.setState({ state: "open" });
      };

      ws.onerror = (event) => {
        if (onError) {
          onError(event);
        }
      };

      ws.onclose = (event) => {
        if (onClose) {
          onClose(event);
        }
        this.setState({ state: "closed" });
      };

      ws.onmessage = (event) => {
        if (onMessage) {
          onMessage(event);
        }
      };
    } catch (e) {
      //
    }
  }

  componentWillUnmount() {
    if (this.ws) {
      this.ws.close();
      this.ws = undefined;
    }
  }

  render() {
    const { state } = this.state;

    if (state === "open") {
      return null;
    } else {
      return (
        <h1>
          {state === "connecting"
            ? "connecting"
            : state === "closed"
            ? "closed"
            : null}
        </h1>
      );
    }
  }
}
