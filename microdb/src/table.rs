use log::debug;
use serde::{Serialize, de::DeserializeOwned};
use std::collections::{HashMap, hash_map::Values, hash_map::ValuesMut};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::sync::{Arc, Mutex};
use crate::entity::Entity;
use crate::transaction::{TransactionManager, TransactionEntry};

// Trait defining rollback related functions for tables (used by the transaction manager)
pub trait TableBase
{
    // Revert an entity to its original state, what already existed before the transaction
    fn rollback_to_existing(&mut self, id: usize, state: &Vec<u8>);

    // Remove and entity what did not exist before thre transaction
    fn rollback_to_not_existing(&mut self, id: usize);
}

// A table, what can store specific type of entities
pub struct Table<T> where T : Serialize + DeserializeOwned
{
    // Name of the table
    name: &'static str,
    // Unique identifier of table
    id: u64,
    // Hash map to store all entities by their unique identifiers
    rows: HashMap<usize, Entity<Box<T>>>,
    // First free unique identifier in the table
    first_free_id: usize,
    // Transaction manager
    transaction_manager: Arc<Mutex<TransactionManager>>
}

impl<T> Table<T> where T : Serialize + DeserializeOwned
{
    // Create a new table
    pub fn new(name: &'static str, transaction_manager: Arc<Mutex<TransactionManager>>) -> Self
    {
        // Unique identifier of table is a hash generated from its name
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        let id = hasher.finish();

        return Self {name, id, rows: HashMap::new(), first_free_id: 1, transaction_manager };
    }
    
    // Returns the unique identifier of table
    pub fn get_id(&self) -> u64
    {
        self.id
    }

    // Gets an item from the table by identifier
    pub fn get(&self, id: usize) -> Option<&Entity<Box<T>>>
    {
        self.rows.get(&id)
    }

    // Get an item from the table as mutable byidentifirt
    pub fn get_mut(&mut self, id: usize) -> Option<&mut Entity<Box<T>>>
    {
        self.rows.get_mut(&id)
    }

    // Add a struct to the table as a new entity
    pub fn add(&mut self, item: Box<T>) -> usize
    {
        // Use the first free identifier for the new entity
        let id = self.first_free_id;
        self.first_free_id += 1;

        // Create the new entity        
        let entity = Entity::new(id, self.id, item, Arc::clone(&self.transaction_manager));
        
        // Add the new entity to the hash map
        self.rows.insert(id, entity);
        
        let mut locked_transaction_manager = self.transaction_manager.lock().unwrap();
        
        if locked_transaction_manager.is_transaction_running()
        {
            // Add an entry to the transaction log indicating that entity did not exist before thre transaction
            debug!("Add transaction entry for a new entity (Table: {}, Id: {})", self.name, id);
            locked_transaction_manager.add_entry(TransactionEntry::NotExisting(
                self.id,
                id,
            ));        
        }

        return id;
    }

    // Remove an entity from the table
    pub fn remove(&mut self, id: usize)
    {
        self.rows.remove(&id);
    }

    // Get an iterator for the entities stored in the table
    pub fn iter(&self) -> Values<usize, Entity<Box<T>>>
    {            
        self.rows.values()
    }
    
    // Get a mutable iterator for the entities stored in the table
    pub fn iter_mut(&mut self) -> ValuesMut<usize, Entity<Box<T>>>
    {            
        self.rows.values_mut()
    }  

}

impl<T> TableBase for Table<T> where T: Serialize + DeserializeOwned
{
    // Revert an entity to its original state, what already existed before the transaction
    fn rollback_to_existing(&mut self, id: usize, state: &Vec<u8>)
    {
        debug!("rollback_to_existing ({}-{})", self.name, id);
        // Remove the modified version of entity if it is still in the table
        self.rows.remove(&id);        
        // Deserialize the original version of struct stored the entity
        let item = bincode::deserialize::<Box<T>>(&state[..]).unwrap();
        // Create a new entity (containing original version of the stored struct)
        let new_entity = Entity::<Box<T>>::new(id, self.id, item, self.transaction_manager.clone());
        // Add the new entity to the hash map
        self.rows.insert(id, new_entity);
    }

    // Remove and entity what did not exist before thre transaction
    fn rollback_to_not_existing(&mut self, id: usize)
    {
        debug!("rollback_to_not_existing ({}-{})", self.name, id);
        // Remove entity from hash map
        self.rows.remove(&id);
    }
}