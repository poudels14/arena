use rocksdb::DBPinnableSlice;

pub mod cf;

pub type PinnableSlice<'a> = DBPinnableSlice<'a>;
