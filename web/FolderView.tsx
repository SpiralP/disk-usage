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
}

interface FolderViewWorkerState {}

const NameColumnStyle = { width: "100%" };
const SizeColumnStyle: { textAlign: "right" } = { textAlign: "right" };

@ContextMenuTarget
class EntryRow extends React.Component<
  { entry: Entry },
  { isDelete: boolean }
> {
  state: { isDelete: boolean } = { isDelete: false };

  public renderContextMenu() {
    const { entry } = this.props;
    return (
      <Menu>
        <MenuItem
          onClick={() => {
            this.setState({ isDelete: true });
          }}
          text="Delete"
        />
      </Menu>
    );
  }

  render() {
    const { isDelete } = this.state;
    const { entry } = this.props;

    return (
      <tr>
        <Alert
          icon="trash"
          intent={Intent.DANGER}
          cancelButtonText="Cancel"
          confirmButtonText="Delete Forever"
          onConfirm={() => {
            console.warn("TODO delete", entry);
            this.setState({ isDelete: false });
          }}
          onCancel={() => {
            this.setState({ isDelete: false });
          }}
          isOpen={isDelete}
        >
          <p>
            Are you sure you want to delete <b>{entry.name}</b> forever?
          </p>
        </Alert>
        <td style={NameColumnStyle}>
          <Icon
            iconSize={20}
            intent="primary"
            icon={entry.type === "directory" ? "folder-close" : "document"}
            style={{ paddingRight: "10px" }}
          />
          {entry.name}
        </td>
        <td style={SizeColumnStyle} title={`${entry.size} bytes`}>
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
              <EntryRow key={i} entry={entry} />
            ))}
          </tbody>
        </HTMLTable>
      </div>
    );
  }
}
