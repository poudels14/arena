import { sql } from "@arena/slonik";
import { z } from "zod";
import { widgetConfigSchema } from "@arena/widgets/schema";
import { Context } from "./context";

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
    async fetchByAppId(appId: string): Promise<DbWidget[]> {
      const { rows } = await ctx.client.query<DbWidget>(
        sql`SELECT * FROM widgets WHERE archived_at IS NULL AND app_id = ${appId}`
      );
      return rows;
    },
    async insert(widget: DbWidget): Promise<void> {
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
      await ctx.client.query(
        sql`INSERT INTO widgets
        (id, name, slug, description, app_id, template_id, parent_id, config, created_by)
        VALUES (${id}, ${name}, ${slug}, ${
          description || ""
        }, ${appId}, ${templateId}, ${
          parentId || null
        }, ${config}, ${createdBy})`
      );
    },
    async update(widget: DbWidget) {
      const { id, name, slug, description, config, archivedAt } = widget;
      await ctx.client.query(
        sql`UPDATE widgets
        SET name=${name}, slug=${slug}, description=${description}, config=${config}, archived_at=${
          archivedAt || null
        }
        WHERE id = ${id}`
      );
    },
  };
};

export { createRepo, dbWidgetSchema };
export type { DbWidget };
