use microdb::command::{CommandDirectory, CommandDirectoryFactory, CommandDefinition};
use microdb_derive::{CommandDirectory, CommandDirectoryFactory};
use crate::schema::{BlogDatabase, Blogger};

#[derive(CommandDirectory, CommandDirectoryFactory)]
pub struct BlogCommands
{    
  pub create_blogger: CommandDefinition::<BlogDatabase, Blogger> 
}

impl BlogCommands
{
  fn create_blogger(db: &mut BlogDatabase, blogger: &Blogger) -> Result<(), String>
  {
    db.bloggers.add(blogger.clone());    
    Ok(())
  }
}

  

