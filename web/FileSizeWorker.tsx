import React from "react";
import EventEmitter from "events";
import FolderView from "./FolderView";
import { Breadcrumbs, Divider, Text } from "@blueprintjs/core";
import { bytes } from "./helpers";

interface FileSizeWorkerProps {
  emitter: EventEmitter;
}

interface FileSizeWorkerState {
  path: Array<string>;
  entries: Array<Entry>;
}

export default class FileSizeWorker extends React.Component<
  FileSizeWorkerProps,
  FileSizeWorkerState
> {
  state: FileSizeWorkerState = { path: [], entries: [] };

  componentDidMount() {
    const { emitter } = this.props;
    emitter.on("receive", this.receiver);

    // @ts-ignore
    global.ag = emitter;
  }

  componentWillUnmount() {
    const { emitter } = this.props;
    emitter.off("receive", this.receiver);
  }

  send(msg: ControlMessage) {
    const { emitter } = this.props;
    emitter.emit("send", JSON.stringify(msg));
  }

  receiver = (data: EventMessage) => {
    console.log(data);

    if (data.type === "directoryChange") {
      const { path, entries } = data;
      this.setState({
        path,
        entries,
      });
    } else if (data.type === "sizeUpdate") {
      const newEntry = data.entry;
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
  };

  render() {
    const { path, entries } = this.state;

    return (
      <div>
        <Breadcrumbs
          items={[".", ...path].map((name, i) => ({
            text: name,
            icon: "folder-close",
            onClick: () => {
              this.send({ type: "changeDirectory", path: path.slice(0, i) });
            },
          }))}
        />

        <FolderView entries={entries} />

        <div style={{ display: "flex" }}>
          <Text>{`${entries.length} items`}</Text>
          <Divider />
          <Text>
            {`Total size: ${bytes(
              entries
                .map((entry) => entry.size)
                .reduce((last, current) => last + current, 0)
            )}`}
          </Text>
        </div>
      </div>
    );
  }
}
