use super::table::{Table, TableType};

#[allow(dead_code)]
pub struct Store {
  config: sled::Config,
  db: sled::Db,
}

#[allow(dead_code)]
impl Store {
  pub fn new(config: sled::Config) -> super::Result<Self> {
    Ok(Self {
      db: config.open()?,
      config
    })
  }

  pub fn generate_id(&self) -> super::Result<u64> { 
    Ok(self.db.generate_id()?)
  }

  pub fn table<T: TableType>(&self, name: &str) -> super::Result<Table<T>> {
    Ok(Table::new(self.db.open_tree(name)?))
  }

  pub fn drop(&self, name: &str) -> super::Result<()> {
    self.db.drop_tree(name)?;
    Ok(())
  }

  pub fn export(&self) -> Vec<(Vec<u8>, Vec<u8>, impl Iterator<Item = Vec<Vec<u8>>>)> {
    self.db.export()
  }

  pub fn import(&self, export: Vec<(Vec<u8>, Vec<u8>, impl Iterator<Item = Vec<Vec<u8>>>)>) {
    self.db.import(export)
  }
}