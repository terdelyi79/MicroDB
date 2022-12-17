use microdb::{ DbDefault, Database, table::Table, table::TableBase, transaction::TransactionManager, command::CommandDefinitions, command::CommandDefinitionBase, command::CommandDefinition };
use std::sync::{Arc, Mutex};
use super::schema::*;
use super::commands::*;

impl DbDefault for AirlineDatabase
{
    fn default(transaction_manager_ref: Arc<Mutex<TransactionManager>>) -> Self
    {        
        return Self
        {
            airports: Table::<Airport>::new("airports", transaction_manager_ref.clone()),
            flights: Table::<Flight>::new("flights", transaction_manager_ref.clone()),
            reservations: Table::<Reservation>::new("reservations", transaction_manager_ref.clone()),
            flight_reservation_counts: Table::<FlightReservationCount>::new("flight_reservation_counts", transaction_manager_ref.clone()),
        };
    }
}

impl Database for AirlineDatabase
{
    fn get_table_mut(&mut self, table_id: u64) -> &mut dyn TableBase
    {
        if table_id == self.airports.get_id() { return &mut self.airports };
        if table_id == self.flights.get_id() { return &mut self.flights };
        if table_id == self.reservations.get_id() { return &mut self.reservations };
        if table_id == self.flight_reservation_counts.get_id() { return &mut self.flight_reservation_counts };
        panic!("Unknown table");
    }
}

impl CommandDefinitions<AirlineDatabase> for AirlineCommandDefinitions
{
    fn get(&self, name: &str) -> Box<dyn CommandDefinitionBase<AirlineDatabase>>
    {
        match name
        {
            "add_reservation" => Box::new(CommandDefinition::<AirlineDatabase, Vec<Reservation>>::new(self.add_reservations.get_name(), self.add_reservations.get_cmd())),
            _ => panic!("Unknown command")
        }
    }
}