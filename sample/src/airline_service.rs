use std::sync::{Mutex, Arc};
use microdb::{QueryEngine, CommandEngine, entity::Entity};
use super::schema::*;
use super::commands::*;

/// A sample service for basic airline functionality
/// Service is thread safe, it can be used from multiple threads 
pub struct AirlineService
{
    query_engine: QueryEngine<AirlineDatabase>,
    command_engine_mutex: Mutex<CommandEngine<AirlineDatabase, AirlineCommandDefinitions>>
}

#[allow(dead_code)]
impl AirlineService
{
    /// Create a new instance of the service.    
    pub fn new(query_engine: QueryEngine<AirlineDatabase>, command_engine: CommandEngine<AirlineDatabase, AirlineCommandDefinitions>) -> Self
    {
        Self { query_engine, command_engine_mutex: Mutex::new(command_engine) }
    }

    /// Get airport identifier from code.
    /// Implementation is based on a query with O(n) complexity. Hash table based keys will be implemented later to support O(1) complexity.
    pub fn get_airport_id(&self, code: &str) -> usize
    {
        let db = self.query_engine.get_db();
        return db.airports.iter().filter(|f| f.code == code).next().unwrap().get_id();
    }
    
    /// Get flight identifier from flight number.
    /// Implementation is based on a query with O(n) complexity. Hash table based keys will be implemented later to support O(1) complexity.
    pub fn get_flight_id(&self, flight_number: &str) -> usize
    {
        let db = self.query_engine.get_db();
        return db.flights.iter().filter(|f| f.flight_numer == flight_number).next().unwrap().get_id();
    }

    /// Get all resrevations for a specific flight
    pub fn get_reservations(&self, flight_id: usize) -> Vec<Reservation>
    {
        let db = self.query_engine.get_db();
        return db.reservations.iter().filter(|r| r.flight_id == flight_id).map(|r| (*r).clone()).collect();
    }

    /// Add reservations in one transaction (both direction, connected flights, multiple passangers)
    pub fn add_reservations(&mut self, reservations: Vec<Reservation>) -> usize
    {        
        let mut command_engine = self.command_engine_mutex.lock().unwrap();
        let command_definitions = command_engine.get_command_definitions();
        return command_engine.push_command(Arc::new(command_definitions.add_reservations.create(reservations)));
    }

    /// Change schedule of a specific flight
    pub fn change_flight_schedule(&mut self, parameters: ChangeFlightScheduleParameters) -> usize
    {
        let mut command_engine = self.command_engine_mutex.lock().unwrap();
        let command_definitions = command_engine.get_command_definitions();
        return command_engine.push_command(Arc::new(command_definitions.change_flight_schedule.create(parameters)));
    }

    /// Wait while a transaction with specific identifier is processed
    pub fn wait_for_transaction(&mut self, transaction_id: usize)
    {
        let mut command_engine = self.command_engine_mutex.lock().unwrap();
        command_engine.wait_for_transaction(transaction_id);
    }

    /// Finds the fastest list of flights to travel from one airport to other on a specific day
    /// Returns a vector of flight ids or None if no route exist
    pub fn find_fastest_route(&self, day_of_week: u8, departure_airport_id: usize, arrival_airport_id: usize, min_change_time_in_minutes: u16) -> Option<Vec<usize>>
    {
        let db = self.query_engine.get_db();

        // Get all direct flights from departure airport as one-flight routes
        let mut routes: Vec<Vec<&Entity<Flight>>> = db.flights.iter()
          .filter(|f| f.day_of_week == day_of_week && f.departure_airport_id == departure_airport_id)
          .map(|f| vec! [f])
          .collect();

        loop
        {
            let mut routes_modified = false;
            let mut extended_routes: Vec<Vec<&Entity<Flight>>> = Vec::new();

            for route in routes
            {
                let last_flight = route.last().unwrap();

                // If current route already ends at the destination airport, then no extension is needed
                if last_flight.arrival_airport_id == arrival_airport_id
                {
                    extended_routes.push(route);
                }
                else
                {   
                    routes_modified = true;

                    // Get all flights can be used as a connection for the last flight of route
                    let connected_flights = db.flights.iter()
                      .filter(|f| f.day_of_week == day_of_week && f.departure_airport_id == last_flight.arrival_airport_id && f.departure_time_utc.get_total_minutes() - last_flight.arrival_time_utc.get_total_minutes() >= min_change_time_in_minutes);

                    for connected_flight in connected_flights
                    {
                        // Extend existing route with all possible connecting flights to make new routes
                        let mut extended_route = Vec::from_iter(route.iter().cloned());                    
                        extended_route.push(connected_flight);
                        extended_routes.push(extended_route);
                    }
                }
            }
            routes = extended_routes;

            // If routes were not modified in the last step, then we are ready 
            if routes_modified {
                break;
            }            
        }

        // If no any routes were found then return None
        if routes.len() == 0
        {
            return None;
        }

        // Sort routes by the total time of travel
        routes.sort_by(|a,b| 
            {                
                let flight_time_a = a.last().unwrap().arrival_time_utc.get_total_minutes() - a.first().unwrap().departure_time_utc.get_total_minutes();
                let flight_time_b = b.last().unwrap().arrival_time_utc.get_total_minutes() - b.first().unwrap().departure_time_utc.get_total_minutes();
                return flight_time_a.cmp(&flight_time_b);
            });

        // Return the first route from the sorted vector (the fastest one)
        return Some(routes.first().unwrap().iter().map(|f| f.get_id()).collect());

    }
}

#[cfg(test)]
mod tests {

    use crate::*;
    use microdb::{transaction_storage::NullTransactionStorage, TransactionStatus, CommandExecutionType };
    
