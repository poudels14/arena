import { Accessor, createSignal } from "solid-js";

/**
 * This generates arguments that are passed to the query.
 *
 * This function needs to be resolved before querying data.
 * So, if a widget depends on a state of other widget, the args
 * builder shouldn't resolve until the state of other widget
 * is ready. This makes it easier for dependency handling, etc.
 */
type QueryArgsBuilder<Args> = () => Args | Promise<Args>;
type DataLoader<T> = (...args: [any]) => Promise<T>;

/**
 * If any of the following variables are accessed inside this function,
 * those values are passed from frontend using argument builder
 *    - app
 *    - widgets
 *    - self
 */
const createWidgetData = <Args, Data>(
  argsBuilder: QueryArgsBuilder<Args>,
  loader: DataLoader<Data>
): [Accessor<Data | null>] => {
  const args = argsBuilder();
  const [data, setData] = createSignal<Data | null>(null);

  loader(args).then();
  setTimeout(() => {
    setData({
      name: "CreateWidgetData",
    } as any);
  }, 2000);
  return [data];
};

export { createWidgetData };
