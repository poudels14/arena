import { Breadcrumbs } from "./Breadcrumbs";
import { Uploader } from "./Uploader";

type HeaderProps = {
  currentDir: string | null;
  selected?: any;
  breadcrumbs: {
    id: string;
    title: string;
  }[];
  onClickBreadcrumb: (id: string) => void;
  onUpload: (files: any[]) => void;
  onNewDirectory: () => void;
};

const Header = (props: HeaderProps) => {
  return (
    <div class="header flex justify-end shadow-sm bg-gray-50">
      <div class="flex-1">
        <Breadcrumbs
          breadcrumbs={props.breadcrumbs}
          onClickBreadcrumb={props.onClickBreadcrumb}
        />
      </div>
      <Uploader
        parentId={props.currentDir}
        onUpload={props.onUpload}
        onNewDirectory={props.onNewDirectory}
      />
    </div>
  );
};

export { Header };
