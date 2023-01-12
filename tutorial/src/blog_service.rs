use std::sync::{Mutex, Arc};
use microdb::{QueryEngine, CommandEngine};
use crate::{schema::{BlogDatabase, Blogger, BloggerStatistics }, blog_commands::{BlogCommands}};

pub struct BlogService
{
    query_engine: QueryEngine<BlogDatabase>,
    command_engine_mutex: Mutex<CommandEngine<BlogDatabase, BlogCommands>>
}

#[allow(dead_code)]
impl BlogService
{
    pub fn new(engine: (QueryEngine<BlogDatabase>, CommandEngine<BlogDatabase, BlogCommands>)) -> Self
    {
        Self { query_engine: engine.0, command_engine_mutex: Mutex::new(engine.1) }
    }

    pub fn create_blogger(&self, name: String) -> usize
    {        
        let mut command_engine = self.command_engine_mutex.lock().unwrap();
        let command_definitions = command_engine.get_command_definitions();
        let blogger = Blogger { name, statistics: BloggerStatistics { post_count: 0, like_count: 0 } };
        return command_engine.push_command(Arc::new(command_definitions.create_blogger.create(blogger)));
    }

    pub fn get_bloggers(&self) -> Vec<(usize, Blogger)>
    {
        self.query_engine.get_db().bloggers.iter().map(|blogger| (blogger.get_id(), (*blogger).clone())).collect()
    }

    pub fn wait_for_transaction(&mut self, transaction_id: usize)
    {
        let mut command_engine = self.command_engine_mutex.lock().unwrap();
        command_engine.wait_for_transaction(transaction_id);
    }
}