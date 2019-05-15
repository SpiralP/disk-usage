import React from "react";
import EventEmitter from "events";
import FolderView from "./FolderView";

interface FileSizeWorkerProps {
  emitter: EventEmitter;
}

interface FileSizeWorkerState {
  path: Array<String>;
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
    const { entries } = this.state;

    return (
      <>
        <FolderView entries={entries} />
        <h4>
          Total size:{" "}
          {entries
            .map((entry) => entry.size)
            .reduce((last, current) => last + current, 0)}
        </h4>
      </>
    );
  }
}
