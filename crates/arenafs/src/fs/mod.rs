use std::ffi::OsStr;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant, SystemTime};

use chrono::{DateTime, Utc};
use fuser::{
  FileType, MountOption, ReplyAttr, ReplyData, ReplyDirectory, ReplyEmpty,
  ReplyEntry, ReplyWrite, ReplyXattr, Request,
};

mod cache;

pub use self::cache::FilesCache;
use self::cache::{CachedFile, Node, NANOID_CHARS};
use crate::backend::{Backend, DbAttribute};
use crate::error::Error;

#[derive(Clone)]
pub struct Options {
  pub root_id: Option<String>,
  pub user_id: u32,
  pub group_id: u32,
}

#[derive(Clone)]
pub struct FileSystem {
  options: Options,
  backend: Arc<dyn Backend>,
  cache: Arc<Mutex<FilesCache>>,
}

impl FileSystem {
  pub async fn with_backend(
    options: Options,
    cache: Arc<Mutex<FilesCache>>,
    backend: Arc<dyn Backend>,
  ) -> Result<Self, Error> {
    let mut fs = Self {
      options,
      backend,
      cache,
    };

    let root_id = fs.options.root_id.clone();
    fs.reset(root_id).await?;
    Ok(fs)
  }

  pub async fn reset(&mut self, root_id: Option<String>) -> Result<(), Error> {
    let root = self
      .backend
      .fetch_node(root_id.as_ref())
      .await?
      .unwrap_or_else(|| DbAttribute {
        id: None,
        parent_id: Some("/dev/null".to_owned()),
        is_directory: true,
        ..Default::default()
      });
    let mut cache = self.cache();
    cache.nodes.push(Node {
      attr: root,
      ..Default::default()
    });
    drop(cache);
    self.load_children_nodes(1)?;
    Ok(())
  }

  pub fn mount(
    self,
    mountpoint: &str,
    options: &Vec<MountOption>,
  ) -> Result<(), Error> {
    fuser::mount2(self, mountpoint, options)?;
    Ok(())
  }

  #[tracing::instrument(skip(self), level = "debug")]
  // Before fetching a dir, the dir's attr should already be in nodes
  pub fn load_children_nodes(&mut self, ino: usize) -> Result<(), Error> {
    let mut cache = self.cache();
    let node = &mut cache.nodes[ino];
    let id = node.attr.id.clone();
    let children_nodes = futures::executor::block_on(async {
      self.backend.fetch_children(id.as_ref()).await
    })
    .expect("Error loading children nodes");

    node.cached_at = Some(Instant::now());
    children_nodes.into_iter().for_each(|child| {
      let existing_ino = cache.find_node_index(child.id.as_ref());
      let node = Node {
        attr: child,
        ..Default::default()
      };
      if let Some(ino) = existing_ino {
        cache.nodes[ino] = node;
      } else {
        cache.nodes.push(node);
      }
    });
    Ok(())
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

  fn find_child_ino(
    &self,
    parent_id: Option<&String>,
    child: &OsStr,
  ) -> Option<usize> {
    let cache = self.cache();
    cache.nodes.iter().position(|node| {
      let node = &node.attr;
      child.eq(node.name.as_str()) && parent_id == node.parent_id.as_ref()
    })
  }

  fn update_file_content_cache(
    &mut self,
    id: &str,
    offset: usize,
    content: &[u8],
    is_new: bool,
  ) {
    let mut cache = self.cache();
    let file = cache.find(id);
    let mut new_content = vec![0; offset + content.len()];
    match file {
      Some(file) => {
        new_content[0..offset].clone_from_slice(&file.content[0..offset]);
        new_content[offset..].clone_from_slice(content);
        file.content = new_content;
        file.updated_at = Instant::now();
      }
      None => {
        new_content[offset..].clone_from_slice(content);
        cache.add(CachedFile {
          id: id.to_owned(),
          is_new,
          content: content.to_vec(),
          updated_at: Instant::now(),
        })
      }
    }
  }

  /// Returns ino of the new node
  fn add_new_node(
    &mut self,
    parent_id: Option<String>,
    name: &OsStr,
    is_directory: bool,
  ) -> usize {
    let mut cache = self.cache();
    let attr = DbAttribute {
      id: Some(nanoid::nanoid!(21, &NANOID_CHARS)),
      name: name.to_str().expect("Unsupported file name").to_owned(),
      parent_id,
      created_at: Utc::now().naive_utc(),
      is_directory,
      ..Default::default()
    };
    cache.add(CachedFile {
      id: attr.id.to_owned().unwrap(),
      content: vec![],
      is_new: true,
      updated_at: Instant::now(),
    });
    cache.nodes.push(Node {
      attr,
      ..Default::default()
    });
    cache.nodes.len() - 1
  }

  fn cache(&self) -> MutexGuard<'_, FilesCache> {
    self.cache.lock().unwrap()
  }
}

