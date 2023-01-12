use blog_commands::{BlogCommands};
use blog_service::BlogService;
use microdb::{transaction_storage::FileTransactionStorage, CommandExecutionType, Engine, command::CommandDirectoryFactory};

mod schema;
mod blog_commands;
mod blog_service;

fn main()
{
    let engine = Engine::new( BlogCommands::new(), Box::new(FileTransactionStorage::new(".")), CommandExecutionType::Asynchronous, &|_| {} );

    let mut blog_service = BlogService::new( engine );    
    
    blog_service.wait_for_transaction(blog_service.create_blogger(String::from("John Smith")));

    for (blogger_id, blogger) in blog_service.get_bloggers()
    {
        println!("{} ({})", blogger.name, blogger_id);
    }
}