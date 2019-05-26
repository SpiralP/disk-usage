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
  free: number;
}

export default class FileSizeWorker extends React.Component<
  FileSizeWorkerProps,
  FileSizeWorkerState
> {
  state: FileSizeWorkerState = { path: [], entries: [], free: 0 };

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
    if (data.type === "directoryChange") {
      const { path, entries, free } = data;
      this.setState({
        path,
        entries,
        free,
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
                this.send({ type: "changeDirectory", path: path.slice(0, i) });
              },
            }))}
          />
        </div>

        <FolderView
          entries={entries}
          onChangeDirectory={(entry) => {
            this.send({ type: "changeDirectory", path: [...path, entry.name] });
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
  }
}
