mod session;

pub use session::{
  AuthenticatedSession, AuthenticatedSessionBuilder, AuthenticatedSessionStore,
};

#[derive(Debug, Clone, PartialEq)]
pub enum AuthHeader {
  Authenticated { session_id: String },
  Token { token: String },
  None,
}

impl Default for AuthHeader {
  fn default() -> Self {
    Self::None
  }
}
