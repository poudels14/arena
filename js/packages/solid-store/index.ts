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
  : () => T;
type Store<T> = StoreValue<T>;

type StoreSetter<T> = SetStoreFunction<T>;

function createStore<T>(initValue: T) {
  let store = new Proxy(createDataNode(initValue, "$store"), proxyHandlers);
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
    if (
      p === $RAW ||
      p === $NODE ||
      p === $GET ||
      p === $SET ||
      p === $UPDATEDAT
    ) {
      return target[p];
    }
    // TODO(sagar): if getListener() is null, return $RAW data
    // this allows us to use store in non-reactive settings without
    // proxy, which is much more performant!
    let tp = toInternalKey(p);
    let v = target[tp];
    if (!v) {
      const rawTarget = target[$RAW];
      let desc;
      if (
        typeof rawTarget == "object" &&
        (desc = Reflect.getOwnPropertyDescriptor(rawTarget, tp)) &&
        desc.get
      ) {
        return desc.get;
      }
      const value = rawTarget?.[p];
      // Note(sagar): if the value of sub-field is null || undefined,
      // return undefined but make this call reactive
      if (value === undefined || value === null) {
        getListener() && void target();
        return value === undefined ? UNDEFINED_TRAP : NULL_TRAP;
      }
      v = target[tp] = new Proxy(
        (target[$NODE][tp] = createDataNode(value, p)),
        proxyHandlers
      );
    }
    return v;
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

const createDataNode = <T>(value: T, nodeName?: string) => {
  const node = function () {};
  return Object.defineProperties(node, {
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
};

const toInternalKey = (p: any) =>
  p === "name" ? $NAME : p === "length" ? $LENGTH : p;

const UNDEFINED_TRAP = new Proxy(function () {}, {
  get(_target: any, _: any): any {
    return UNDEFINED_TRAP;
  },
  apply(_target: any) {
    return undefined;
  },
});

const NULL_TRAP = new Proxy(function () {}, {
  get(_target: any, _: any): any {
    return NULL_TRAP;
  },
  apply(_target: any) {
    return null;
  },
});

let clockStopped = false;
function storeUpdater(root: any) {
  return (...path: any[]) => {
    if (!clockStopped) updateEpoch += 1;
    batch(() => {
      updatePath(root, path);
    });
  };
}

// TODO(sagar): not tested at all
const batchUpdates = (fn: any) => {
  updateEpoch += 1;
  clockStopped = true;
  // TODO(sagar): to improve batch performance, we can stop
  // cloning/copying data if the epoch is same as previous
  // since we know that the data wasn't changed
  batch(() => {
    fn();
  });
  clockStopped = false;
};

const updatePath = (root: any, path: any[]) => {
  const value = path.pop();

  const rootNode = root[$NODE];
  // TODO(sagar): maybe we don't need immutable data since
  // epoch is used to keep track of when the data was updated
  let nodeValue = (rootNode[$RAW] = copy(rootNode[$RAW]));
  rootNode[$UPDATEDAT] = updateEpoch;
  rootNode[$SET]?.(updateEpoch);

  let nodeNode = rootNode;
  let p = null;
  for (let i = 0; i < path.length - 1; i++) {
    p = path[i];
    nodeValue = nodeValue[p] = copy(nodeValue[p]);
    if (nodeNode && (nodeNode = nodeNode[toInternalKey(p)]?.[$NODE])) {
      nodeNode[$UPDATEDAT] = updateEpoch;
      nodeNode[$SET]?.(updateEpoch);
      nodeNode[$RAW] = nodeValue;
    }
  }

  const field = path[path.length - 1];
  const internalField = toInternalKey(field);
  // Note(sagar): if the value if not primitive type, need
  // to update the children objects. so call compareAndNotify
  nodeNode &&
    compareAndNotify(
      path.length > 0 ? nodeNode[internalField] : nodeNode,
      value
    );

  if (path.length > 0) {
    if (value === undefined) {
      delete nodeValue[field];
      nodeNode && delete nodeNode[internalField];
    } else {
      nodeValue[field] = value;
    }
  }
};

function copy(source: any) {
  // TODO(sagar): remove copy
  // To allow using getters inside a store, property
  // descriptor should be copied when copying data
  // to make updates immutable. And copying property
  // descriptors is ~4X slower. so, just mutate data
  // and use $UPDATEDAT field to compare data in
  // memo and etc.
  // return source;
  return source && !!source.pop
    ? [...source]
    : (Object.defineProperties(
        {},
        Object.getOwnPropertyDescriptors(source)
      ) as typeof source);
}

const compareAndNotify = (node: any, value: any) => {
  let prev;
  if (!node || !(node = node[$NODE]) || value === (prev = node[$RAW])) {
    return;
  }

  if (typeof value === "object" || typeof prev === "object") {
    const newKeys = isWrappable(value)
      ? Object.keys(value || {}).map((k) => toInternalKey(k))
      : [];

    const removedFields = new Set(
      isWrappable(prev)
        ? Object.keys(prev || {}).map((k) => toInternalKey(k))
        : []
    );

    for (let i = 0; i < newKeys.length; i++) {
      const k = newKeys[i];
      let v = value[k];
      const refK = node[k];
      removedFields.delete(k);
      // TODO: figure out how to "cache" array items properly
      compareAndNotify(refK, v);
    }

    [...removedFields].forEach((k) => {
      compareAndNotify(node[k], undefined);
      delete node[k];
    });

    if (prev?.length !== value?.length) {
      node[$LENGTH][$SET]?.(updateEpoch);
    }
  }

  node[$UPDATEDAT] = updateEpoch;
  node[$SET]?.(value);
  node[$RAW] = value;
};

export function isWrappable(obj: any) {
  let proto;
  return (
    obj != null &&
    typeof obj === "object" &&
    (obj[$RAW] ||
      !(proto = Object.getPrototypeOf(obj)) ||
      proto === Object.prototype ||
      Array.isArray(obj))
  );
}

export { createStore, batchUpdates, $RAW, $UPDATEDAT };
export type { StoreValue, Store, StoreSetter };
