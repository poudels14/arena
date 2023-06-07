import { $RAW } from "@arena/solid-store";
import { klona } from "klona";
import { createMemo, untrack } from "solid-js";
import { Plugin } from "..";
import { Row } from "../row";

type PaginationConfig = {
  pageSize?: number;
};

type PaginationState = {
  currentPage: number;
  pageSize: number;
};

type PaginationMethods = {
  /**
   * @returns whether the table can go to the previous page
   */
  hasPreviousPage: () => boolean;

  /**
   * go to previous page
   *
   * @returns previous page index
   */
  previousPage: () => null | number;

  /**
   * @returns whether the table can go to the next page
   */
  hasNextPage: () => boolean;

  /**
   * go to next page
   *
   * @returns next page index
   */
  nextPage: () => null | number;

  pageSize: () => number;

  /**
   * @returns the current page number
   */
  currentPage: () => number;

  /**
   * @returns the total number of pages
   */
  totalPages: () => number;
};

const withPagination: Plugin<
  PaginationConfig,
  { pagination: PaginationState },
  PaginationMethods
> = (config) => {
  config = klona(config);
  return (table) => {
    const setPage = (page: number) => {
      table.setState("_plugins", "pagination", "currentPage", page);
      return page;
    };

    table.setState("_plugins", "pagination", {
      currentPage: 1,
      pageSize: config.pageSize || 1000,
    });

    Object.assign(table, {
      hasPreviousPage() {
        return table.state._plugins.pagination.currentPage() > 1;
      },
      previousPage() {
        const currentPage = table.state[$RAW]._plugins.pagination.currentPage;
        if (untrack(table.hasPreviousPage)) {
          return setPage(currentPage - 1);
        }
        return currentPage;
      },
      hasNextPage() {
        const pagination = table.state._plugins.pagination();
        return (
          pagination.currentPage <
          table.state._core.data().length / pagination.pageSize
        );
      },
      nextPage() {
        const currentPage = table.state[$RAW]._plugins.pagination.currentPage;
        if (untrack(table.hasNextPage)) {
          return setPage(currentPage + 1);
        }
        return currentPage;
      },
      pageSize() {
        return table.state._plugins.pagination.pageSize();
      },
      currentPage() {
        return table.state._plugins.pagination.currentPage();
      },
      totalPages() {
        const pagination = table.state._plugins.pagination();
        return (
          Math.floor(table.state._core.data().length / pagination.pageSize) + 1
        );
      },
    });

    const getVisibleRows = createMemo(() => {
      const rows = table.state._core.data();
      const pagination = table.state._plugins.pagination();
      const startIdx = (pagination.currentPage - 1) * pagination.pageSize;

      return [
        ...Array(Math.min(pagination.pageSize, rows.length - startIdx)),
      ].map((_, idx) => {
        return new Row(table.state, startIdx + idx);
      });
    });

    Object.assign(table.internal, {
      getVisibleRows,
    });
  };
};

export { withPagination };
