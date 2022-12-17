use microdb::command::CommandDefinition;
use super::schema::*;
use serde::{Serialize, Deserialize};

pub struct AirlineCommandDefinitions
{    
  pub add_reservations: CommandDefinition<AirlineDatabase, Vec<Reservation>>,
  pub change_flight_schedule: CommandDefinition<AirlineDatabase, ChangeFlightScheduleParameters>
}

impl AirlineCommandDefinitions
{
  pub fn new() -> Self
  {
    Self {
        add_reservations: CommandDefinition::new("add_reservation", AirlineCommandDefinitions::add_reservations),
        change_flight_schedule : CommandDefinition::new("change_flight_schedule", AirlineCommandDefinitions::change_flight_schedule)
    }
  }
  
  // Add reservations in one transaction (Multiple passangers, connecting and return flights must be reserved in one atomic step)
  fn add_reservations(db: &mut AirlineDatabase, reservations: &Vec<Reservation>) -> Result<(), String>
  {
    for reservation in reservations
    {
        // Get the number of all seats on the flight
        let seats = db.flights.get(reservation.flight_id).ok_or("Invalid flight id")?.seats;        

        match db.flight_reservation_counts.iter_mut().filter(|f| f.flight_id == reservation.flight_id && f.year == reservation.year && f.week == reservation.week).next()
        {
          None => {
            // There aren't any reservations on this flight, therefore free seat is avaiable for sure, we need to add it
            db.flight_reservation_counts.add(FlightReservationCount { flight_id: reservation.flight_id, year:reservation.year, week: reservation.week, count: 1 });
          },          
          Some(flight_reservation_count) => {           
            
            // There are reservations, therefore must be checked whether any free seat is available
            if seats <= flight_reservation_count.count
            {
              // If no seat is available, we return an error to roll back transaction and revert all reservations made earlier in this loop
              return Err(String::from("No free seat is avaiable for reservation"));
            }

            // Update the number of reservation for this flight
            flight_reservation_count.count += 1;            
          }
        };
        
        // Create reservation
        db.reservations.add(reservation.clone());
    }

    // Return Ok to commit the transaction
    return Ok(());
  }

  // Change schedule of an existing flight
  pub fn change_flight_schedule(db: &mut AirlineDatabase, change_flight_schedule_parameters: &ChangeFlightScheduleParameters) -> Result<(), String>
  {
    // Get flight by flight id
    let flight = db.flights.get_mut(change_flight_schedule_parameters.flight_id).ok_or("Invalid flight id")?;

    // Change internal fields of "HoursAndMinutes" typed fields
    flight.departure_time_utc.hours = change_flight_schedule_parameters.departure_time_hours;
    flight.departure_time_utc.minutes = change_flight_schedule_parameters.departure_time_minutes;
    flight.arrival_time_utc.hours = change_flight_schedule_parameters.arrival_time_hours;
    flight.arrival_time_utc.minutes = change_flight_schedule_parameters.arrival_time_minutes;

    // Return an error when day_of_week parameter is invalid to roll back transaction and revert changes in previous lines
    if (change_flight_schedule_parameters.day_of_week < 1) || (change_flight_schedule_parameters.day_of_week > 7)
    {
      return Err(String::from("Invalid day of week"));
    }

    // Change day_of_week field
    flight.day_of_week = change_flight_schedule_parameters.day_of_week;

    // Return Ok to commit transaction
    return Ok(());
  }
}

#[derive(Serialize, Deserialize)]
pub struct ChangeFlightScheduleParameters
{
  pub flight_id: usize,
  pub day_of_week: u8,
  pub departure_time_hours: u8,
  pub departure_time_minutes: u8,
  pub arrival_time_hours: u8,
  pub arrival_time_minutes: u8
}