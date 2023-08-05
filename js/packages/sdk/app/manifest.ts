type TemplateManifest = {
  id: string;
  name: string;
  /**
   * semver version
   */
  version: string;

  editor?: {
    /**
     * Whether the app using this template is editable
     */
    editable?: boolean;
  };
};

export type { TemplateManifest };
