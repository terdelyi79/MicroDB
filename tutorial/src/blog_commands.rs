use microdb::prelude::*;
use microdb_derive::*;
use crate::schema::{BlogDatabase, Blogger};

#[derive(CommandDirectory, CommandDirectoryFactory)]
pub struct BlogCommands
{    
  pub create_blogger: CommandDefinition::<BlogDatabase, Box<Blogger>> 
}

impl BlogCommands
{
  fn create_blogger(db: &mut BlogDatabase, blogger: &Box<Blogger>) -> Result<(), String>
  {
    db.bloggers.add((*blogger).clone());    
    Ok(())
  }
}

  

