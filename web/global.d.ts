type Path = Array<string>;

declare type Entry = EntryFile | EntryDirectory;

interface EntryFile {
  type: "file";
  path: Path;
  size: number;
}

interface EntryDirectory {
  type: "directory";
  path: Path;
  size: number;
  updating: "idle" | "updating" | "finished";
}

// Event Messages

declare type EventMessage =
  | EventMessageDirectoryChange
  | EventMessageSizeUpdate
  | EventMessageDeleting;

interface EventMessageDirectoryChange {
  type: "directoryChange";
  currentDirectory: EntryDirectory;
  entries: Array<Entry>;
  breadcrumbEntries: Array<Entry>;
  availableSpace: number;
}

interface EventMessageSizeUpdate {
  type: "sizeUpdate";
  entry: EntryDirectory;
}

interface EventMessageDeleting {
  type: "deleting";
  path: Path;
  status: "deleting" | "finished";
}

// Control Messages

declare type ControlMessage =
  | ControlMessageChangeDirectory
  | ControlMessageDelete
  | ControlMessageReveal;

interface ControlMessageChangeDirectory {
  type: "changeDirectory";
  path: Path;
}

interface ControlMessageDelete {
  type: "delete";
  path: Path;
}

interface ControlMessageReveal {
  type: "reveal";
  path: Path;
}
