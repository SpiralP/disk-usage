interface EntryFile {
  type: "file";
  name: string;
  size: number;
}

interface EntryDirectory {
  type: "directory";
  name: string;
  size: number;
}

declare type Entry = EntryFile | EntryDirectory;

interface EventMessageDirectoryChange {
  type: "directoryChange";
  path: Array<string>;
  entries: Array<Entry>;
}

interface EventMessageSizeUpdate {
  type: "sizeUpdate";
  entry: Entry;
}

declare type EventMessage =
  | EventMessageDirectoryChange
  | EventMessageSizeUpdate;

interface ControlMessageChangeDirectory {
  type: "changeDirectory";
  path: Array<string>;
}

declare type ControlMessage = ControlMessageChangeDirectory;