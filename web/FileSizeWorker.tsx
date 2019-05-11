import React from "react";
import EventEmitter from "events";

interface FileSizeWorkerProps {
  emitter: EventEmitter;
}

interface FileSizeWorkerState {
  totalSize: number;
}

export default class FileSizeWorker extends React.Component<
  FileSizeWorkerProps,
  FileSizeWorkerState
> {
  state = { totalSize: 0 };
  startTime = 0;

  componentDidMount() {
    const { emitter } = this.props;
    emitter.on("receive", this.receiver);

    emitter.emit("send", "D:\\Programs\\");
  }

  componentWillUnmount() {
    const { emitter } = this.props;
    emitter.off("receive", this.receiver);
  }

  receiver = (data: string) => {
    const parsed = JSON.parse(data) as FileSizeStatus;
    if (parsed.t === "start") {
      this.startTime = Date.now();
    } else if (parsed.t === "finish") {
      console.log(Date.now() - this.startTime);
    } else if (parsed.t === "chunk") {
      const files = parsed.c;

      let chunkSize = 0;
      files.forEach(([path, size]) => {
        chunkSize += size;
      });

      const totalSize = this.state.totalSize + chunkSize;
      this.setState({ totalSize });
    }
  };

  render() {
    const { totalSize } = this.state;

    return <h3>Total size: {totalSize}</h3>;
  }
}

type FileSize = [string, number];

interface FileSizeStatusStart {
  t: "start";
  c: undefined;
}
interface FileSizeStatusChunk {
  t: "chunk";
  c: Array<FileSize>;
}
interface FileSizeStatusFinish {
  t: "finish";
  c: undefined;
}

type FileSizeStatus =
  | FileSizeStatusStart
  | FileSizeStatusChunk
  | FileSizeStatusFinish;