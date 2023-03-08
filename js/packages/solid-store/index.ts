import { batch, createSignal, getListener } from "solid-js";
import type { SetStoreFunction } from "solid-js/store";

let updateEpoch = 1;
const $STORE = Symbol("_solid_store_");
const $RAW = Symbol("_value_");
const $NODE = Symbol("_node_");
const $UPDATEDAT = Symbol("_updated_at_");
const $GET = Symbol("_get_signal_");
const $SET = Symbol("_set_signal_");
// since function is the target of the proxy,
// name and length can't be changed. so, use symbol
// for those fields
const $NAME = Symbol("_name_");
const $LENGTH = Symbol("_length_");

type StoreValue<T> = T extends {}
  ? { [K in keyof T]: StoreValue<T[K]> & (() => T[K]) } & {
      [$RAW]: T;
      [$UPDATEDAT]: number;
    } & (() => T)
  : null;
type Store<T> = StoreValue<T>;

type StoreSetter<T> = SetStoreFunction<T>;

function createStore<T>(initValue: T) {
  let store = trap(initValue, "$store");
  Object.defineProperties(store, {
    [$STORE]: {
      value: true,
      writable: false,
      enumerable: true,
    },
  });
  return [store, storeUpdater(store)] as [Store<T>, StoreSetter<T>];
}

const proxyHandlers = {
  get(target: any, p: any) {
    if (p === $RAW || p === $GET || p === $SET) {
      return target[p];
    }
    let tp = p === "name" ? $NAME : p === "length" ? $LENGTH : p;
    let t = target[tp];
    if (!t && (tp === $NAME || p === $LENGTH || typeof p === "string")) {
      const data = target[$RAW]?.[p];
      // Note(sagar): if the value of sub-field is null || undefined,
      // return undefined but make this call reactive
      if (data === undefined || data === null) {
        getListener() && void target.call();
        return UNDEFINED_TRAP;
      }
      t = trap(data, p);
      target[tp] = t;
    }
    return t;
  },
  apply(target: any) {
    /**
     * Note(sagar): instead of storing the value in signal, only store
     * the updated "epoch". when caller calls the function, return
     * the actual value but make this call reactive by calling signal
     * getter
     */
    if (!target[$GET]) {
      const [get, set] = createSignal(target[$RAW]);
      target[$GET] = get;
      target[$SET] = set;
      target[$UPDATEDAT] = updateEpoch;
    }
    // call signal here only to make apply reactive
    getListener() && void target[$GET]!();
    return target[$RAW];
  },
  set(_target: any, __: any) {
    throw new Error("Can't edit store directly");
  },
  deleteProperty(_target: any, _: any) {
    throw new Error("Can't edit store directly");
  },
  // TODO: trap set so that storeRef can't be assigned
  // setStore should be called to trigger the reactive update
};

const trap = (value: any, nodeName?: string) => {
  const node = function () {};
  let target = Object.defineProperties(node, {
    [$RAW]: {
      // TODO(sagar): maybe store weak ref here? atleast check memory leaks
      value,
      writable: true,
      enumerable: true, // TODO
    },
    [$NODE]: {
      // TODO(sagar): maybe store weak ref here? atleast check memory leaks
      value: node,
      writable: true,
      enumerable: false,
    },
    name: {
      value: nodeName || "node",
      writable: false,
      enumerable: false,
    },
  });
  return new Proxy(target, proxyHandlers);
};

const UNDEFINED_TRAP = new Proxy(function () {}, {
  get(_target: any, _: any): any {
    return UNDEFINED_TRAP;
  },
  apply(_target: any) {
    return undefined;
  },
});

function storeUpdater(root: any) {
  return (...path: any[]) => {
    updateEpoch += 1;
    batch(() => {
      const value = path.pop();

      const rootNode = root[$NODE];
      // TODO(sagar): maybe we don't need immutable data since
      // epoch is used to keep track of when the data was updated
      let nodeValue = (rootNode[$RAW] = copy(rootNode[$RAW]));
      rootNode[$UPDATEDAT] = updateEpoch;
      rootNode[$SET]?.(updateEpoch);

      let node = root;
      let p = null;
      for (let i = 0; i < path.length - 1; i++) {
        p = path[i];
        node = node[p];
        nodeValue = nodeValue[p] = copy(nodeValue[p]);

        const nodeNode = node[$NODE];
        nodeNode[$UPDATEDAT] = updateEpoch;
        nodeNode[$SET]?.(updateEpoch);
        nodeNode[$RAW] = nodeValue;
      }

      let field = path[path.length - 1];
      // Note(sagar): if the value if not primitive type, need
      // to update the children objects. so call compareAndNotify
      compareAndNotify(path.length > 0 ? node[field] : node, value);

      if (path.length > 0) {
        if (value === undefined) {
          delete nodeValue[field];
          delete node[$NODE][field];
        } else {
          nodeValue[field] = value;
        }
      }
    });
  };
}

function copy(source: any) {
  return source && !!source.pop ? [...source] : { ...source };
}

const compareAndNotify = (node: any, value: any) => {
  const nodeNode = node[$NODE];
  const prev = nodeNode[$RAW];
  if (prev === value) {
    return;
  }

  if (typeof value === "object" || typeof prev === "object") {
    const newKeys = Object.keys(value || {});

    const removedFields = new Set([...Object.keys(prev || {})]);

    for (let i = 0; i < newKeys.length; i++) {
      const k = newKeys[i];
      let v = value[k];
      const refK = nodeNode[k];
      removedFields.delete(k);
      // TODO: figure out how to "cache" array items properly
      compareAndNotify(refK, v);
    }

    [...removedFields].forEach((k) => {
      compareAndNotify(nodeNode[k], undefined);
      delete nodeNode[k];
    });

    if (prev?.length !== value?.length) {
      nodeNode[$LENGTH][$SET]?.(value.length);
    }
  }

  nodeNode[$UPDATEDAT] = updateEpoch;
  nodeNode[$SET]?.(value);
  nodeNode[$RAW] = value;
};

export { createStore, $RAW, $UPDATEDAT };
export type { StoreValue, Store, StoreSetter };
