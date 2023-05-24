import Heading1, {
  metadata as heading1,
} from "@arena/widgets/builtin/Heading1";
import Heading2, {
  metadata as heading2,
} from "@arena/widgets/builtin/Heading2";
import Heading3, {
  metadata as heading3,
} from "@arena/widgets/builtin/Heading3";
import Table, { metadata as tableMetadata } from "@arena/widgets/builtin/table";
import Chart, { metadata as chartMetadata } from "@arena/widgets/builtin/Chart";
import GridLayout, {
  metadata as gridLayoutMetadata,
} from "@arena/widgets/builtin/GridLayout";

const TEMPLATES = {
  // TODO(sagar): make these lazy load
  [heading1.id]: {
    Component: Heading1,
    metadata: heading1,
  },
  [heading2.id]: {
    Component: Heading2,
    metadata: heading2,
  },
  [heading3.id]: {
    Component: Heading3,
    metadata: heading3,
  },
  [gridLayoutMetadata.id]: {
    Component: GridLayout,
    metadata: gridLayoutMetadata,
  },
  [tableMetadata.id]: {
    Component: Table,
    metadata: tableMetadata,
  },
  [chartMetadata.id]: {
    Component: Chart,
    metadata: chartMetadata,
  },
};

export { TEMPLATES };
