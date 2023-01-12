pub mod entity;
pub mod table;
pub mod command;
pub mod transaction;
pub mod transaction_storage;

use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard};
use std::thread;
use tokio::sync::{mpsc, Notify};
use command::{ CommandBase, CommandDirectory };
use transaction::TransactionManager;
use transaction_storage::TransactionStorage;
use table::TableBase;
use futures::executor::block_on;

pub trait DatabaseFactory
{
    fn create_database(transaction_manager_ref: Arc<Mutex<TransactionManager>>) -> Self;    
}

pub trait Database
{
    fn get_table_mut(&mut self, table_id: u64) -> &mut dyn TableBase;
}

pub struct QueryEngine<D> where D: Database
{
    db_lock_arc: Arc<RwLock<D>>
}

impl<D> QueryEngine<D> where D: Database
{
    pub fn get_db(&self) -> RwLockReadGuard<'_, D>
    {
        return self.db_lock_arc.read().unwrap();
    }
}

#[derive(PartialEq)]
pub enum CommandExecutionType { Synchronous, Asynchronous }

#[derive(PartialEq)]
pub enum TransactionStatus { Completed, Failed, NotExecuted }

pub struct CommandEngine<D, C> where D: Database + Sync + Send, C: CommandDirectory<D>
{
    db_lock_arc: Arc<RwLock<D>>,
    command_definitions: Arc<C>,
    transaction_storage: Box<dyn TransactionStorage>,
    last_pushed_transaction_id: usize,
    last_processed_transaction_id_lock: Arc<RwLock<usize>>,
    transaction_manager_ref: Arc<Mutex<TransactionManager>>,
    failed_transaction_ids_lock: Arc<RwLock<Vec<usize>>>,
    command_execution_type: CommandExecutionType,
    command_sender: Option<mpsc::Sender<Arc<dyn CommandBase<D> + Sync + Send>>>,
    processed_transaction_id_notify: Option<Arc<Notify>>
}

impl<D, C> CommandEngine<D, C> where D: Database + Sync + Send + 'static, C: CommandDirectory<D>
{
    pub fn new(
        db_lock_arc: Arc<RwLock<D>>,
        command_definitions: C,
        mut transaction_storage: Box<dyn TransactionStorage>,
        transaction_manager_ref: Arc<Mutex<TransactionManager>>,
        command_execution_type: CommandExecutionType
        ) -> Self
    {
        let mut last_processed_transaction_id: usize = 0;
        loop {
            let serialized_transaction = transaction_storage.get();            
            if serialized_transaction.is_some()
             {                
                let serialized_transaction = serialized_transaction.unwrap();
                let command_definition = command_definitions.get(&serialized_transaction.name);
                let command = command_definition.create_from_serialized(serialized_transaction.serialized_parameters);
                let db_lock = db_lock_arc.clone();
                let mut db = db_lock.write().unwrap();                
                last_processed_transaction_id += 1;
                // TODO: Store falied transaction ids on the disk to skip them when database is loaded
                command.run(&mut *(db)).expect("Transaction failed, what was succesful earlier");
             }
             else {
                 break;                
             }    
        }         

        let mut command_engine = Self {
             db_lock_arc: db_lock_arc.clone(),
             command_definitions: Arc::new(command_definitions),
             transaction_storage,
             last_pushed_transaction_id: last_processed_transaction_id,
             last_processed_transaction_id_lock: Arc::new(RwLock::new(last_processed_transaction_id)),
             transaction_manager_ref: transaction_manager_ref.clone(),
             failed_transaction_ids_lock: Arc::new(RwLock::new(Vec::new())),
             command_execution_type,
             command_sender: None,
             processed_transaction_id_notify : None
             };

        if command_engine.command_execution_type == CommandExecutionType::Asynchronous
        {
            let (command_sender, mut command_receiver): (mpsc::Sender<Arc<dyn CommandBase<D> + Sync + Send>>, mpsc::Receiver<Arc<dyn CommandBase<D> + Sync + Send>>) = mpsc::channel(100);
            command_engine.command_sender = Some(command_sender);

            let transactioprocessed_transaction_id_notify = Arc::new(Notify::new());
            command_engine.processed_transaction_id_notify = Some(transactioprocessed_transaction_id_notify.clone());

            let db_lock_arc = command_engine.db_lock_arc.clone();
            let transaction_manager_ref =  command_engine.transaction_manager_ref.clone();
            let last_processed_transaction_id_arc = command_engine.last_processed_transaction_id_lock.clone();
            let failed_transaction_ids_lock = command_engine.failed_transaction_ids_lock.clone();
            thread::spawn(move ||
                {
                    loop
                    {
                        let command = block_on(command_receiver.recv());

                        // If the channel is closed by the other thread
                        if command.is_none()
                        {
                            break;
                        }

                        let command = command.unwrap();

                        transaction_manager_ref.lock().unwrap().begin_transaction();
                        let mut last_processed_transaction_id = last_processed_transaction_id_arc.write().unwrap();
                        *last_processed_transaction_id += 1;
                        let mut db = db_lock_arc.write().unwrap();
                        let transaction_result = command.run(&mut *(db));
                        match transaction_result
                        {
                            Ok(_) => {
                            transaction_manager_ref.lock().unwrap().commit_transaction();
                        }
                        Err(_) => {                                
                            transaction_manager_ref.lock().unwrap().rollback_transaction(&mut db);
                            let mut failed_transaction_ids = failed_transaction_ids_lock.write().unwrap();
                            failed_transaction_ids.push(*last_processed_transaction_id);
                            }
                        }
                    
                        transactioprocessed_transaction_id_notify.notify_waiters();
                    }
                }
            );
        }

        command_engine
    }

