use std::{sync::{RwLockWriteGuard}, fmt::{Display, self}};

use log::debug;

use  crate::Database;


pub enum TransactionEntry
{
    Existing(u64, usize, Vec<u8>),
    NotExisting(u64, usize)
}

impl Display for TransactionEntry
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            TransactionEntry::Existing(id, _, _ ) => { write!(f, "Existing ({})", id) },
            TransactionEntry::NotExisting(id, _ ) => { write!(f, "Not Existing ({})", id) }
        }
    }
}

pub struct TransactionManager
{    
    transaction_id: usize,    
    entries: Vec<TransactionEntry>,
    transaction_running: bool
}

impl TransactionManager
{
    pub fn new() -> Self
    {        
        return Self { transaction_id: 1, entries: Vec::new(), transaction_running: false };
    }

    pub fn is_transaction_running(&self) -> bool
    {
        self.transaction_running
    }

    pub fn begin_transaction(&mut self)
    {
        debug!("Begin Transaction ({})", self.transaction_id + 1);

        self.transaction_running = true;
        self.transaction_id += 1;
        
    }

    pub fn commit_transaction(&mut self)
    {
        debug!("Commit Transaction ({})", self.transaction_id);

        self.transaction_running = false;
        self.entries.clear();        
    }

    pub fn rollback_transaction<D>(&mut self, db: &mut RwLockWriteGuard<'_, D>) where D: Database
    {
        debug!("Rollback Transaction ({})", self.transaction_id);
        
        for transaction_entry in &self.entries
        {
            match transaction_entry
            {
                TransactionEntry::Existing(table_id, id, state) =>
                {
                    let table = db.get_table_mut(*table_id);
                    table.rollback_to_existing(*id, state);
                },
                TransactionEntry::NotExisting(table_id, id) =>
                {
                    let table = db.get_table_mut(*table_id);
                    table.rollback_to_not_existing(*id);
                }
            }
        }
        self.entries.clear();
    }

    pub fn add_entry(&mut self, entry: TransactionEntry)
    {        
        self.entries.push(entry);        
    }

    pub fn get_transaction_id(&self) -> usize
    {
        self.transaction_id
    }

}