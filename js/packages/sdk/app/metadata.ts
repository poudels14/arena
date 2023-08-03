import { lazy } from "solid-js";

type Metadata = {
  id: string;
  /**
   * semver version
   */
  version: string;
  title: string;

  // component: (props: any) => JSX.Element;
} & (
  | {
      // entry: (props: any) => JSX.Element;
    }
  | {
      /**
       * This is set if the current module is an app
       */
      children: [
        Metadata,
        /**
         * This is the entry component of the app and should only be set
         * if it's is an App module.
         *
         * Use ReturnType of lazy(...) to make sure theses components are
         * lazily loaded. This is to ensure that the code splitting happens
         * at app level
         *
         * If entry isn't set, the current directory/module will be considered
         * a directory.
         */
        ReturnType<typeof lazy>
      ];
    }
);

const createMetadata = (init: Metadata) => {
  return init;
};

export type { Metadata };
export { createMetadata };