    pub fn push_command(&mut self, cmd: Arc<dyn CommandBase<D> + Sync + Send>) -> usize
    {
        let serialized_parameters = cmd.get_serialized_parameters();
        let name = String::from(cmd.get_name());
        self.transaction_storage.add(name, serialized_parameters);
        self.last_pushed_transaction_id +=1;

        if self.command_execution_type == CommandExecutionType::Synchronous
        {
            let db_lock = self.db_lock_arc.clone();
            let mut db = db_lock.write().unwrap();

            self.transaction_manager_ref.lock().unwrap().begin_transaction();
            let mut last_processed_transaction_id = self.last_processed_transaction_id_lock.write().unwrap();
            *last_processed_transaction_id += 1;
            let transaction_result = cmd.run(&mut *(db));
            match transaction_result
            {
                Ok(_) => {
                     self.transaction_manager_ref.lock().unwrap().commit_transaction();
                }
                Err(_) => {                                
                     self.transaction_manager_ref.lock().unwrap().rollback_transaction(&mut db);
                    let mut failed_transaction_ids = self.failed_transaction_ids_lock.write().unwrap();
                    failed_transaction_ids.push(*last_processed_transaction_id);
                }
            }            
        }
        else
        {            
            let _ = block_on(self.command_sender.as_ref().unwrap().send(cmd));
        }

        self.last_pushed_transaction_id
    }

    pub fn get_command_definitions(&self) -> Arc<C>
    {
        return self.command_definitions.clone();
    }

    pub fn get_transaction_status(&self, transaction_id: usize) -> TransactionStatus
    {
        let last_processed_transaction_id = *self.last_processed_transaction_id_lock.read().unwrap();
        let failed_transaction_ids = self.failed_transaction_ids_lock.read().unwrap();

        if transaction_id > last_processed_transaction_id
            { return TransactionStatus::NotExecuted; }
        else if failed_transaction_ids.contains(&transaction_id)
            { return TransactionStatus::Failed; }
        else {
            { return TransactionStatus::Completed; }
        }
    }

    pub fn wait_for_transaction(&mut self, transaction_id: usize)
    {
        let mut last_processed_transaction_id = *self.last_processed_transaction_id_lock.read().unwrap();        

        loop {            

            if transaction_id <= last_processed_transaction_id            
            {
                break;
            }
            
            block_on(self.processed_transaction_id_notify.as_ref().unwrap().notified());
            
            last_processed_transaction_id = *self.last_processed_transaction_id_lock.read().unwrap();
            
        }
    }
}

pub struct Engine
{
}

impl Engine
{
    pub fn new<D, C>(command_definitions: C, transaction_storage: Box<dyn TransactionStorage>, command_execution_type: CommandExecutionType, init: &'static dyn Fn(&mut D)) -> (QueryEngine<D>, CommandEngine<D, C>) where D: Database + DatabaseFactory + Send + Sync, C: CommandDirectory<D>
    {
        let transaction_manager_ref = Arc::new(Mutex::new(TransactionManager::new()));
        let mut db = D::create_database(transaction_manager_ref.clone());        
        init(&mut db);
        let db_lock_arc = Arc::new(RwLock::new(db));
        let query_engine = QueryEngine { db_lock_arc: db_lock_arc.clone() };
        let command_engine = CommandEngine::new( db_lock_arc.clone(), command_definitions, transaction_storage, transaction_manager_ref.clone(), command_execution_type );
        return (query_engine, command_engine);
    }
}