use blog_commands::BlogCommands;
use blog_service::BlogService;
use microdb::prelude::*;

mod schema;
mod blog_commands;
mod blog_service;



fn main()
{
    const N: usize = 1000000;    

    let engine = Engine::new( BlogCommands::new(), Box::new(FileTransactionStorage::new(".")), CommandExecutionType::Asynchronous, &|_| {} );    

    let mut blog_service = BlogService::new( engine );

    let start = std::time::Instant::now();    

    //Run a transaction for reservation N times
    let mut i = 0;
    let mut transaction_id = 0;
    while i < N
    {
        transaction_id = blog_service.create_blogger(String::from("John Smith"));
        i += 1;        
    }    

    // Wait for the last transaction to finish
    blog_service.wait_for_transaction(transaction_id);
    
    println!("{} items were added in {:?}", N, start.elapsed());

    println!("Number of bloggers in the database: {}", blog_service.get_bloggers().len());
}