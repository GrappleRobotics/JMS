use std::marker::PhantomData;

use super::types;
use super::types::Key;

pub trait TableType: serde::Serialize + serde::de::DeserializeOwned {
  const TABLE: &'static str;
  type Id: types::Key;

  fn id(&self) -> Option<Self::Id>;
  fn set_id(&mut self, id: Self::Id);

  fn table(db: &super::Store) -> super::Result<Table<Self>> {
    Table::<Self>::instance(db)
  }

  fn insert<'a>(&'a mut self, db: &super::Store) -> super::Result<&'a Self> {
    let t = Self::table(db)?;
    t.insert(db, self)?;
    Ok(self)
  }

  fn remove(&self, db: &super::Store) -> super::Result<()> {
    let t = Self::table(db)?;
    t.remove(self.id().ok_or(anyhow::anyhow!("Can't remove an element without an ID!"))?)?;
    Ok(())
  }

  fn remove_by<X: Into<Self::Id>>(id: X, db: &super::Store) -> super::Result<Option<Self>> {
    let t = Self::table(db)?;
    t.remove(id)
  }

  fn all(db: &super::Store) -> super::Result<Vec<Self>> {
    let t = Self::table(db)?;
    t.all()
  }

  fn get<X: Into<Self::Id>>(id: X, db: &super::Store) -> super::Result<Option<Self>> {
    let t = Self::table(db)?;
    t.get(id)
  }

  fn get_or_err<X: Into<Self::Id> + std::fmt::Debug + Clone>(id: X, db: &super::Store) -> super::Result<Self> {
    match Self::get(id.clone(), db)? {
      Some(s) => Ok(s),
      None => Err(anyhow::anyhow!("No Record with ID: {:?}", id)),
    }
  }

  fn clear(db: &super::Store) -> super::Result<()> {
    let t = Self::table(db)?;
    t.clear()
  }
}

#[derive(Clone)]
pub struct Table<T: TableType>(pub sled::Tree, PhantomData<T>);

#[allow(dead_code)]
impl<'a, T: TableType> Table<T> {  
  pub fn new(t: sled::Tree) -> Self {
    Self(t, PhantomData)
  }

  pub fn instance(db: &super::Store) -> super::Result<Self> {
    db.table(T::TABLE)
  }

  pub fn tree(&'a self) -> &'a sled::Tree {
    &self.0
  }

  pub fn contains<X: Into<T::Id>>(&self, key: X) -> super::Result<bool> {
    Ok(self.0.contains_key(key.into())?)
  }

  pub fn get<X: Into<T::Id>>(&self, key: X) -> super::Result<Option<T>> {
    let v = self.0.get(key.into())?;
    Ok(match v {
      Some(v) => Some(serde_json::from_slice(&v)?),
      None => None,
    })
  }

  pub fn insert<'v>(&self, db: &super::Store, value: &'v mut T) -> super::Result<&'v T> {
    let key = match value.id() {
      Some(id) => id,
      None => {
        let new_id = T::Id::generate(db);
        value.set_id(new_id.clone());
        new_id
      },
    };

    let bytes = serde_json::to_vec(&value)?;
    self.0.insert(key.as_ref(), bytes)?;
    Ok(value)
  }

  pub fn remove<X: Into<T::Id>>(&self, key: X) -> super::Result<Option<T>> {
    let removed = self.0.remove(key.into())?;
    Ok(match removed {
      Some(v) => Some(serde_json::from_slice(&v)?),
      None => None,
    })
  }

  pub fn first(&self) -> super::Result<Option<T>> {
    let f = self.0.first()?;
    Ok(match f {
      Some((_, v)) => Some(serde_json::from_slice(&v)?),
      None => None,
    })
  }

  pub fn iter(&self) -> Iter<T> {
    Iter(self.0.iter(), PhantomData)
  }

  pub fn all(&self) -> super::Result<Vec<T>> {
    self.iter().collect()
  }

  pub fn watch_prefix<X: Into<T::Id>>(&self, prefix: X) -> Watch<T> {
    Watch(self.0.watch_prefix(prefix.into()), PhantomData)
  }

  pub fn watch_all(&self) -> Watch<T> {
    Watch(self.0.watch_prefix(vec![]), PhantomData)
  }

  pub fn apply_batch(&self, batch: Batch<T>) -> super::Result<()> {
    self.0.apply_batch(batch.0)?;
    Ok(())
  }

  // TODO: Transaction

  pub fn flush(&self) -> super::Result<usize> {
    Ok(self.0.flush()?)
  }

  pub async fn flush_async(&self) -> super::Result<usize> {
    Ok(self.0.flush_async().await?)
  }

  pub fn len(&self) -> usize {
    self.0.len()
  }

  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }

  pub fn clear(&self) -> super::Result<()> {
    self.0.clear()?;
    Ok(())
  }
}

pub struct Iter<T>(sled::Iter, PhantomData<T>);

impl<T: TableType> Iterator for Iter<T> {
  type Item = super::Result<T>;

  fn next(&mut self) -> Option<Self::Item> {
    match self.0.next() {
      Some(Ok((_, v))) => {
        match serde_json::from_slice(&v) {
          Ok(dat) => Some(Ok(dat)),
          Err(e) => Some(Err(e.into())),
        }
      },
      Some(Err(e)) => Some(Err(e.into())),
      None => None,
    }
  }
}

impl<T: TableType> DoubleEndedIterator for Iter<T> {
  fn next_back(&mut self) -> Option<Self::Item> {
    match self.0.next_back() {
      Some(Ok((_, v))) => {
        match serde_json::from_slice(&v) {
          Ok(dat) => Some(Ok(dat)),
          Err(e) => Some(Err(e.into())),
        }
      },
      Some(Err(e)) => Some(Err(e.into())),
      None => None,
    }
  }
}

pub struct Watch<T>(sled::Subscriber, PhantomData<T>);

pub enum WatchEvent<T: TableType> {
  Insert(T),
  Remove(T::Id)
}

impl<T: TableType> Iterator for Watch<T> {
  type Item = super::Result<WatchEvent<T>>;

  fn next(&mut self) -> Option<Self::Item> {
    match self.0.next() {
      Some(sled::Event::Insert { key: _, value }) => {
        match serde_json::from_slice(&value) {
          Ok(dat) => Some(Ok(WatchEvent::Insert(dat))),
          Err(e) => Some(Err(e.into())),
        }
      },
      Some(sled::Event::Remove { key }) => {
        let a: &[u8] = key.as_ref();
        Some(Ok(WatchEvent::Remove(T::Id::from_raw(a))))
      },
      None => None,
    }
  }
}
pub struct Batch<T>(sled::Batch, PhantomData<T>);

#[allow(dead_code)]
impl<T: TableType> Batch<T> {
  pub fn new() -> Self {
    Self(sled::Batch::default(), PhantomData)
  }

  pub fn insert<'a>(&mut self, db: &super::Store, value: &'a mut T) -> super::Result<&'a T> {
    let key = match value.id() {
      Some(id) => id,
      None => {
        let new_id = T::Id::generate(db);
        value.set_id(new_id.clone());
        new_id
      },
    };

    let bytes = serde_json::to_vec(&value)?;
    self.0.insert(key.as_ref(), bytes);
    Ok(value)
  }

  pub fn remove<X: Into<T::Id>>(&mut self, key: X) -> super::Result<()> {
    let a: T::Id = key.into();
    self.0.remove(a.as_ref());
    Ok(())
  }
}