use std::marker::PhantomData;

use super::{FromRaw, ToRaw, Json};

pub trait TableType: serde::Serialize + serde::de::DeserializeOwned {
  const TABLE: &'static str;
  type Id: ToRaw + FromRaw;

  fn id(&self) -> Option<Self::Id>;

  fn generate_id(&mut self, _: &super::Store) -> super::Result<()> { Ok(()) }

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
    Ok(t.all()?)
  }
  
  fn keys(db: &super::Store) -> super::Result<Vec<Self::Id>> {
    let t = Self::table(db)?;
    t.iter_keys().collect()
  }

  fn get<X: Into<Self::Id>>(id: X, db: &super::Store) -> super::Result<Option<Self>> {
    let t = Self::table(db)?;
    t.get(id)
  }

  fn get_all<I, X>(ids: I, db: &super::Store) -> super::Result<Vec<Option<Self>>>
  where
    X: Into<Self::Id> + Clone,
    I: Iterator<Item = X>
  {
    let t = Self::table(db)?;
    ids.map(|id| t.get(id)).collect()
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

impl<T: TableType> ToRaw for T {
  fn to_raw(&self) -> Vec<u8> {
    Json(self).to_raw()
  }
}

impl<T: TableType> FromRaw for T {
  fn from_raw(r: &[u8]) -> Self {
    Json::<T>::from_raw(r).0
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
    let k: T::Id = key.into();
    Ok(self.0.contains_key(k.to_raw())?)
  }

  pub fn get<X: Into<T::Id>>(&self, key: X) -> super::Result<Option<T>> {
    let k: T::Id = key.into();
    Ok(self.0.get(k.to_raw())?.map(|v| T::from_raw(&v)))
  }

  pub fn insert<'v>(&self, db: &super::Store, value: &'v mut T) -> super::Result<&'v T> {
    let key = match value.id() {
      Some(id) => id,
      None => {
        value.generate_id(&db)?;

        match value.id() {
          Some(id) => id,
          None => anyhow::bail!("No ID given or generated during insert")
        }
      },
    };

    self.0.insert(key.to_raw(), value.to_raw())?;
    Ok(value)
  }

  pub fn remove<X: Into<T::Id>>(&self, key: X) -> super::Result<Option<T>> {
    let k: T::Id = key.into();
    let removed = self.0.remove(k.to_raw())?;
    Ok(removed.map(|r| T::from_raw(&r)))
  }

  pub fn first(&self) -> super::Result<Option<T>> {
    Ok(self.0.first()?.map(|(_, v)| T::from_raw(&v)))
  }

  pub fn iter_keys(&self) -> impl DoubleEndedIterator<Item = super::Result<T::Id>> {
    self.0.iter().keys().map(|el| el.map_err(|e| anyhow::anyhow!(e)).map(|v| T::Id::from_raw(&v)))
  }

  pub fn iter_values(&self) -> impl DoubleEndedIterator<Item = super::Result<T>> {
    self.0.iter().values().map(|el| el.map_err(|e| anyhow::anyhow!(e)).map(|v| T::from_raw(&v)))
  }

  pub fn iter(&self) -> impl DoubleEndedIterator<Item = super::Result<(T::Id, T)>> {
    // Iter(self.0.iter(), PhantomData)
    self.0.iter().map(|el| el.map_err(|e| anyhow::anyhow!(e)).map(|(k, v)| ( T::Id::from_raw(&k), T::from_raw(&v) ) ) )
  }

  pub fn all(&self) -> super::Result<Vec<T>> {
    self.iter_values().collect()
  }

  pub fn watch_prefix<X: Into<T::Id>>(&self, prefix: X) -> Watch<T> {
    let pfx: T::Id = prefix.into();
    Watch(self.0.watch_prefix(pfx.to_raw()), PhantomData)
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

// pub struct FromRawIter<I: Iterator, T: FromRaw>(I, PhantomData<T>);

// impl<T: FromRaw, I: Iterator> Iterator for FromRawIter<I, T> 
// where
//   T: FromRaw,
//   I: Iterator,
//   <I as Iterator>::Item: super::Result<sled::IVec>
// {
//   type Item = super::Result<T>;

//   fn next(&mut self) -> Option<Self::Item> {
//     match self.0.next() {
//       Some(v) => todo!(),
//       None => None,
//     }
//   }
// }

// impl<T: TableType> Iterator for Iter<T> {
//   type Item = super::Result<T>;

//   fn next(&mut self) -> Option<Self::Item> {
//     match self.0.next() {
//       Some(Ok((_, v))) => {
//         match serde_json::from_slice(&v) {
//           Ok(dat) => Some(Ok(dat)),
//           Err(e) => Some(Err(e.into())),
//         }
//       },
//       Some(Err(e)) => Some(Err(e.into())),
//       None => None,
//     }
//   }
// }

// impl<T: TableType> DoubleEndedIterator for Iter<T> {
//   fn next_back(&mut self) -> Option<Self::Item> {
//     match self.0.next_back() {
//       Some(Ok((_, v))) => {
//         match serde_json::from_slice(&v) {
//           Ok(dat) => Some(Ok(dat)),
//           Err(e) => Some(Err(e.into())),
//         }
//       },
//       Some(Err(e)) => Some(Err(e.into())),
//       None => None,
//     }
//   }
// }

pub struct Watch<T>(sled::Subscriber, PhantomData<T>);

impl<T: TableType> Watch<T> {
  pub async fn get(&mut self) -> super::Result<WatchEvent<T>> {
    let i = (&mut self.0).await;
    match i {
      Some(sled::Event::Insert { key: _, value }) => {
        Ok(WatchEvent::Insert(T::from_raw(&value)))
      },
      Some(sled::Event::Remove { key }) => {
        Ok(WatchEvent::Remove(T::Id::from_raw(&key)))
      },
      None => Err(anyhow::anyhow!("No Data!")),
    }
  }
}

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
        value.generate_id(&db)?;

        match value.id() {
          Some(id) => id,
          None => anyhow::bail!("No ID given or generated during insert")
        }
      },
    };

    let bytes = serde_json::to_vec(&value)?;
    self.0.insert(key.to_raw(), bytes);
    Ok(value)
  }

  pub fn remove<X: Into<T::Id>>(&mut self, key: X) -> super::Result<()> {
    let a: T::Id = key.into();
    self.0.remove(a.to_raw());
    Ok(())
  }
}