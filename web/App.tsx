import React from "react";
import MainView from "./MainView";

interface AppProps {
  path: string;
}

type AppState =
  | {
      state: "connecting";
    }
  | {
      state: "closed";
    }
  | {
      state: "open";
      ws: WebSocket;
    };

export default class App extends React.Component<AppProps, AppState> {
  state: AppState = { state: "connecting" };

  componentDidMount() {
    const { path } = this.props;

    const ws = new WebSocket(`ws://${location.host}/${path}`);

    ws.onopen = () => {
      this.setState({ state: "open", ws });
    };

    ws.onerror = () => {};

    ws.onclose = () => {
      this.setState({ state: "closed" });
    };

    ws.onmessage = () => {};
  }

  render() {
    const { state } = this.state;

    if (this.state.state === "open") {
      return <MainView ws={this.state.ws} />;
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
