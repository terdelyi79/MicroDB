use std::sync::{Arc, Mutex};
use std::ops::{Deref, DerefMut};
use std::fmt::{Display, Formatter};
use log::debug;
use serde::{Serialize, de::DeserializeOwned};
use crate::transaction::{TransactionManager, TransactionEntry};

// Entity is a smart pointer to struct stored in a MicroDb table
pub struct Entity<T> where T : Serialize + DeserializeOwned
{
    // Unique identifier of the entity
    id: usize,
    // Unique identifier of the table the entity is stored in
    table_id: u64,
    // The struct itself stored as an entity
    val: T,
    // Reference to the transaction manager, what handles the transaction log in the memory
    transaction_manager: Arc<Mutex<TransactionManager>>,
    // Identifier of the last transacion the entity was modified in
    last_modified_transaction_id: usize
}

impl<T> Entity<T> where T : Serialize + DeserializeOwned
{
    // Create a new entity
    pub fn new(id: usize, table_id: u64, val: T, transaction_manager: Arc<Mutex<TransactionManager>>) -> Self
    {
        Entity { id, table_id, val, transaction_manager, last_modified_transaction_id: 0 }
    }

    // Get the unique identifier of entity
    pub fn get_id(&self) -> usize
    {
        self.id
    }
}

impl<T> Deref for Entity<T> where T : Serialize + DeserializeOwned
{
    type Target = T;

    // Dereference returns the struct itself stored in the entity
    fn deref(&self) -> &Self::Target
    {
        &self.val
    }
}

impl<T> DerefMut for Entity<T> where T : Serialize + DeserializeOwned
{
    // Mutable dereference not returns the stored struct only, but stores the original version of the struct in the transaction manager if not already done
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        let mut locked_transaction_manager = self.transaction_manager.lock().unwrap();
        
        if locked_transaction_manager.is_transaction_running()
        {
            // If original version of the entity was not stored for this transaction yet
            if locked_transaction_manager.get_transaction_id() > self.last_modified_transaction_id
            {
                // Add an entry to the transaction log indicating that entity did not exist before thre transaction
                debug!("Add transaction entry for an existing entity (Table Id: {}, Entity Id: {})", self.table_id, self.id);

                // Add an "Existing" transaction entry indicating that the entity existed before the transaction
                locked_transaction_manager.add_entry(TransactionEntry::Existing(
                    self.table_id,
                    self.id,
                    // Transaction entry contains the whole entity in serialized form
                    bincode::serialize(&self.val).unwrap()
                ));

                // Transaction id is stored in the entity, because no other transaction entry is needed in the same transaction
                self.last_modified_transaction_id = locked_transaction_manager.get_transaction_id();
            }
        }

        return &mut self.val
    }
}

impl<T> Display for Entity<T> where T : Display + Serialize + DeserializeOwned
{
    // Display implementation of entity returns the same as in the original struct stored in the entity
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result
    {     
        self.val.fmt(f)        
    }
}