    // Create a service to used by any unit tests below
    fn create_service() -> AirlineService
    {
        let command_definitions = AirlineCommandDefinitions::new();
        // Null transaction storage does not write commands to anywhere, as persistence  is not needed in unit tests
        let transaction_storage = NullTransactionStorage::new();
        let (query_engine, command_engine) = Engine::new(
        command_definitions,
         Box::new(transaction_storage),
         // Synchonous command processing executes commands on the same thread as which pushed it.
         // Unit tests can expect that commands are already executed when check results and no extra thread is created for each test
         CommandExecutionType::Synchronous,
         &|db|
         {
             let bud_id = db.airports.add(Airport { code: String::from("BUD"), name: String::from("Budapest Airport") });
             let vie_id = db.airports.add(Airport { code: String::from("VIE"), name: String::from("Vienna Airport") });
             let prg_id = db.airports.add(Airport { code: String::from("PRG"), name: String::from("Prague Airport") });
             db.flights.add(Flight {
                 flight_numer: String::from("TEST-001"), departure_airport_id: bud_id, arrival_airport_id: vie_id,
                 day_of_week: 1, departure_time_utc: HoursAndMinutes::new(8, 0), arrival_time_utc: HoursAndMinutes::new(8, 45),
                 seats: 3
             });
             db.flights.add(Flight {
                 flight_numer: String::from("TEST-002"), departure_airport_id: vie_id, arrival_airport_id: prg_id,
                 day_of_week: 1, departure_time_utc: HoursAndMinutes::new(10, 0), arrival_time_utc: HoursAndMinutes::new(11, 0),
                 seats: 3
             });
         });

    return AirlineService::new(query_engine, command_engine);
    }    

    // Function to check transaction statuses by unit tests
    fn get_transaction_status(airline_service: &AirlineService, transaction_id: usize) -> TransactionStatus
    {
        let command_engine = airline_service.command_engine_mutex.lock().unwrap();
        return command_engine.get_transaction_status(transaction_id);
    }

    // Test to demonstrate transaction handling
    // Flight has 3 seats only. First transaction should successfully reserve 2 seats. Second one should fail and revert its first reservation.
    #[test]
    fn reservation_test()
    {
        let mut airline_service = create_service();

        // Get flight id to use in reservations
        let flight_id = airline_service.get_flight_id("TEST-001");
        
        // Do a reservation on the flight for two passangers
        let transaction_id = airline_service.add_reservations( vec![
            Reservation { flight_id: flight_id, year: 2022, week: 30, name: String::from("Test Passanger 1") },
            Reservation { flight_id: flight_id, year: 2022, week: 30, name: String::from("Test Passanger 2") },
            ]);

        // Check if transaction was successful and reservation were made
        assert!(TransactionStatus::Completed == get_transaction_status(&airline_service, transaction_id));        
        assert_eq!(2, airline_service.get_reservations(flight_id).len());

        // Try to do a reservation again
        let transaction_id = airline_service.add_reservations( vec![
            Reservation { flight_id: flight_id, year: 2022, week: 30, name: String::from("Test Passanger 3") },
            Reservation { flight_id: flight_id, year: 2022, week: 30, name: String::from("Test Passanger 4") },
            ]);

        // Reservation should be failed and transaction rolled back
        assert!(TransactionStatus::Failed == get_transaction_status(&airline_service, transaction_id));  
        assert_eq!(2, airline_service.get_reservations(flight_id).len());
    }

    // Test to demonstrate transaction handling with custom typed fields
    // Changes are made on departure_time_utc and flight.arrival_time_utc fields. Type of these fields are HoursAndMinutes. All changes on these fields are reverted on a rollback.
    #[test]
    fn flight_schedule_change_test()
    {
        let mut airline_service = create_service();

        // Get flight id to use in reservations
        let flight_id = airline_service.get_flight_id("TEST-001");        

        // Try to modify schedule with invalid parameters (day_of_week = 8)
        let parameters = ChangeFlightScheduleParameters { flight_id, day_of_week: 8, departure_time_hours: 9, departure_time_minutes: 30, arrival_time_hours: 10, arrival_time_minutes: 15 };
        let transaction_id = airline_service.change_flight_schedule(parameters);        

        // Transaction should fail
        assert!(TransactionStatus::Failed == get_transaction_status(&airline_service, transaction_id));

        // Get the flight we tried to change
        let db = airline_service.query_engine.get_db();
        let flight = db.flights.get(flight_id).unwrap();

        // Fields with custom types should be reverted to original values as well
        assert_eq!(8, flight.departure_time_utc.hours);
        assert_eq!(0, flight.departure_time_utc.minutes);
        assert_eq!(8, flight.arrival_time_utc.hours);
        assert_eq!(45, flight.arrival_time_utc.minutes);
    }

    // Test to demonstrate how easy is to write complex queries
    // The "find_fastest_route" function finds the flights to travel from one ariport to other on a given day
    #[test]
    fn find_fastest_route()
    {
        let airline_service = create_service();
        
        // Get airport and flight ids needed to check results
        let bud_id = airline_service.get_airport_id("BUD");
        let prg_id = airline_service.get_airport_id("PRG");
        let flight1_id = airline_service.get_flight_id("TEST-001");
        let flight2_id = airline_service.get_flight_id("TEST-002");  

        // If minimum change time is 60 minutes, then the two test flights can be taken after each other
        let fastest_route = airline_service.find_fastest_route(1, bud_id, prg_id, 60);                
        assert_eq!(format!("{},{},", flight1_id, flight2_id), fastest_route.unwrap().iter().map(|id| id.to_string() + ",").collect::<String>());

        // If minimum change time is 120 minutes, then no any root exist
        let fastest_route = airline_service.find_fastest_route(1, bud_id, prg_id, 120);
        assert_eq!(None, fastest_route);
    }
}