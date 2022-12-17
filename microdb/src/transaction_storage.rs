use serde::{Serialize, Deserialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

#[derive(Serialize, Deserialize)]
pub struct SerializedTransaction
{
    pub name: String,
    pub serialized_parameters: Vec<u8>
}

pub trait TransactionStorage
{
    fn add(&mut self, name: String, serialized_parameters: Vec<u8>);

    fn get(&mut self) -> Option<SerializedTransaction>;
}

// ***************************** NullTransactionStorage ***************************** //

pub struct NullTransactionStorage
{    
}

impl NullTransactionStorage
{
    pub fn new() -> Self
    {
        NullTransactionStorage { }
    }
}

impl TransactionStorage for NullTransactionStorage
{
    fn add(&mut self, _name: String, _serialized_parameters: Vec<u8>)
    {
    }

    fn get(&mut self) -> Option<SerializedTransaction>
    {
        None
    }
}

// ***************************** FileTransactionStorage ***************************** //

pub struct FileTransactionStorage
{
    pub file: File
}

impl FileTransactionStorage
{
    pub fn new(path: &str) -> Self
    {
        let file_name = format!("{}/transactions.bin", path);
        return Self { file: OpenOptions::new().read(true).write(true).create(true).open(file_name).unwrap() };
    }    
}

impl TransactionStorage for FileTransactionStorage
{
    fn add(&mut self, name: String, serialized_parameters: Vec<u8>)
    {
        let serialized_transaction = SerializedTransaction { name: String::from(name), serialized_parameters };
        let buf = bincode::serialize(&serialized_transaction).unwrap();
        let _ = self.file.write(&buf.len().to_le_bytes());
        self.file.write_all(&buf[..]).unwrap();        
    }

    fn get(&mut self) -> Option<SerializedTransaction>
    {
        let mut buf: [u8;8] = [0,0,0,0,0,0,0,0];
        let reader_length = self.file.read(&mut buf).expect("Unable to read transaction from storage");        
        if reader_length == 0
        {
            return None;
        } 
        let length = usize::from_le_bytes(buf);
        let mut vec_buf = vec![0u8; length];
        self.file.read(&mut vec_buf).expect("Unable to read transaction from storage");        
        return Some(bincode::deserialize::<SerializedTransaction>(&mut vec_buf[..]).unwrap());
    }
}