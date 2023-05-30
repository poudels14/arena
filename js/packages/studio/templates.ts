import Heading1, {
  metadata as heading1,
} from "@arena/widgets/builtin/Heading1";
import Heading2, {
  metadata as heading2,
} from "@arena/widgets/builtin/Heading2";
import Heading3, {
  metadata as heading3,
} from "@arena/widgets/builtin/Heading3";
import Text, { metadata as text } from "@arena/widgets/builtin/Text";
import Table, { metadata as table } from "@arena/widgets/builtin/table";
import Chart, { metadata as chart } from "@arena/widgets/builtin/Chart";
import SplitLayout, {
  metadata as splitLayout,
} from "@arena/widgets/builtin/SplitLayout";
import VerticalLayout, {
  metadata as verticalLayout,
} from "@arena/widgets/builtin/VerticalLayout";

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
  [text.id]: {
    Component: Text,
    metadata: text,
  },
  [splitLayout.id]: {
    Component: SplitLayout,
    metadata: splitLayout,
  },
  [verticalLayout.id]: {
    Component: VerticalLayout,
    metadata: verticalLayout,
  },
  [table.id]: {
    Component: Table,
    metadata: table,
  },
  [chart.id]: {
    Component: Chart,
    metadata: chart,
  },
};

export { TEMPLATES };
