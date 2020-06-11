use kvdb::{DBTransaction, KeyValueDB};
use serde::{de::DeserializeOwned, Serialize, Deserialize};
use std::{rc::Rc, io};

pub trait DatabaseValueID {
    type ID: ToString;
    fn id(&self) -> Self::ID;
}

pub trait DatabaseValueRead: Sized
{
    type ID: ToString;
    fn read_from_db<'a, DB, S, P>(id: &Self::ID, path: P, database: &DB, db_store: &S) -> Option<Self>
    where
        DB: KeyValueDBSerde,
        S: KeyValueDBStore,
        P: Into<Option<&'a str>>;    
}

pub trait DatabaseValueWrite: DatabaseValueID {
    fn write_to_db<'a, T, S, P>(&self, path: P, transaction: &mut T, db_store: &S)
    where
        T: DBTransactionSerde,
        S: KeyValueDBStore,
        P: Into<Option<&'a str>>;
}

pub trait DatabaseValueWriteID {
    type ID;
    fn write_to_db_id<'a, T, S, P>(&self, id: Self::ID, path: P, transaction: &mut T, db_store: &S)
    where
        T: DBTransactionSerde,
        S: KeyValueDBStore,
        P: Into<Option<&'a str>>;
}


impl <T> DatabaseValueRead for Vec<T> 
where 
    T: DatabaseValueRead + DeserializeOwned,
    T::ID: DeserializeOwned,
{
    type ID = String;
    fn read_from_db<'a, DB, S, P>(id: &Self::ID, path: P, database: &DB, db_store: &S) -> Option<Self>
    where
        DB: KeyValueDBSerde,
        S: KeyValueDBStore,
        P: Into<Option<&'a str>>,
    {
        let key = match path.into() {
            Some(path) => format!("{}/{}", path, id),
            None => id.clone(),
        };

        let item_ids_option: Option<Vec<T::ID>> = database
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

impl <T> DatabaseValueWriteID for Vec<T> 
where 
    T: DatabaseValueWrite + DatabaseValueID + Serialize,
    <T as DatabaseValueID>::ID: Serialize, {
    type ID = String;
    fn write_to_db_id<'a, TR, S, P>(&self, id: Self::ID, path: P, transaction: &mut TR, db_store: &S)
    where
        TR: DBTransactionSerde,
        S: KeyValueDBStore,
        P: Into<Option<&'a str>> {

        let key = match path.into() {
            Some(path) => format!("{}/{}", path, id),
            None => id.clone(),
        };

        let item_ids: Vec<T::ID> = self.iter().map(|item| item.id()).collect();

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