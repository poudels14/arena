export const $STORE = Symbol("_solid_store_");
export const $RAW = Symbol("_value_");
export const $NODE = Symbol("_node_");
export const $UPDATEDAT = Symbol("_updated_at_");
export const $GET = Symbol("_get_signal_");
export const $SET = Symbol("_set_signal_");
// since function is the target of the proxy,
// name and length can't be changed. so, use symbol
// for those fields
export const $NAME = Symbol("_name_");
export const $LENGTH = Symbol("_length_");
