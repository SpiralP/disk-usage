import React from "react";
import MainView from "./MainView";
import { IToaster } from "@blueprintjs/core";

interface AppProps {
  path: string;
  toaster: IToaster;
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
      // this.setState({ state: "closed" });
      this.props.toaster.show({
        message: "websocket closed",
        intent: "danger",
        timeout: 10000,
      });
    };

    ws.onmessage = () => {};
  }

  render() {
    const { toaster } = this.props;
    const { state } = this.state;

    if (this.state.state === "open") {
      return <MainView ws={this.state.ws} toaster={toaster} />;
    } else {
      // TODO maybe don't remove the MainView when closed!
      // but instead show a status message
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
