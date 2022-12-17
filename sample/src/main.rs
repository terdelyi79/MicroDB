mod schema;
mod generated;
mod commands;
mod airline_service;

use schema::*;
use commands::*;
use microdb::{Engine, transaction_storage::FileTransactionStorage, CommandExecutionType};
use airline_service::AirlineService;

/// MicroDB is a revolutionary high productivity database engine
/// 
/// Benefits:
/// - Use structs defined in Rust source directly without any ORM
/// - Use Rust iterators to select from tables
/// - Whole database implementation is strongly typed causing compile time errors (Unlike runtime errors in SQL databases)
/// - Full ACID serialized transaction support
/// - All concurrency issues are avoided
/// - All database functionality can be easily unit tested
/// - Complex queries can be easily implemented like any functions in rust
/// - Custom field types can be easily used with full transaction support
/// - Crazy performance
/// - Zero deployment (database is automatically created on the fly)
/// 
/// Drawbacks:
/// 
///  Whole database must fit into the memory. However, 
///   - As all services have their own database in Microservices Architecture, size of databases are much smaller than it was in Monolits.
///   - A VM with 6TB (!) memory in the cloud currently costs the same per hour as a senior software developer. (Memories will be even bigger and cheaper in the future.)
///   - There are different ways to decrease database size (partitioning, etc)
/// 
/// How MicroDb works?
/// 
/// CQRS (Command Query Responsibility Segregation) Pattern:
///  Query: A read only operation on the database like SQL SELECTs on classic relational databases
///  Command: An operation to modify data in the database without any return values like INSERTs, UPDATEs, DELETEs or stored procedures in classic relational databases
/// 
/// Event Sourcing Pattern:
///  Events are the commands, while aggregates are the contents of database tables
/// 
/// Concurrency handling:
///  Content of tables are stored in the memory. Multiple queries can select data from them at the same time, but commands lock the whole database
///  Commands are processed in asynchronous way on one dedicated thread after each other (serialized transactions), therefore all concorrency issues are avoided.
///   (Traditional relational databases may have issues according to the used isolation level and dead locks may happen.)
///  As transactions do changes in memory only, they are fast, therfore bigger transactions do not cause significant delays for smaller ones
/// 
/// Transaction handling (ACID transaction support):
///  A transaction log is written to the memory. It is used to roll back transactions on soft errors.
///  All commands are stored on the disk as soon as arrived. On hard errors the database engine is restarted and all commands are executed again to do all changes in the memory. (What is fast)
/// 
/// Snapshots:
///  After lots of transactions the disk usage can be big and a database engine restart can be slow (executing all the transactions again)
///  Snapshot is a planned feature to persist sometimes the content of all tables. Only commands arrived after the last snapshot must be stored and executed this way
///
/// This is a demo application for an airline service based on MicroDB.
/// Some interesting features are demonstrated by unit tests for the service itself.
/// The main function of apllication contains a performance tests. It runs about 50.000 ACID transactions on a 5 years old notebook.
fn main()
{
    // Number of transactions to use in the perfromance test
    const N: usize = 100000;

    // Create an airline database with minimal data for performance testing
    let command_definitions = AirlineCommandDefinitions::new();
    // Commands are stored on the disk
    let transaction_storage = FileTransactionStorage::new(".");
    let (query_engine, command_engine) = Engine::new(
        command_definitions,
         Box::new(transaction_storage),
         // Commands will be processed in asynchronous way
         CommandExecutionType::Asynchronous,
         &|db|
         {
             let bud_id = db.airports.add(Airport { code: String::from("BUD"), name: String::from("Budapest Airport") });
             let vie_id = db.airports.add(Airport { code: String::from("VIE"), name: String::from("Vienna Airport") });             
             db.flights.add(Flight {
                 flight_numer: String::from("TEST-001"), departure_airport_id: bud_id, arrival_airport_id: vie_id,
                 day_of_week: 1, departure_time_utc: HoursAndMinutes::new(8, 0), arrival_time_utc: HoursAndMinutes::new(8, 45),
                 seats: N
             });             
         });
    let mut airline_service = AirlineService::new(query_engine, command_engine);
    
    // Get the flight is to use in the test
    let flight_id = airline_service.get_flight_id("TEST-001");    

    let start = std::time::Instant::now();

    // Run a transaction for reservation N times
    let mut i = 0;
    let mut transaction_id = 0;
    while i < N
    {
        transaction_id = airline_service.add_reservations( vec![
        Reservation { flight_id: flight_id, year: 2022, week: 30, name: String::from("Test Passanger 1") }
        ]);
        i += 1;        
    }    

    // Wait for the last transaction to finish
    airline_service.wait_for_transaction(transaction_id);    

    
    println!("{} reservation were added in {:?}", N, start.elapsed());
}