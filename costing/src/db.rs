use kvdb::{DBTransaction, KeyValueDB};
use serde::{de::DeserializeOwned, Serialize, Deserialize};
use std::{rc::Rc, io};

pub trait DatabaseValueID<ID> {
    fn id(&self) -> ID;
}

impl <T, ID> DatabaseValueID<ID> for Rc<T> 
where
    T: DatabaseValueID<ID>
{
    fn id(&self) -> ID {
        (**self).id()
    }
}

pub trait DatabaseValueRead<ID, TID>: Sized
{
    fn read_from_db<'a, DB, S, P>(id: &ID, path: P, database: &DB, db_store: &S) -> Option<Self>
    where
        DB: KeyValueDBSerde,
        S: KeyValueDBStore,
        P: Into<Option<&'a str>>;    
}

impl <T, TID> DatabaseValueRead<String, TID> for Vec<T> 
where
    T: DatabaseValueRead<TID, ()>,
    TID: DeserializeOwned
{
    fn read_from_db<'a, DB, S, P>(id: &String, path: P, database: &DB, db_store: &S) -> Option<Self>
    where
        DB: KeyValueDBSerde,
        S: KeyValueDBStore,
        P: Into<Option<&'a str>>,
    {
        let key = match path.into() {
            Some(path) => format!("{}/{}", path, id.to_string()),
            None => id.clone(),
        };

        let item_ids_option: Option<Vec<TID>> = database
            .get_deserialize(db_store, key.clone())
            .expect("unable to read from database");

        item_ids_option.map(|item_ids| {
            item_ids
                .iter()
                .map(|item_id| {
                    T::read_from_db(item_id, key.as_str(), database, db_store)
                        .expect("unable to read tab from database")
                })
                .collect()
        })
    }
}

impl <T, ID> DatabaseValueRead<ID, ()> for Rc<T> 
where
    T: DatabaseValueRead<ID, ()>,
{
    fn read_from_db<'a, DB, S, P>(id: &ID, path: P, database: &DB, db_store: &S) -> Option<Self>
    where
        DB: KeyValueDBSerde,
        S: KeyValueDBStore,
        P: Into<Option<&'a str>> {
        T::read_from_db(id, path, database, db_store).map(|v| Rc::new(v))
    }
}

pub trait DatabaseValueWrite<ID>: DatabaseValueID<ID> {
    fn write_to_db<'a, TR, S, P>(&self, path: P, transaction: &mut TR, db_store: &S)
    where
        TR: DBTransactionSerde,
        S: KeyValueDBStore,
        P: Into<Option<&'a str>>;
}

impl <T, ID> DatabaseValueWrite<ID> for Rc<T> 
where
    T: DatabaseValueWrite<ID>
{
    fn write_to_db<'a, TR, S, P>(&self, path: P, transaction: &mut TR, db_store: &S)
    where
        TR: DBTransactionSerde,
        S: KeyValueDBStore,
        P: Into<Option<&'a str>> {
        (**self).write_to_db(path, transaction, db_store);
    }
}

pub trait DatabaseValueWriteID<ID, TID> {
    fn write_to_db_id<'a, T, S, P>(&self, id: &ID, path: P, transaction: &mut T, db_store: &S)
    where
        T: DBTransactionSerde,
        S: KeyValueDBStore,
        P: Into<Option<&'a str>>;
}

impl <T, TID> DatabaseValueWriteID<String, TID> for Vec<T> 
where 
    T: DatabaseValueWrite<TID> + DatabaseValueID<TID> + Serialize,
    TID: Serialize + ToString, {
    fn write_to_db_id<'a, TR, S, P>(&self, id: &String, path: P, transaction: &mut TR, db_store: &S)
    where
        TR: DBTransactionSerde,
        S: KeyValueDBStore,
        P: Into<Option<&'a str>> {

        let key = match path.into() {
            Some(path) => format!("{}/{}", path, id.to_string()),
            None => id.to_string(),
        };

        let item_ids: Vec<TID> = self.iter().map(|item| item.id()).collect();

        transaction.put_serialize(db_store, key.clone(), item_ids);
        
        for item in self {
            item.write_to_db(key.as_str(), transaction, db_store);
        }
    }
}

/// A subset of a key-value database (a column usually).
pub trait KeyValueDBStore {
    /// The name of this store.
    fn name(&self) -> &str;
    /// The column that this store is kept in.
    fn db_col(&self) -> u32;
    /// The number of database columns.
    fn n_db_cols() -> u32;
}

pub trait KeyValueDBSerde {
    fn get_deserialize<S: KeyValueDBStore, K: AsRef<str>, V: DeserializeOwned>(
        &self,
        store: &S,
        key: K,
    ) -> io::Result<Option<V>>;
}

pub trait DBTransactionSerde {
    fn put_serialize<S: KeyValueDBStore, K: AsRef<str>, V: Serialize>(
        &mut self,
        db_store: &S,
        key: K,
        value: V,
    );
}

impl KeyValueDBSerde for &dyn KeyValueDB {
    fn get_deserialize<S: KeyValueDBStore, K: AsRef<str>, V: DeserializeOwned>(
        &self,
        db_store: &S,
        key: K,
    ) -> io::Result<Option<V>> {
        self.get(db_store.db_col(), key.as_ref().as_bytes())
            .map(|value_option| {
                value_option.map(|value_bytes| {
                    serde_json::from_slice(&value_bytes)
                        .expect("unable to desrialize database value")
                })
            })
    }
}

impl DBTransactionSerde for DBTransaction {
    fn put_serialize<S: KeyValueDBStore, K: AsRef<str>, V: Serialize>(
        &mut self,
        store: &S,
        key: K,
        value: V,
    ) {
        let value_string =
            serde_json::to_string(&value).expect("unable to serialize database value");

        self.put(
            store.db_col(),
            key.as_ref().as_bytes(),
            value_string.as_bytes(),
        )
    }
}