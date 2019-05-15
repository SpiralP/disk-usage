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

    const sortedEntries = entries.slice(0).sort((left, right) => {
      if (left.type === "directory" && right.type === "file") return -1;
      if (left.type === "file" && right.type === "directory") return 1;
      return left.name < right.name ? -1 : 1;
    });

    return (
      <>
        {sortedEntries.map((entry) => {
          if (entry.type === "file") {
            return (
              <h3 key={entry.name}>
                {entry.name}:{entry.size}
              </h3>
            );
          } else if (entry.type === "directory") {
            return (
              <h2 key={entry.name}>
                {entry.name}:{entry.size}
              </h2>
            );
          }
        })}
      </>
    );
  }
}
