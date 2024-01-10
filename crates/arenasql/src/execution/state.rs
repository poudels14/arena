use std::any::{type_name, Any, TypeId};
use std::collections::BTreeMap;

// Credit: deno
/// This is used to store aribitary data related to the session.
/// The fields in the SessionState can't be changed. If mutable
/// state is needed, a struct that can be mutated should be added
/// to the map
#[derive(Default, Debug)]
pub struct SessionState {
  data: BTreeMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl SessionState {
  pub fn put<T>(&mut self, t: T) -> Option<Box<T>>
  where
    T: 'static + Send + Sync,
  {
    let type_id = TypeId::of::<T>();
    let prev = self.data.insert(type_id, Box::new(t));
    if let Some(prev) = prev {
      return Some(prev.downcast::<T>().unwrap());
    } else {
      None
    }
  }

  pub fn remove<T>(&mut self) -> Option<Box<T>>
  where
    T: 'static + Send + Sync,
  {
    let type_id = TypeId::of::<T>();
    let value = self.data.remove(&type_id);
    if let Some(prev) = value {
      return Some(prev.downcast::<T>().unwrap());
    } else {
      None
    }
  }

  pub fn has<T>(&self) -> bool
  where
    T: 'static + Send + Sync,
  {
    let type_id = TypeId::of::<T>();
    self.data.get(&type_id).is_some()
  }

  pub fn try_borrow<T>(&self) -> Option<&T>
  where
    T: 'static + Send + Sync,
  {
    let type_id = TypeId::of::<T>();
    self.data.get(&type_id).and_then(|b| b.downcast_ref())
  }

  pub fn borrow<T>(&self) -> &T
  where
    T: 'static + Send + Sync,
  {
    self.try_borrow().unwrap_or_else(|| missing::<T>())
  }
}

fn missing<T: 'static>() -> ! {
  panic!(
    "required type {} is not present in SessionState container",
    type_name::<T>()
  );
}
