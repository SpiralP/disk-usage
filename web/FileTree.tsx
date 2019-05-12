import React from "react";

type Tree = { [componentName: string]: number | Tree };

interface FileTreeProps {
  tree: Tree;
  root: string;
}

interface FileTreeState {}

export default class FileTree extends React.Component<
  FileTreeProps,
  FileTreeState
> {
  render() {
    const { tree, root } = this.props;

    return (
      <div>
        <h2>{root}</h2>
        {Object.entries(tree).map(([component, next]) => {
          const path = root + "/" + component;

          if (typeof next === "number") {
            return <h3 key={"file " + path}>{component}</h3>;
          } else {
            return <FileTree key={path} root={path} tree={next} />;
          }
        })}
      </div>
    );
  }
}
