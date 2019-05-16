import React from "react";
import EventEmitter from "events";
import {
  HTMLTable,
  Icon,
  ContextMenuTarget,
  Menu,
  Intent,
  Alert,
  MenuItem,
} from "@blueprintjs/core";
import { bytes } from "./helpers";

interface FolderViewWorkerProps {
  entries: Array<Entry>;
  onChangeDirectory: (entry: Entry) => void;
}

interface FolderViewWorkerState {
  deleteEntry?: Entry;
}

const NameColumnStyle = { width: "100%" };
const SizeColumnStyle: { textAlign: "right" } = { textAlign: "right" };

class ProgressBar extends React.Component<
  {
    value: number;
  },
  {}
> {
  render() {
    const { value, children } = this.props;

    return (
      <div
        style={{
          position: "relative",
        }}
      >
        <div
          style={{
            zIndex: 9,
            position: "relative",
          }}
        >
          {children}
        </div>
        <div
          style={{
            backgroundColor: "#137CBD1a",
            zIndex: 1,
            position: "absolute",
            width: `${value * 100}%`,
            height: "32px",
            top: "-6px",
            left: "-10px",
          }}
        />
      </div>
    );
  }
}

@ContextMenuTarget
class EntryRow extends React.Component<
  {
    entry: Entry;
    onDelete: () => void;
    onClick: () => void;
    totalSize: number;
  },
  {}
> {
  public renderContextMenu() {
    const { entry, onDelete } = this.props;
    return (
      <Menu>
        <MenuItem
          onClick={() => {
            onDelete();
          }}
          text="Delete"
        />
      </Menu>
    );
  }

  // <div>
  //           {/* <ProgressBar value={0.6} intent="primary" stripes animate /> */}
  //         </div>
  render() {
    const { entry, onClick, totalSize } = this.props;

    return (
      <tr
        onClick={() => {
          onClick();
        }}
      >
        <td style={NameColumnStyle}>
          <ProgressBar value={entry.size / totalSize}>
            <Icon
              iconSize={20}
              intent="primary"
              icon={entry.type === "directory" ? "folder-close" : "document"}
              style={{ paddingRight: "10px" }}
            />
            {entry.name}
          </ProgressBar>
        </td>
        <td
          style={SizeColumnStyle}
          title={`${entry.size.toLocaleString()} bytes`}
        >
          {bytes(entry.size)}
        </td>
      </tr>
    );
  }
}

export default class FolderViewWorker extends React.Component<
  FolderViewWorkerProps,
  FolderViewWorkerState
> {
  state: FolderViewWorkerState = {};

  render() {
    const { entries, onChangeDirectory } = this.props;
    const { deleteEntry } = this.state;

    // sort by size
    const sortedEntries = entries.slice(0).sort((left, right) => {
      const a = left.size;
      const b = right.size;

      // greater first
      if (a > b) return -1;
      if (a < b) return 1;

      // show directories first
      if (left.type === "directory" && right.type === "file") return -1;
      if (left.type === "file" && right.type === "directory") return 1;

      // a-z
      if (left.name < right.name) return -1;
      if (left.name > right.name) return 1;

      return 0;
    });

    return (
      <div style={{ paddingBottom: "16px" }}>
        <Alert
          isOpen={deleteEntry ? true : false}
          icon="trash"
          intent={Intent.DANGER}
          cancelButtonText="Cancel"
          confirmButtonText="Delete Forever"
          onConfirm={() => {
            console.warn("TODO delete", deleteEntry);
            this.setState({ deleteEntry: undefined });
          }}
          onCancel={() => {
            this.setState({ deleteEntry: undefined });
          }}
        >
          <p>
            Are you sure you want to delete{" "}
            <b>{deleteEntry ? deleteEntry.name : "<unknown>"}</b> forever?
          </p>
        </Alert>

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
              <EntryRow
                key={i}
                entry={entry}
                onDelete={() => {
                  this.setState({ deleteEntry: entry });
                }}
                onClick={() => {
                  if (entry.type === "directory") {
                    onChangeDirectory(entry);
                  }
                }}
                totalSize={entries
                  .map((entry) => entry.size)
                  .reduce((last, current) => last + current, 0)}
              />
            ))}
          </tbody>
        </HTMLTable>
      </div>
    );
  }
}
