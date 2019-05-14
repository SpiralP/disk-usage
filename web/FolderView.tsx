import React from "react";
import EventEmitter from "events";

interface FolderViewWorkerProps {
  entries: Array<Entry>;
}

interface FolderViewWorkerState {}

export default class FolderViewWorker extends React.Component<
  FolderViewWorkerProps,
  FolderViewWorkerState
> {
  render() {
    const { entries } = this.props;

    return (
      <>
        {entries.map((entry) => {
          if (entry.type === "file") {
            return <h3>{entry.name}</h3>;
          } else if (entry.type === "directory") {
            return <h2>{entry.name}</h2>;
          }
        })}
      </>
    );
  }
}
