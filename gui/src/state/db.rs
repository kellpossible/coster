use costing::db::KeyValueDBStore;

#[derive(Debug)]
pub enum CosterClientDBStore {
    /// Used for storing general application variables that need to be
    /// persisted (such as user selected language).
    General,
    /// Used for storing [costing::Tab]s.
    Tabs,
}

impl KeyValueDBStore for CosterClientDBStore {
    fn name(&self) -> &str {
        match self {
            CosterClientDBStore::General => "General",
            CosterClientDBStore::Tabs => "Tabs",
        }
    }
    fn db_col(&self) -> u32 {
        match self {
            CosterClientDBStore::General => 0,
            CosterClientDBStore::Tabs => 1,
        }
    }
    fn n_db_cols() -> u32 {
        2
    }
}
