import React from "react";
import {
  HTMLTable,
  Icon,
  ContextMenuTarget,
  Menu,
  Intent,
  Alert,
  MenuItem,
  Spinner,
} from "@blueprintjs/core";
import { bytes, time } from "./helpers";
import ReactDOM from "react-dom";

class ProgressBar extends React.Component<
  {
    value: number;
  },
  {}
> {
  render() {
    const { value, children } = this.props;

    const percent = isNaN(value) ? 0 : Math.max(0, Math.min(1, value));

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
            width: `${percent * 100}%`,
            height: "32px",
            top: "-6px",
            left: "-10px",
          }}
        />
      </div>
    );
  }
}

const NameColumnStyle = { width: "100%" };
const SizeColumnStyle: { textAlign: "right" } = { textAlign: "right" };

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
            <div
              style={{
                paddingRight: "10px",
                display: "inline-block",
                verticalAlign: "text-bottom",
              }}
            >
              {entry.type === "directory" && entry.updating ? (
                <Spinner size={20} intent="primary" tagName="stop" />
              ) : (
                <Icon
                  iconSize={20}
                  intent="primary"
                  icon={
                    entry.type === "directory" ? "folder-close" : "document"
                  }
                />
              )}
            </div>
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

interface FolderViewProps {
  entries: Array<Entry>;
  onChangeDirectory: (entry: Entry) => void;
  onDelete: (entry: Entry) => void;
}

interface FolderViewState {
  deleteEntry?: Entry;
  numberOfShownEntries: number;
}

export default class FolderView extends React.Component<
  FolderViewProps,
  FolderViewState
> {
  state: FolderViewState = {
    numberOfShownEntries: 100,
  };

  tableRef: React.RefObject<HTMLTable> = React.createRef();

  componentDidMount() {
    window.addEventListener("scroll", this.handleScroll);
    this.handleScroll();
  }

  componentWillUnmount() {
    window.removeEventListener("scroll", this.handleScroll);
  }

  isScrolledPastEnd() {
    const last = this.tableRef.current;
    if (!last) {
      return false;
    }

    const el = ReactDOM.findDOMNode(last);
    if (el instanceof HTMLElement) {
      const elTop = el.offsetTop;
      const elHeight = el.offsetHeight;
      const elBottomPos = elTop + elHeight;

      const scrollTop = window.scrollY;
      const windowHeight = window.innerHeight;
      const pageBottomPos = scrollTop + windowHeight;

      if (elBottomPos < pageBottomPos) {
        return true;
      }
    }

    return false;
  }

  handleScroll = () => {
    const { entries } = this.props;
    const { numberOfShownEntries } = this.state;

    if (entries.length <= numberOfShownEntries) {
      return;
    }

    if (this.isScrolledPastEnd()) {
      this.setState({
        numberOfShownEntries: numberOfShownEntries + 100,
      });
    }
  };

  componentDidUpdate() {
    this.handleScroll();
  }

  render() {
    return time("FolderView render", () => {
      const { entries, onChangeDirectory, onDelete } = this.props;
      const { deleteEntry, numberOfShownEntries } = this.state;

      const totalSize = entries
        .map((entry) => entry.size)
        .reduce((last, current) => last + current, 0);

      // sort by size
      const sortedEntries = entries
        .slice(0)
        .sort((left, right) => {
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
        })
        .slice(0, numberOfShownEntries);

      const sortedEntriesElements = sortedEntries.map((entry, i) => (
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
          totalSize={totalSize}
        />
      ));

      return (
        <div style={{ paddingBottom: "16px" }}>
          <Alert
            isOpen={deleteEntry ? true : false}
            icon="trash"
            intent={Intent.DANGER}
            cancelButtonText="Cancel"
            confirmButtonText="Delete Forever"
            onConfirm={() => {
              this.setState({ deleteEntry: undefined });

              console.info("delete", deleteEntry);

              if (deleteEntry != null) {
                onDelete(deleteEntry);
              }
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
            ref={this.tableRef}
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
            <tbody>{sortedEntriesElements}</tbody>
          </HTMLTable>
        </div>
      );
    });
  }
}
