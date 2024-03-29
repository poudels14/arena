use ring::aead::{self, Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};

pub fn encrypt(
  key: &[u8],
  nonce: [u8; aead::NONCE_LEN],
  data: &[u8],
) -> Vec<u8> {
  let key = UnboundKey::new(&AES_256_GCM, key).expect("error creating key");
  let nonce_sequence = Nonce::assume_unique_for_key(nonce);
  let aad = Aad::empty();

  let mut in_out = data.to_vec();
  in_out.extend_from_slice(&vec![0; AES_256_GCM.tag_len()]);
  let s_key = LessSafeKey::new(key);

  s_key
    .seal_in_place_append_tag(nonce_sequence, aad, &mut in_out)
    .expect("error encrypting");
  in_out
}

pub fn decrypt(
  key: &[u8],
  nonce: &[u8],
  mut encrypted_data: Vec<u8>,
) -> Vec<u8> {
  let unbound_key =
    UnboundKey::new(&AES_256_GCM, key).expect("encryption error");

  let mut nonce_slice = [0u8; 12];
  nonce_slice.copy_from_slice(&nonce[0..12]);
  let nonce = Nonce::assume_unique_for_key(nonce_slice);
  let aad = Aad::empty();

  let s_key = LessSafeKey::new(unbound_key);
  let decrypted_data = s_key
    .open_in_place(nonce, aad, &mut encrypted_data)
    .expect("encryption error");

  decrypted_data[..decrypted_data.len() - AES_256_GCM.tag_len()].to_vec()
}
