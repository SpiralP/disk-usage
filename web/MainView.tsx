import React from "react";
import FolderView from "./FolderView";
import { Breadcrumbs, Divider, Text } from "@blueprintjs/core";
import { bytes, time } from "./helpers";

interface MainViewProps {
  ws: WebSocket;
}

interface MainViewState {
  path: Array<string>;
  entries: Array<Entry>;
  free: number;
}

export default class MainView extends React.Component<
  MainViewProps,
  MainViewState
> {
  state: MainViewState = { path: [], entries: [], free: 0 };

  componentDidMount() {
    const { ws } = this.props;
    ws.addEventListener("message", this.onMessage);

    // get initial current directory entries
    this.send({ type: "changeDirectory", path: [] });
  }

  componentWillUnmount() {
    const { ws } = this.props;
    ws.removeEventListener("message", this.onMessage);
  }

  send(msg: ControlMessage) {
    const { ws } = this.props;
    ws.send(JSON.stringify(msg));
  }

  onMessage = (event: MessageEvent) => {
    const { data } = event;

    // if (typeof data === "string") {

    const parsed = time("json parse", () => {
      return JSON.parse(data);
    });
    // } else if (data instanceof ArrayBuffer) {
    // const parsed = messagePack.decode(Buffer.from(data));
    // }

    time(`receive ${parsed.type}`, () => {
      this.receive(parsed);
    });
  };

  receive(data: EventMessage) {
    if (data.type === "directoryChange") {
      const { path, entries, free } = data;

      this.setState({
        path,
        entries,
        free,
      });
    } else if (data.type === "sizeUpdate") {
      const { entry: newEntry } = data;

      this.setState({
        entries: this.state.entries.map((entry) => {
          if (entry.name === newEntry.name) {
            return { ...entry, ...newEntry };
          } else {
            return entry;
          }
        }),
      });
    }
  }

  render() {
    return time("MainView render", () => {
      const { path, entries, free } = this.state;

      const totalSize = entries
        .map((entry) => entry.size)
        .reduce((last, current) => last + current, 0);

      return (
        <div>
          <div style={{ paddingLeft: "16px" }}>
            <Breadcrumbs
              items={["\u2022", ...path].map((name, i) => ({
                text: name,
                icon: "folder-close",
                onClick: () => {
                  this.send({
                    type: "changeDirectory",
                    path: path.slice(0, i),
                  });
                },
              }))}
            />
          </div>

          <FolderView
            key={path.join("/")}
            entries={entries}
            onChangeDirectory={(entry) => {
              this.send({
                type: "changeDirectory",
                path: [...path, entry.name],
              });
            }}
            onDelete={(entry) => {
              this.send({ type: "delete", path: [...path, entry.name] });
            }}
          />

          <div style={{ display: "flex" }}>
            <h4>{`${entries.length} items`}</h4>
            <Divider />
            <h4 title={`${totalSize.toLocaleString()} bytes`}>
              {`Total size: ${bytes(totalSize)}`}
            </h4>
            <Divider />
            <h4 title={`${free.toLocaleString()} bytes`}>
              {`Free space: ${bytes(free)}`}
            </h4>
          </div>
        </div>
      );
    });
  }
}
