use arenafs::{Backend, DbAttribute, DbFile, Error};

pub struct NoopBackend {}

#[async_trait::async_trait]
impl Backend for NoopBackend {
  async fn fetch_node(
    &self,
    _id: Option<&String>,
  ) -> Result<Option<DbAttribute>, Error> {
    Ok(None)
  }

  async fn fetch_children(
    &self,
    _id: Option<&String>,
  ) -> Result<Vec<DbAttribute>, Error> {
    Ok(vec![])
  }

  async fn read_file(&self, _id: String) -> Result<Option<DbFile>, Error> {
    Ok(None)
  }

  async fn write_file(
    &self,
    _attr: &DbAttribute,
    _content: &[u8],
  ) -> Result<(), Error> {
    Ok(())
  }
}
