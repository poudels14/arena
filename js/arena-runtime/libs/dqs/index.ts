class WorkspaceServer {
  rid: number;
  private constructor(rid: number) {
    this.rid = rid;
  }

  static async start(workspaceId: string) {
    const rid = await Arena.core.opAsync(
      "op_dqs_start_workspace_server",
      workspaceId
    );
    return new WorkspaceServer(rid);
  }
}

export { WorkspaceServer };
