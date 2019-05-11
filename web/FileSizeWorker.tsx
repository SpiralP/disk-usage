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

  componentDidMount() {
    const { emitter } = this.props;
    emitter.on("receive", this.receiver);

    emitter.emit("send", "."); // TODO send root path
  }

  componentWillUnmount() {
    const { emitter } = this.props;
    emitter.off("receive", this.receiver);
  }

  receiver = (data: string) => {
    const [path, size] = JSON.parse(data) as FileSizeMessage;

    const totalSize = this.state.totalSize + size;
    this.setState({ totalSize });
  };

  render() {
    const { totalSize } = this.state;

    return <h3>Total size: {totalSize}</h3>;
  }
}

type FileSizeMessage = [string, number];
