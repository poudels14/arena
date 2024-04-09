use std::time::Instant;

use once_cell::sync::Lazy;

use crate::backend::DbAttribute;

pub(super) static NANOID_CHARS: Lazy<Vec<char>> = Lazy::new(|| {
  "123456789ABCDEFGHJKMNPQRSTUVWXYZabcdefghjkmnpqrstuvwxyz"
    .chars()
    .collect()
});

#[derive(Debug)]
pub(super) struct CachedFile {
  pub id: String,
  pub is_new: bool,
  pub content: Vec<u8>,
  pub updated_at: Instant,
}

#[allow(unused)]
#[derive(Debug)]
pub struct FileObject {
  pub id: String,
  pub parent_id: Option<String>,
  pub path: String,
  pub content: Vec<u8>,
  pub updated_at: Instant,
}

#[derive(Debug, Default)]
pub(super) struct Node {
  pub attr: DbAttribute,
  pub cached_at: Option<Instant>,
  pub archived_at: Option<Instant>,
}

pub struct FilesCache {
  pub(super) nodes: Vec<Node>,
  cached_files: Vec<CachedFile>,
}

impl FilesCache {
  pub fn new() -> Self {
    let mut cache = Self {
      nodes: vec![],
      cached_files: vec![],
    };
    cache.reset();
    cache
  }

  /// Resets all
  pub fn reset(&mut self) {
    self.nodes = Vec::with_capacity(50);
    // index starts at 1, so add None at first index
    self.nodes.push(Node {
      attr: DbAttribute {
        // Set this to absurd value so that it's not matched during lookup
        parent_id: Some(nanoid::nanoid!(21, &NANOID_CHARS)),
        ..Default::default()
      },
      ..Default::default()
    });
  }

  #[allow(dead_code)]
  pub fn list_files_updated_after(&self, instant: Instant) -> Vec<FileObject> {
    self
      .cached_files
      .iter()
      .filter(|file| file.is_new && file.updated_at > instant)
      .filter_map(|file| {
        let node = self.nodes.iter().find(|node| {
          node.attr.id.as_ref() == Some(&file.id) && node.archived_at.is_none()
        });
        node.filter(|n| !n.attr.is_directory).map(|node| {
          // if parent id is root, set it to none
          let parent_id = node
            .attr
            .parent_id
            .as_ref()
            .filter(|id| id.as_str() != "/")
            .cloned();
          FileObject {
            id: file.id.clone(),
            parent_id,
            path: self
              .get_file_path(self.find_node_index(Some(&file.id)).unwrap()),
            content: file.content.clone(),
            updated_at: file.updated_at.clone(),
          }
        })
      })
      .collect()
  }

  #[inline]
  pub(super) fn add(&mut self, file: CachedFile) {
    self.cached_files.push(file)
  }

  #[inline]
  pub(super) fn find(&mut self, id: &str) -> Option<&mut CachedFile> {
    self.cached_files.iter_mut().find(|file| file.id == id)
  }

  #[inline]
  pub(super) fn find_node_index(&self, id: Option<&String>) -> Option<usize> {
    self
      .nodes
      .iter()
      .position(|node| node.attr.id.as_ref() == id)
  }

  fn get_file_path(&self, ino: usize) -> String {
    let mut components = vec![];
    let mut node = &self.nodes[ino];
    components.push(node.attr.name.clone());
    loop {
      let parent_ino = self.find_node_index(node.attr.parent_id.as_ref());
      match parent_ino {
        Some(ino) => {
          let parent = &self.nodes[ino];
          // if parent.attr.id.as_str() == "/" {

          if parent.attr.id.is_none() {
            // do this to not include the dummy root id
            break;
          }
          components.push(parent.attr.name.to_owned());
          node = parent;
        }
        None => break,
      }
    }
    components.push("".to_owned());
    components.reverse();
    components.join("/")
  }
}
