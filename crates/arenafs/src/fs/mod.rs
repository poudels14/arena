use std::ffi::OsStr;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use chrono::{DateTime, Utc};
use fuser::{
  FileType, ReplyAttr, ReplyData, ReplyDirectory, ReplyEmpty, ReplyEntry,
  ReplyWrite, ReplyXattr, Request,
};
use once_cell::sync::Lazy;

use crate::backend::{Backend, DbAttribute};
use crate::error::Error;

static NANOID_CHARS: Lazy<Vec<char>> = Lazy::new(|| {
  "123456789ABCDEFGHJKMNPQRSTUVWXYZabcdefghjkmnpqrstuvwxyz"
    .chars()
    .collect()
});

#[derive(Debug, Default)]
struct Node {
  attr: DbAttribute,
  cached_at: Option<Instant>,
  archived_at: Option<Instant>,
}

#[derive(Debug)]
struct CachedFile {
  id: String,
  is_new: bool,
  content: Vec<u8>,
}

#[allow(unused)]
#[derive(Debug)]
pub struct NewFile {
  id: String,
  parent_id: Option<String>,
  path: String,
  content: Vec<u8>,
}

pub struct Options {
  pub root_id: Option<String>,
  pub user_id: u32,
  pub group_id: u32,
}

pub struct FileSystem {
  options: Options,
  backend: Arc<dyn Backend>,
  nodes: Vec<Node>,
  cached_files: Vec<CachedFile>,
}

impl FileSystem {
  pub async fn with_backend(
    options: Options,
    backend: Arc<dyn Backend>,
  ) -> Result<Self, Error> {
    let mut nodes = Vec::with_capacity(50);
    // index starts at 1, so add None at first index
    nodes.push(Node {
      attr: DbAttribute {
        // Set this to absurd value so that it's not matched during lookup
        parent_id: Some(nanoid::nanoid!(21, &NANOID_CHARS)),
        ..Default::default()
      },
      ..Default::default()
    });

    let mut fs = Self {
      options,
      backend,
      nodes,
      cached_files: vec![],
    };

    let root_id = fs.options.root_id.clone();
    let root = fs
      .backend
      .fetch_node(root_id.as_ref())
      .await?
      .unwrap_or_else(|| DbAttribute {
        id: "/".to_owned(),
        parent_id: Some("/dev/null".to_owned()),
        is_directory: true,
        ..Default::default()
      });
    fs.nodes.push(Node {
      attr: root,
      ..Default::default()
    });
    fs.load_children_nodes(1)?;
    Ok(fs)
  }

  #[tracing::instrument(skip(self), level = "debug")]
  // Before fetching a dir, the dir's attr should already be in nodes
  pub fn load_children_nodes(&mut self, ino: usize) -> Result<(), Error> {
    let node = &mut self.nodes[ino];
    let id = node.attr.id.clone();
    let children_nodes = futures::executor::block_on(async {
      // root should be passed as None
      let id = if id == "/" { None } else { Some(id) };
      self.backend.fetch_children(id.as_ref()).await
    })
    .expect("Error loading children nodes");

    node.cached_at = Some(Instant::now());
    children_nodes.into_iter().for_each(|child| {
      let existing_ino = self.find_node_index(Some(&child.id));
      let node = Node {
        attr: child,
        ..Default::default()
      };
      if let Some(ino) = existing_ino {
        self.nodes[ino] = node;
      } else {
        self.nodes.push(node);
      }
    });
    Ok(())
  }