const TTL: Duration = Duration::from_secs(5);
impl fuser::Filesystem for FileSystem {
  #[tracing::instrument(skip(self, _req, reply), level = "debug")]
  fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
    let cache = self.cache();
    let ino = ino as usize;
    if ino < cache.nodes.len() {
      reply.attr(&TTL, &&self.get_fuser_attr(ino, &cache.nodes[ino].attr))
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
    let cache = self.cache();
    let parent = parent as usize;
    let parent_node = match cache.nodes.get(parent) {
      Some(parent) => parent,
      _ => {
        return reply.error(libc::ENOENT);
      }
    };

    let parent_node_id = parent_node.attr.id.clone();

    if parent_node.cached_at.is_none() {
      drop(cache);
      self.load_children_nodes(parent).unwrap();
    } else {
      drop(cache);
    }

    let file_index = self.find_child_ino(parent_node_id.as_ref(), name);
    match file_index {
      Some(ino) => {
        let cache = self.cache();
        let node = &cache.nodes[ino];
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
    let mut cache = self.cache();
    let offset = offset as usize;
    let attr = &cache.nodes[ino as usize].attr;

    let file_id = attr.id.clone().unwrap();
    let new_cached_file = cache.find(&file_id);

    if let Some(file) = new_cached_file {
      if file.is_new {
        return reply.data(&file.content.as_slice()[offset..]);
      }
    }

    drop(cache);
    futures::executor::block_on(async move {
      let file = self.backend.read_file(file_id).await.unwrap_or_default();
      match file {
        Some(ref file) => {
          let content = file.file.content.as_bytes();
          let content = base64::decode(&content).unwrap();
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
    let cache = self.cache();
    let ino = ino as usize;
    let dir = &cache.nodes[ino];
    let dir_id = dir.attr.id.clone();
    if dir.cached_at.is_none() {
      drop(cache);
      self.load_children_nodes(ino).unwrap();
    } else {
      drop(cache);
    }
    let mut entries = Vec::with_capacity(25);
    entries.push((ino, FileType::Directory, "."));
    entries.push((ino, FileType::Directory, ".."));

    let cache = self.cache();
    cache
      .nodes
      .iter()
      .enumerate()
      .filter(|n| n.1.archived_at.is_none())
      .for_each(|(idx, node)| {
        let attr = &node.attr;
        if dir_id.as_ref() == attr.parent_id.as_ref() {
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
    let cache = self.cache();
    match cache.nodes.get(parent as usize) {
      Some(parent) => {
        let parent_id = parent.attr.id.clone();
        drop(cache);
        let ino = self.add_new_node(parent_id, name, false);
        let cache = self.cache();
        reply.entry(
          &TTL,
          &self.get_fuser_attr(ino, &cache.nodes[ino].attr),
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
    let cache = self.cache();
    match cache.nodes.get(parent as usize) {
      Some(parent) => {
        let parent_id = parent.attr.id.clone();
        drop(cache);
        let ino = self.add_new_node(parent_id, name, true);
        let cache = self.cache();
        reply.entry(
          &TTL,
          &self.get_fuser_attr(ino, &cache.nodes[ino].attr),
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
    let mut cache = self.cache();
    match cache.nodes.get(parent as usize) {
      Some(parent) => {
        let parent_id = parent.attr.id.clone();
        let dir_ino = self.find_child_ino(parent_id.as_ref(), name);

        if let Some(ino) = dir_ino {
          cache.nodes[ino].archived_at = Some(Instant::now());
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
    let mut cache = self.cache();
    match cache.nodes.get(parent as usize) {
      Some(parent) => {
        let parent_id = parent.attr.id.clone();
        let dir_ino = self.find_child_ino(parent_id.as_ref(), name);

        if let Some(ino) = dir_ino {
          cache.nodes[ino].archived_at = Some(Instant::now());
          return reply.ok();
        }
      }
      _ => {}
    };

    reply.error(libc::ENOENT);
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
    let mut cache = self.cache();
    let attr = &cache.nodes[ino as usize].attr;
    let size = attr.size as u32;
    let file_id = attr.id.as_ref().unwrap().clone();
    let cached_file = cache.find(&file_id);
    if size == 0 {
      reply.size(size as u32);
    } else if size < size as u32 {
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
    let (file_id, is_new) = {
      let mut cache = self.cache();
      let attr = &cache.nodes[ino as usize].attr.clone();
      let cached_file = cache
        .find(attr.id.as_ref().unwrap())
        .expect("File not found");
      let file_id = cached_file.id.clone();
      let is_new = cached_file.is_new;
      (file_id, is_new)
    };

    self.update_file_content_cache(&file_id, offset as usize, data, is_new);
    let mut cache = self.cache();
    cache.nodes[ino as usize].attr.size = data.len() as i32;

    let node = &cache.nodes[ino as usize];
    let backend = self.backend.clone();
    let res = futures::executor::block_on(async move {
      backend.write_file(&node.attr, &data).await
    });

    if let Err(_) = res {
      reply.error(libc::ENOENT);
      return;
    }

    reply.written(data.len() as u32);
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
    reply.ok();
  }
}
