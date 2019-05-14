import React from "react";
import EventEmitter from "events";
import FolderView from "./FolderView";

interface FileSizeWorkerProps {
  emitter: EventEmitter;
}

interface FileSizeWorkerState {
  totalSize: number;
  entries: Array<Entry>;
}

export default class FileSizeWorker extends React.Component<
  FileSizeWorkerProps,
  FileSizeWorkerState
> {
  state = { totalSize: 0, entries: [] };
  startTime = 0;

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
    // if (data.t === "start") {
    //   this.startTime = Date.now();
    // } else if (data.t === "finish") {
    //   console.log(Date.now() - this.startTime);
    // } else if (data.t === "chunk") {
    //   const files = data.c;
    //   let chunkSize = 0;
    //   files.forEach(([pathComponents, size]) => {
    //     chunkSize += size;

    //     let ag = this.tree;
    //     pathComponents.forEach((component, i) => {
    //       if (i === pathComponents.length - 1) return;

    //       if (typeof ag[component] === "undefined") {
    //         ag[component] = {};
    //       }
    //       ag = ag[component];
    //     });

    //     const fileName = pathComponents[pathComponents.length - 1];
    //     ag[fileName] = size;
    //   });

    //   console.log(this.tree);
    //   const totalSize = this.state.totalSize + chunkSize;
    //   this.setState({ totalSize });
    // }
  };

  render() {
    const { totalSize, entries } = this.state;

    return (
      <>
        <h3>Total size: {totalSize}</h3>
        <FolderView entries={entries} />
      </>
    );
  }
}
