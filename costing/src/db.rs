use kvdb::{DBTransaction, KeyValueDB};
use serde::{de::DeserializeOwned, Serialize};
use std::io;

pub trait DatabaseValue<ID>: Sized
where
    ID: ToString,
{
    fn read_from_db<DB, S>(path: &str, id: ID, db: &DB, db_store: &S) -> Option<Self>
    where
        DB: KeyValueDBSerde,
        S: KeyValueDBStore;

    fn write_to_db<T, S>(&self, path: &str, transaction: &mut T, db_store: &S)
    where
        T: DBTransactionSerde,
        S: KeyValueDBStore;
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
        store: &S,
        key: K,
        value: V,
    );
}

impl KeyValueDBSerde for &dyn KeyValueDB {
    fn get_deserialize<S: KeyValueDBStore, K: AsRef<str>, V: DeserializeOwned>(
        &self,
        store: &S,
        key: K,
    ) -> io::Result<Option<V>> {
        self.get(store.db_col(), key.as_ref().as_bytes())
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