  pub fn list_new_files(&self) -> Vec<NewFile> {
    self
      .cached_files
      .iter()
      .filter(|file| file.is_new)
      .filter_map(|file| {
        let node = self
          .nodes
          .iter()
          .find(|node| node.attr.id == file.id && node.archived_at.is_none());
        node.filter(|n| !n.attr.is_directory).map(|node| {
          // if parent id is root, set it to none
          let parent_id = node
            .attr
            .parent_id
            .as_ref()
            .filter(|id| id.as_str() != "/")
            .cloned();
          NewFile {
            id: file.id.clone(),
            parent_id,
            path: self
              .get_file_path(self.find_node_index(Some(&file.id)).unwrap()),
            content: file.content.clone(),
          }
        })
      })
      .collect()
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
          if parent.attr.id.as_str() == "/" {
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

  #[inline]
  fn find_node_index(&self, id: Option<&String>) -> Option<usize> {
    self.nodes.iter().position(|node| Some(&node.attr.id) == id)
  }

  #[inline]
  fn get_fuser_attr(&self, ino: usize, attr: &DbAttribute) -> fuser::FileAttr {
    fuser::FileAttr {
      ino: ino as u64,
      size: attr.size as u64,
      blocks: 1,
      atime: SystemTime::now(),
      mtime: DateTime::<Utc>::from_naive_utc_and_offset(attr.updated_at, Utc)
        .into(),
      ctime: DateTime::<Utc>::from_naive_utc_and_offset(attr.updated_at, Utc)
        .into(),
      crtime: DateTime::<Utc>::from_naive_utc_and_offset(attr.created_at, Utc)
        .into(),
      kind: if attr.is_directory {
        FileType::Directory
      } else {
        FileType::RegularFile
      },
      perm: 0o644,
      nlink: 1,
      uid: self.options.user_id,
      gid: self.options.group_id,
      rdev: 0,
      flags: 0,
      blksize: 512,
    }
  }

  fn find_child_ino(&self, parent_id: &str, child: &OsStr) -> Option<usize> {
    self.nodes.iter().position(|node| {
      let node = &node.attr;
      child.eq(node.name.as_str())
        && parent_id
          == node.parent_id.as_ref().map(|s| s.as_str()).unwrap_or("/")
    })
  }

  fn update_file_content_cache(
    &mut self,
    id: &str,
    offset: usize,
    content: &[u8],
    is_new: bool,
  ) {
    let file_idx = self.cached_files.iter().position(|file| file.id == id);
    let mut new_content = vec![0; offset + content.len()];
    match file_idx {
      Some(idx) => {
        let file = &mut self.cached_files[idx];
        new_content[0..offset].clone_from_slice(&file.content[0..offset]);
        new_content[offset..].clone_from_slice(content);
        file.content = new_content;
      }
      None => {
        new_content[offset..].clone_from_slice(content);
        self.cached_files.push(CachedFile {
          id: id.to_owned(),
          is_new,
          content: content.to_vec(),
        })
      }
    }
  }

  /// Returns ino of the new node
  fn add_new_node(
    &mut self,
    parent_id: &str,
    name: &OsStr,
    is_directory: bool,
  ) -> usize {
    let attr = DbAttribute {
      id: nanoid::nanoid!(21, &NANOID_CHARS),
      name: name.to_str().expect("Unsupported file name").to_owned(),
      parent_id: Some(parent_id.to_owned()),
      created_at: Utc::now().naive_utc(),
      is_directory,
      ..Default::default()
    };
    self.cached_files.push(CachedFile {
      id: attr.id.to_owned(),
      content: vec![],
      is_new: true,
    });
    self.nodes.push(Node {
      attr,
      ..Default::default()
    });
    self.nodes.len() - 1
  }
}

const TTL: Duration = Duration::from_secs(60);
impl fuser::Filesystem for FileSystem {
  #[tracing::instrument(skip(self, _req, reply), level = "debug")]
  fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
    let ino = ino as usize;
    if ino < self.nodes.len() {
      reply.attr(&TTL, &&self.get_fuser_attr(ino, &self.nodes[ino].attr))
    } else {
      reply.error(libc::ENOENT)
    }
  }

  #[tracing::instrument(skip(self, _req, reply), level = "debug")]
  fn lookup(
    &mut self,
    _req: &Request,
    parent: u64,
    name: &OsStr,
    reply: ReplyEntry,
  ) {
    let parent = parent as usize;
    let parent_node = match self.nodes.get(parent) {
      Some(parent) => parent,
      _ => {
        return reply.error(libc::ENOENT);
      }
    };

    let parent_node_id = parent_node.attr.id.clone();
    if parent_node.cached_at.is_none() {
      self.load_children_nodes(parent).unwrap();
    }

    let file_index = self.find_child_ino(&parent_node_id, name);
    match file_index {
      Some(ino) => {
        let node = &self.nodes[ino];
        return reply.entry(
          &TTL,
          &self.get_fuser_attr(ino, &node.attr),
          ino as u64,
        );
      }
      None => {}
    }
    reply.error(libc::ENOENT);
  }

  #[tracing::instrument(skip(self, _req, _fh, _lock, reply), level = "debug")]
  fn read(
    &mut self,
    _req: &Request,
    ino: u64,
    _fh: u64,
    offset: i64,
    size: u32,
    _flags: i32,
    _lock: Option<u64>,
    reply: ReplyData,
  ) {
    let offset = offset as usize;
    let attr = &self.nodes[ino as usize].attr;
    let file_id = attr.id.clone();
    let new_cached_file =
      self.cached_files.iter().find(|file| file.id == file_id);

    if let Some(file) = new_cached_file {
      if file.is_new {
        return reply.data(&file.content.as_slice()[offset..]);
      }
    }
    futures::executor::block_on(async move {
      let file = self.backend.read_file(file_id).await.unwrap_or_default();
      match file {
        Some(ref file) => {
          let content = file.file.content.as_bytes();
          // let decoded_content = base64::decode(content).unwrap();
          // TODO: bae64 decode content if needed
          self.update_file_content_cache(&file.id, 0, &content, false);
          let mut last_idx = offset + size as usize;
          if last_idx > content.len() {
            last_idx = content.len();
          }
          reply.data(&content[offset..last_idx]);
        }
        None => {
          reply.error(libc::ENOENT);
        }
      }
    });
  }

  #[tracing::instrument(skip(self, _req, _fh, reply), level = "debug")]
  fn readdir(
    &mut self,
    _req: &Request,
    ino: u64,
    _fh: u64,
    offset: i64,
    mut reply: ReplyDirectory,
  ) {
    let ino = ino as usize;
    let dir = &self.nodes[ino];
    let dir_id = dir.attr.id.clone();
    if dir.cached_at.is_none() {
      self.load_children_nodes(ino).unwrap();
    }
    let mut entries = Vec::with_capacity(25);
    entries.push((ino, FileType::Directory, "."));
    entries.push((ino, FileType::Directory, ".."));
    self
      .nodes
      .iter()
      .enumerate()
      .filter(|n| n.1.archived_at.is_none())
      .for_each(|(idx, node)| {
        let attr = &node.attr;
        if dir_id.as_str()
          == attr.parent_id.as_ref().map(|s| s.as_str()).unwrap_or("/")
        {
          let file_type = if attr.is_directory {
            FileType::Directory
          } else {
            FileType::RegularFile
          };
          entries.push((idx, file_type, &attr.name));
        }
      });

    for (i, entry) in entries.into_iter().enumerate().skip(offset as usize) {
      if reply.add(entry.0 as u64, (i + 1) as i64, entry.1, entry.2) {
        break;
      }
    }
    reply.ok();
  }

  #[tracing::instrument(skip(self, reply), level = "debug")]
  fn mknod(
    &mut self,
    _req: &Request<'_>,
    parent: u64,
    name: &OsStr,
    _mode: u32,
    _umask: u32,
    _rdev: u32,
    reply: ReplyEntry,
  ) {
    match self.nodes.get(parent as usize) {
      Some(parent) => {
        let parent_id = parent.attr.id.clone();
        let ino = self.add_new_node(&parent_id, name, false);
        reply.entry(
          &TTL,
          &self.get_fuser_attr(ino, &self.nodes[ino].attr),
          ino as u64,
        );
      }
      _ => {
        reply.error(libc::ENOENT);
        return;
      }
    };
  }

  fn mkdir(
    &mut self,
    _req: &Request<'_>,
    parent: u64,
    name: &OsStr,
    _mode: u32,
    _umask: u32,
    reply: ReplyEntry,
  ) {
    match self.nodes.get(parent as usize) {
      Some(parent) => {
        let parent_id = parent.attr.id.clone();
        let ino = self.add_new_node(&parent_id, name, true);
        reply.entry(
          &TTL,
          &self.get_fuser_attr(ino, &self.nodes[ino].attr),
          ino as u64,
        );
      }
      _ => {
        reply.error(libc::ENOENT);
        return;
      }
    };
  }

  #[tracing::instrument(skip(self, _req, reply), level = "debug")]
  fn unlink(
    &mut self,
    _req: &Request<'_>,
    parent: u64,
    name: &OsStr,
    reply: ReplyEmpty,
  ) {
    match self.nodes.get(parent as usize) {
      Some(parent) => {
        let parent_id = parent.attr.id.clone();
        let dir_ino = self.find_child_ino(&parent_id, name);

        if let Some(ino) = dir_ino {
          self.nodes[ino].archived_at = Some(Instant::now());
          return reply.ok();
        }
      }
      _ => {}
    };
    reply.error(libc::ENOENT);
  }

  #[tracing::instrument(skip(self, _req, reply), level = "debug")]
  fn rmdir(
    &mut self,
    _req: &Request<'_>,
    parent: u64,
    name: &OsStr,
    reply: ReplyEmpty,
  ) {
    match self.nodes.get(parent as usize) {
      Some(parent) => {
        let parent_id = parent.attr.id.clone();
        let dir_ino = self.find_child_ino(&parent_id, name);

        if let Some(ino) = dir_ino {
          self.nodes[ino].archived_at = Some(Instant::now());
          return reply.ok();
        }
      }
      _ => {}
    };

    reply.error(libc::ENOENT);
  }

  #[tracing::instrument(skip(self, reply), level = "debug")]
  fn flush(
    &mut self,
    _req: &Request<'_>,
    ino: u64,
    fh: u64,
    lock_owner: u64,
    reply: ReplyEmpty,
  ) {
    reply.ok()
  }

  #[tracing::instrument(skip(self, _req, reply), level = "debug")]
  fn getxattr(
    &mut self,
    _req: &Request<'_>,
    ino: u64,
    name: &OsStr,
    size: u32,
    reply: ReplyXattr,
  ) {
    let attr = &self.nodes[ino as usize].attr;
    let cached_file = self.cached_files.iter().find(|file| file.id == attr.id);
    if size == 0 {
      reply.size(attr.size as u32);
    } else if size < attr.size as u32 {
      match cached_file {
        Some(file) => {
          reply.data(&file.content[..]);
        }
        None => unimplemented!(),
      }
    }
  }

  #[tracing::instrument(skip(self, _req, reply), level = "debug")]
  fn write(
    &mut self,
    _req: &Request<'_>,
    ino: u64,
    fh: u64,
    offset: i64,
    data: &[u8],
    write_flags: u32,
    flags: i32,
    lock_owner: Option<u64>,
    reply: ReplyWrite,
  ) {
    let attr = &self.nodes[ino as usize].attr;
    let cached_file = self
      .cached_files
      .iter()
      .find(|file| file.id == attr.id)
      .expect("File not found");
    let file_id = cached_file.id.clone();
    let is_new = cached_file.is_new;
    self.update_file_content_cache(&file_id, offset as usize, data, is_new);
    self.nodes[ino as usize].attr.size = data.len() as i32;
    reply.written(data.len() as u32);
  }
}
