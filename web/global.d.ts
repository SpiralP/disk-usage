interface EntryFile {
  type: "file";
  name: string;
  size: number;
}

interface EntryDirectory {
  type: "directory";
  name: string;
  size: number;
  updating: boolean;
}

declare type Entry = EntryFile | EntryDirectory;

interface EventMessageDirectoryChange {
  type: "directoryChange";
  path: Array<string>;
  entries: Array<Entry>;
  free: number;
}

interface EventMessageSizeUpdate {
  type: "sizeUpdate";
  entry: EntryDirectory;
}

declare type EventMessage =
  | EventMessageDirectoryChange
  | EventMessageSizeUpdate;

// Control Messages

interface ControlMessageChangeDirectory {
  type: "changeDirectory";
  path: Array<string>;
}

interface ControlMessageDelete {
  type: "delete";
  path: Array<string>;
}

interface ControlMessageReveal {
  type: "reveal";
  path: Array<string>;
}

declare type ControlMessage =
  | ControlMessageChangeDirectory
  | ControlMessageDelete
  | ControlMessageReveal;
