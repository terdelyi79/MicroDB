use serde::{Serialize, Deserialize};
use std::fs::{File, OpenOptions };
use std::io::{Read, Write, BufReader, BufWriter, Seek, SeekFrom };

#[derive(Serialize, Deserialize)]
pub struct SerializedTransaction
{
    pub name: String,
    pub serialized_parameters: Box<Vec<u8>>
}

pub trait TransactionStorage
{
    fn read(&mut self, buf: &mut [u8]) -> usize;

    fn write(&mut self, buf: &[u8]) -> usize;

    fn add(&mut self, name: String, serialized_parameters: Box<Vec<u8>>)
    {
        let name_bytes = name.as_bytes();
        self.write(&name_bytes.len().to_le_bytes());
        self.write(name_bytes);
        self.write(&serialized_parameters.len().to_le_bytes());
        self.write(&serialized_parameters.as_ref());
    }

    fn get(&mut self) -> Option<Box<SerializedTransaction>>
    {
        let mut name_length_buf: [u8;8] = [0;8];
        let count = self.read(&mut name_length_buf);
        if count == 0
        {
            return None;
        }
        let name_length = usize::from_le_bytes(name_length_buf);
        let mut name_buf = vec![0u8; name_length];
        self.read(&mut name_buf);
        let name = std::str::from_utf8(&mut name_buf).unwrap();

        let mut buf: [u8;8] = [0;8];
        self.read(&mut buf);
        let length = usize::from_le_bytes(buf);
        let mut serialized_parameters = vec![0u8; length];
        self.read(&mut serialized_parameters);
        Some(Box::new(SerializedTransaction { name: String::from(name), serialized_parameters: Box::new(serialized_parameters) }))
    }
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
    fn read(&mut self, _buf: &mut [u8]) -> usize
    {
        0
    }

    fn write(&mut self, _buf: &[u8]) -> usize
    {
        0
    }
}

// ***************************** FileTransactionStorage ***************************** //

pub struct FileTransactionStorage
{
    pub reader: BufReader<File>,
    pub writer: BufWriter<File>,
    pos: usize
}

impl FileTransactionStorage
{
    pub fn new(path: &str) -> Self
    {   
        let file2 = OpenOptions::new().write(true).create(true).open(format!("{}/transactions.bin", path)).unwrap();     
        let file1 = OpenOptions::new().read(true).open(format!("{}/transactions.bin", path)).unwrap();
        let reader = BufReader::with_capacity(1000000, file1);
        let mut writer = BufWriter::with_capacity(1000000, file2);
        writer.seek(SeekFrom::End(0)).unwrap();

        Self { reader, writer, pos: 0 }
    }
}

impl TransactionStorage for FileTransactionStorage
{
    fn read(&mut self, buf: &mut [u8]) -> usize
    {
        let capacity = self.reader.capacity();
        let len = buf.len();
        if self.pos + len <= capacity
        {
            self.pos = (self.pos + len) %capacity;
            return self.reader.read(buf).unwrap();
        }
        else
        {            
            let len1 = capacity - self.pos;   
            let readed_len1 = self.reader.read(&mut buf[0..len1]).unwrap();
            let readed_len2 = self.reader.read(&mut buf[len1..]).unwrap();
            self.pos = (self.pos + len) %capacity;
            return readed_len1 + readed_len2;
        }
    }

    fn write(&mut self, buf: &[u8]) -> usize
    {        
        let size = self.writer.write(buf).unwrap();        
        size
    }
}