import z from "zod";

z.ZodType.prototype.title = function (title: string) {
  this._def.title = title;
  return this;
};

declare module "zod" {
  interface ZodTypeDef {
    title?: string;
  }

  interface ZodType<
    Output = any,
    Def extends ZodTypeDef = ZodTypeDef,
    Input = Output
  > {
    title<T extends z.ZodTypeAny>(this: T, title: string): T;
  }
}

export { z };
