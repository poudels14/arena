import { Acl, createRepo } from "../repos/acl";
import { App } from "../repos/app";
import { DbResource } from "../repos/resources";
import type { User } from "../repos/user";
import { Client } from "@arena/runtime/postgres";
import { Workspace } from "../repos/workspace";

type AccessType = "can-view" | "can-trigger-mutate-query" | "admin" | "owner";
type WorkspaceAccessType = "member" | "admin" | "owner";

type UserInfo = Pick<User, "id" | "email" | "config"> & {
  workspaces: Workspace[];
};

class AclChecker {
  private repo: ReturnType<typeof createRepo>;
  private user: UserInfo;
  private accessList: Acl[] | null;
  constructor(client: Client, user: UserInfo | null | undefined) {
    this.repo = createRepo({ client });
    this.user = user || {
      id: "public",
      email: "",
      config: {
        waitlisted: false,
      },
      workspaces: [],
    };
    this.accessList = null;
  }

  private async getAccessList() {
    if (!this.accessList) {
      this.accessList = await this.repo.listAccess({
        userId: this.user.id,
        workspaceId: null,
      });
    }
    return this.accessList;
  }

  private isAccessSameOrSuperseding(access: AccessType, another: AccessType) {
    return (
      access == another ||
      access == "owner" ||
      (access == "admin" &&
        ["can-view", "can-trigger-mutate-query"].includes(another))
    );
  }

  async filterAppsByAccess<A extends Pick<App, "id" | "workspaceId">>(
    apps: A[],
    access: AccessType
  ) {
    if (apps.length > 0 && new Set(apps.map((a) => a.workspaceId)).size != 1) {
      throw new Error("Only apps in the same workspace can be filtered");
    }

    const accessList = await this.getAccessList();
    const appsAccesses = accessList.reduce((agg, acl) => {
      if (
        acl.appId &&
        (acl.userId == this.user.id || acl.userId == "everyone")
      ) {
        agg[acl.appId] = acl.access;
      }
      return agg;
    }, {} as Record<App["id"], AccessType>);

    return apps.filter((a) =>
      this.isAccessSameOrSuperseding(appsAccesses[a.id], access)
    );
  }

  // TODO(sagar): generalize access check for apps/resources
  // TODO(sagar): test
  async filterResourcesByAccess<
    R extends Pick<DbResource, "id" | "workspaceId">
  >(resources: R[], access: AccessType) {
    if (
      resources.length > 0 &&
      new Set(resources.map((a) => a.workspaceId)).size != 1
    ) {
      throw new Error("Only resources in the same workspace can be filtered");
    }

    const accessList = await this.getAccessList();
    const resourcesAccesses = accessList.reduce((agg, acl) => {
      if (
        acl.resourceId &&
        (acl.userId == this.user.id || acl.userId == "everyone")
      ) {
        agg[acl.resourceId] = acl.access;
      }
      return agg;
    }, {} as Record<DbResource["id"], AccessType>);

    return resources.filter((a) =>
      this.isAccessSameOrSuperseding(resourcesAccesses[a.id], access)
    );
  }

  async hasWorkspaceAccess(workspaceId: string, access: WorkspaceAccessType) {
    return (
      this.user.workspaces.findIndex(
        (w) =>
          w.id == workspaceId &&
          (w.access == access ||
            w.access == "owner" ||
            (access == "member" && ["owner", "admin"].includes(w.access)))
      ) > -1
    );
  }

  async hasAppAccess(appId: string, access: AccessType) {
    return (
      (await this.filterAppsByAccess([{ id: appId, workspaceId: "" }], access))
        .length == 1
    );
  }

  async hasResourceAccess(resourceId: string, access: AccessType) {
    return (
      (
        await this.filterResourcesByAccess(
          [{ id: resourceId, workspaceId: "" }],
          access
        )
      ).length == 1
    );
  }
}

export { AclChecker };
export type { AccessType, WorkspaceAccessType };
