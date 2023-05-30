import { sql } from "@arena/db/pg";
import { z } from "zod";
import { widgetConfigSchema } from "@arena/widgets/schema";
import { Context } from "./context";
import { merge } from "lodash-es";

const dbWidgetSchema = z.object({
  id: z.string(),
  appId: z.string(),
  name: z.string(),
  slug: z.string(),
  description: z.string().optional(),
  parentId: z.string().nullable(),
  templateId: z.string(),
  config: widgetConfigSchema,
  createdBy: z.string(),
  updatedAt: z.string(),
  archivedAt: z.string().optional(),
});

type DbWidget = z.infer<typeof dbWidgetSchema>;

const createRepo = (ctx: Context) => {
  return {
    async fetchById(id: string): Promise<DbWidget | null> {
      const { rows } = await ctx.client.query<DbWidget>(
        sql`SELECT * FROM widgets WHERE archived_at IS NULL AND id = ${id}`
      );
      return rows?.[0];
    },
    async getChildenWidgetsIds(parentId: string): Promise<DbWidget["id"][]> {
      const { rows } = await ctx.client.query<Pick<DbWidget, "id">>(
        sql`SELECT id FROM widgets WHERE archived_at IS NULL AND parent_id = ${parentId}`
      );
      return rows.map((r) => r.id);
    },
    async fetchByAppId(appId: string): Promise<DbWidget[]> {
      const { rows } = await ctx.client.query<DbWidget>(
        sql`SELECT * FROM widgets WHERE archived_at IS NULL AND app_id = ${appId}`
      );
      return rows;
    },
    async insert(widget: Omit<DbWidget, "updatedAt">): Promise<DbWidget> {
      const {
        id,
        name,
        slug,
        description,
        appId,
        templateId,
        parentId = null,
        config,
        createdBy,
      } = widget;
      const { rows } = await ctx.client.query(
        sql`INSERT INTO widgets
        (id, name, slug, description, app_id, template_id, parent_id, config, created_by)
        VALUES (${id}, ${name}, ${slug}, ${
          description || ""
        }, ${appId}, ${templateId}, ${
          parentId || null
        }, ${config}, ${createdBy})
        RETURNING id,created_at,updated_at`
      );
      const updated = rows[0];
      return merge(widget, updated) as DbWidget;
    },
    async update(widget: DbWidget): Promise<typeof widget> {
      const { name, slug, description, config, archivedAt } = widget;
      const { rows } = await ctx.client.query(
        sql`UPDATE widgets
        SET name=${name}, slug=${slug}, description=${description}, config=${config}, updated_at=NOW(), archived_at=${
          archivedAt || null
        }
        WHERE id = ${widget.id}
        RETURNING id,created_at,updated_at`
      );
      const updated = rows[0];
      return merge(widget, updated);
    },
    async archive(widgetIds: DbWidget["id"][]): Promise<void> {
      await ctx.client.query(
        sql`UPDATE widgets SET archived_at = NOW() WHERE id IN ${widgetIds}`
      );
    },
  };
};

export { createRepo, dbWidgetSchema };
export type { DbWidget };
