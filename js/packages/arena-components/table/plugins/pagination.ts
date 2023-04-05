import { $RAW } from "@arena/solid-store";
import { klona } from "klona";
import { untrack } from "solid-js/web";
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
};

const withPagination: Plugin<
  PaginationConfig,
  { withPagination: PaginationState },
  PaginationMethods
> = (config) => {
  config = klona(config);
  return (table) => {
    const setPage = (page: number) => {
      table.setState("_plugins", "withPagination", "currentPage", page);
      return page;
    };

    table.setState("_plugins", "withPagination", {
      currentPage: 0,
      pageSize: config.pageSize || Infinity,
    });

    Object.assign(table, {
      hasPreviousPage() {
        return table.state._plugins.withPagination.currentPage() > 1;
      },
      previousPage() {
        const currentPage =
          table.state[$RAW]._plugins.withPagination.currentPage;
        if (untrack(table.hasPreviousPage)) {
          return setPage(currentPage - 1);
        }
        return currentPage;
      },
      hasNextPage() {
        const pagination = table.state._plugins.withPagination();
        return (
          pagination.currentPage + 1 <
          table.state._core.data().length / pagination.pageSize
        );
      },
      nextPage() {
        const currentPage =
          table.state[$RAW]._plugins.withPagination.currentPage;
        if (untrack(table.hasNextPage)) {
          return setPage(currentPage + 1);
        }
        return currentPage;
      },
    });

    Object.assign(table.internal, {
      getVisibleRows() {
        // generate new rows if data is changed
        void table.state._core.data();
        const pagination = table.state._plugins.withPagination();
        const startIdx = pagination.currentPage * pagination.pageSize;

        return [...Array(pagination.pageSize)].map((_, idx) => {
          return new Row(table.state, startIdx + idx);
        });
      },
    });
  };
};

export { withPagination };
