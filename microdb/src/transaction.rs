use std::sync::{RwLockWriteGuard};
use  crate::Database;


pub enum TransactionEntry
{
    Existing(u64, usize, Vec<u8>),
    NotExisting(u64, usize)
}

pub struct TransactionManager
{    
    transaction_id: usize,    
    entries: Vec<TransactionEntry>
}

impl TransactionManager
{
    pub fn new() -> Self
    {        
        return Self { transaction_id: 1, entries: Vec::new() };
    }

    pub fn begin_transaction(&mut self)
    {        
        self.transaction_id += 1;
    }

    pub fn commit_transaction(&mut self)
    {        
        self.entries.clear();        
    }

    pub fn rollback_transaction<D>(&mut self, db: &mut RwLockWriteGuard<'_, D>) where D: Database
    {        
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