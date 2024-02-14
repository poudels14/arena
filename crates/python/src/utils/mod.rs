use once_cell::sync::Lazy;

pub(crate) static NANOID_CHARS: Lazy<Vec<char>> = Lazy::new(|| {
  "123456789ABCDEFGHJKMNPQRSTUVWXYZabcdefghjkmnpqrstuvwxyz"
    .chars()
    .collect()
});
