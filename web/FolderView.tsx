import React from "react";
import EventEmitter from "events";
import { HTMLTable, Icon } from "@blueprintjs/core";
import { bytes } from "./helpers";

interface FolderViewWorkerProps {
  entries: Array<Entry>;
}

interface FolderViewWorkerState {}

const NameColumnStyle = { width: "100%" };
const SizeColumnStyle: { textAlign: "right" } = { textAlign: "right" };

export default class FolderViewWorker extends React.Component<
  FolderViewWorkerProps,
  FolderViewWorkerState
> {
  render() {
    const { entries } = this.props;

    // sort by size
    const sortedEntries = entries.slice(0).sort((left, right) => {
      const a = left.size;
      const b = right.size;

      return a > b ? -1 : a === b ? 0 : 1;
    });

    return (
      <div style={{ paddingBottom: "16px" }}>
        <HTMLTable
          bordered
          condensed
          interactive
          striped
          style={{ width: "100%", whiteSpace: "nowrap" }}
        >
          <thead>
            <tr>
              <th style={NameColumnStyle}>Name</th>
              <th style={SizeColumnStyle}>Size</th>
            </tr>
          </thead>
          <tbody>
            {sortedEntries.map((entry, i) => (
              <tr key={i}>
                <td style={NameColumnStyle}>
                  <Icon
                    iconSize={20}
                    intent="primary"
                    icon={
                      entry.type === "directory" ? "folder-close" : "document"
                    }
                    style={{ paddingRight: "10px" }}
                  />
                  {entry.name}
                </td>
                <td style={SizeColumnStyle} title={`${entry.size} bytes`}>
                  {bytes(entry.size)}
                </td>
              </tr>
            ))}
          </tbody>
        </HTMLTable>
      </div>
    );
  }
}
