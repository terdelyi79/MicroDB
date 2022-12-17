use crate::{Database};
use serde::{Serialize, de::DeserializeOwned};

// ***************************** Command Definition ***************************** //

pub trait CommandDefinitionBase<D> where D: Database
{
  fn create_from_serialized(&self, serialized_parameters: Vec<u8>) -> Box<dyn CommandBase<D> + '_>;  
}

#[derive(Clone)]
pub struct CommandDefinition<D, P> where D: Database, P: Serialize + DeserializeOwned
{
  name: &'static str,
  cmd: fn (&mut D, &P) -> Result<(), String>  
}

impl<D, P> CommandDefinition<D, P> where D: Database, P: Serialize + DeserializeOwned
{
  pub fn new(name: &'static str, cmd: fn (&mut D, &P) -> Result<(), String>) -> Self
  {
    Self {name, cmd}
  }

  pub fn create(&self, p: P) -> Command<D, P>
  {
    return Command { definition: CommandDefinition { name: self.name, cmd: self.cmd }, parameters: p };
  }

  fn run(&self, db: &mut D, parameters: &P) -> Result<(), String>
  {
    return (self.cmd)(db, parameters);
  }

  pub fn get_name(&self) -> &'static str
  {
    self.name
  }

  pub fn get_cmd(&self) -> fn (&mut D, &P) -> Result<(), String>  
  {
    self.cmd
  }
}

impl<D, P> CommandDefinitionBase<D> for CommandDefinition<D, P> where D: Database, P: Serialize + DeserializeOwned
{
  fn create_from_serialized(&self, serialized_parameters: Vec<u8>) -> Box<dyn CommandBase<D> + '_>
  {
    let parameters = bincode::deserialize::<P>(&serialized_parameters[..]).unwrap();
    return Box::new(Command::<D, P> { definition: CommandDefinition { name: self.name, cmd: self.cmd }, parameters });
  } 
}

// ********************************** Command *********************************** //

pub trait CommandBase<D> where D: Database
{
  fn run(&self, db: &mut D) -> Result<(), String>;

  fn get_name(&self) -> &'static str;  
  
  fn get_serialized_parameters(&self) -> Vec<u8>;
}

pub struct Command<D, P> where D: Database, P: Serialize + DeserializeOwned
{
  definition: CommandDefinition<D, P>,
  parameters: P
}

impl<D, P> CommandBase<D> for Command<D, P> where D: Database, P: Serialize + DeserializeOwned
{
  fn run(&self, db: &mut D) -> Result<(), String>
  {    
    return self.definition.run(db, &self.parameters);
  }

  fn get_name(&self) -> &'static str
  {
    &self.definition.name
  }

  fn get_serialized_parameters(&self) -> Vec<u8>
  {
    bincode::serialize(&self.parameters).unwrap()
  }
}

// ***************************** Command Definitions ***************************** //

pub trait CommandDefinitions<D>
{
    fn get(&self, name: &str) -> Box<dyn CommandDefinitionBase<D>>;
}