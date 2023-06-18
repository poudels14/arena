import { Acl, createRepo } from "../repos/acl";
import { App } from "../repos/app";
import type { User } from "../repos/user";
import { Client } from "@arena/runtime/postgres";

type AccessType = "can-view" | "can-trigger-mutate-query" | "admin";

class AclChecker {
  private repo: ReturnType<typeof createRepo>;
  private user: Pick<User, "id" | "email" | "config">;
  private accessList: Acl[] | null;
  constructor(client: Client, user: User | null | undefined) {
    this.repo = createRepo({ client });
    this.user = user || {
      id: "public",
      email: "",
      config: {
        waitlisted: false,
      },
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
      (access == "admin" &&
        ["can-view", "can-trigger-mutate-query"].includes(another))
    );
  }

  async filterAppsByAccess<A extends Pick<App, "id" | "workspaceId">>(
    apps: A[],
    access: AccessType
  ) {
    if (new Set(apps.map((a) => a.workspaceId)).size != 1) {
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

  async hasWorkspaceAccess(workspaceId: string, access: AccessType) {
    throw new Error("not implemented");
  }

  async hasAppAccess(appId: string, access: AccessType) {
    return (
      (await this.filterAppsByAccess([{ id: appId, workspaceId: "" }], access))
        .length == 1
    );
  }
}

export { AclChecker };
export type { AccessType };
