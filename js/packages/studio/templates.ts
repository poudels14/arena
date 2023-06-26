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
import Box, { metadata as box } from "@arena/widgets/builtin/Box";
import Button, { metadata as button } from "@arena/widgets/builtin/Button";
import Select, { metadata as select } from "@arena/widgets/builtin/Select";
import Input, { metadata as input } from "@arena/widgets/builtin/Input";
import Textarea, {
  metadata as textarea,
} from "@arena/widgets/builtin/Textarea";
import Table, { metadata as table } from "@arena/widgets/builtin/table";
import Chart, { metadata as chart } from "@arena/widgets/builtin/Chart";
import SplitLayout, {
  metadata as splitLayout,
} from "@arena/widgets/builtin/SplitLayout";
import VerticalLayout, {
  metadata as verticalLayout,
} from "@arena/widgets/builtin/VerticalLayout";

import ResourcePicker, {
  metadata as resourcePicker,
} from "@arena/widgets/arena/ResourcePicker";

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
  [box.id]: {
    Component: Box,
    metadata: box,
  },
  [button.id]: {
    Component: Button,
    metadata: button,
  },
  [select.id]: {
    Component: Select,
    metadata: select,
  },
  [input.id]: {
    Component: Input,
    metadata: input,
  },
  [textarea.id]: {
    Component: Textarea,
    metadata: textarea,
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
  [resourcePicker.id]: {
    Component: ResourcePicker,
    metadata: resourcePicker,
  },
};

export { TEMPLATES };
