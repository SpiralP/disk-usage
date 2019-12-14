import React from "react";
import WebSocketConnection from "./WebSocketConnection";
import MainView from "./MainView";

interface AppState {
  ws: WebSocket | null;
}

export default class App extends React.Component<{}, AppState> {
  state: AppState = { ws: null };

  render() {
    const { ws } = this.state;

    return (
      <div>
        <WebSocketConnection
          path="ws"
          onOpen={(ws) => {
            this.setState({ ws });
          }}
          onClose={() => {
            this.setState({ ws: null });
          }}
        />

        {ws ? <MainView ws={ws} /> : null}
      </div>
    );
  }
}
