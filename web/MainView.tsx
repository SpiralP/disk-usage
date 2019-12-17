import React from "react";
import FolderView from "./FolderView";
import {
  Breadcrumbs,
  Divider,
  Text,
  IToaster,
  Breadcrumb,
  Icon,
  Spinner,
} from "@blueprintjs/core";
import { bytes, time } from "./helpers";

interface MainViewProps {
  ws: WebSocket;
  toaster: IToaster;
}

interface MainViewState {
  currentDirectory?: EntryDirectory;
  entries: Array<Entry>;
  breadcrumbEntries: Array<Entry>;
  free: number;
}

export default class MainView extends React.Component<
  MainViewProps,
  MainViewState
> {
  state: MainViewState = {
    currentDirectory: undefined,
    entries: [],
    breadcrumbEntries: [],
    free: 0,
  };

  componentDidMount() {
    const { ws } = this.props;
    ws.addEventListener("message", this.onMessage);

    // get initial current directory entries
    this.send({ type: "changeDirectory", path: [] });
  }

  componentWillUnmount() {
    const { ws } = this.props;
    ws.removeEventListener("message", this.onMessage);
  }

  send(msg: ControlMessage) {
    const { ws } = this.props;
    ws.send(JSON.stringify(msg));
  }

  onMessage = (event: MessageEvent) => {
    const { data } = event;

    // if (typeof data === "string") {

    const parsed = time("json parse", () => {
      return JSON.parse(data);
    });
    // } else if (data instanceof ArrayBuffer) {
    // const parsed = messagePack.decode(Buffer.from(data));
    // }

    time(`receive ${parsed.type}`, () => {
      this.receive(parsed);
    });
  };

  receive(data: EventMessage) {
    if (data.type === "directoryChange") {
      const { currentDirectory, entries, breadcrumbEntries, free } = data;

      this.setState({
        currentDirectory,
        entries,
        breadcrumbEntries,
        free,
      });
    } else if (data.type === "sizeUpdate") {
      const { entry } = data;
      // if (
      //   entry.path.length === 0 &&
      //   entry.type === "directory" &&
      //   !entry.updating
      // ) {
      //   this.props.toaster.show({
      //     message: "finished scanning",
      //     intent: "success",
      //     timeout: 3000,
      //   });
      //   return;
      // }

      this.setState({
        entries: this.state.entries.map((oldEntry) => {
          if (oldEntry.path.join("/") === entry.path.join("/")) {
            return entry;
          } else {
            return oldEntry;
          }
        }),
        breadcrumbEntries: this.state.breadcrumbEntries.map((oldEntry) => {
          if (oldEntry.path.join("/") === entry.path.join("/")) {
            return entry;
          } else {
            return oldEntry;
          }
        }),
      });
    }
  }

  render() {
    return time("MainView render", () => {
      const { currentDirectory, entries, breadcrumbEntries, free } = this.state;

      if (!currentDirectory || entries.length === 0) {
        return <div> loading </div>;
      }

      const totalSize = entries
        .map((entry) => entry.size)
        .reduce((last, current) => last + current, 0);

      return (
        <div>
          <div style={{ paddingLeft: "16px" }}>
            <Breadcrumbs
              key={"Breadcrumbs-" + currentDirectory.path.join("/")}
              items={["\u2022", ...currentDirectory.path].map((name, i) => {
                const path = currentDirectory.path.slice(0, i);
                return {
                  path,
                  text: name,
                  onClick: () => {
                    this.send({
                      type: "changeDirectory",
                      path,
                    });
                  },
                };
              })}
              // @ts-ignore
              breadcrumbRenderer={({ text, path, ...restProps }) => {
                const entry = breadcrumbEntries.find((entry) => {
                  return entry.path.join("/") === path.join("/");
                });
                if (!entry || entry.type !== "directory") {
                  throw new Error("?");
                }

                return (
                  <Breadcrumb {...restProps}>
                    <div
                      style={{
                        paddingRight: "10px",
                        display: "inline-block",
                        verticalAlign: "text-bottom",
                      }}
                    >
                      {entry.updating === "idle" ? (
                        <Spinner size={20} intent="none" />
                      ) : entry.updating === "updating" ? (
                        <Spinner size={20} intent="success" />
                      ) : entry.updating === "finished" ? (
                        <Icon iconSize={20} icon="folder-close" />
                      ) : null}
                    </div>
                    {text}
                  </Breadcrumb>
                );
              }}
            />
          </div>

          <FolderView
            key={"FolderView-" + currentDirectory.path.join("/")}
            entries={entries}
            onChangeDirectory={({ path }) => {
              this.send({
                type: "changeDirectory",
                path,
              });
            }}
            onDelete={({ path }) => {
              this.send({ type: "delete", path });
            }}
            onReveal={({ path }) => {
              this.send({ type: "reveal", path });
            }}
          />

          <div style={{ display: "flex" }}>
            <h4>{`${entries.length} items`}</h4>
            <Divider />
            <h4 title={`${totalSize.toLocaleString()} bytes`}>
              {`Total size: ${bytes(totalSize)}`}
            </h4>
            <Divider />
            <h4 title={`${free.toLocaleString()} bytes`}>
              {`Free space: ${bytes(free)}`}
            </h4>
          </div>
        </div>
      );
    });
  }
}
