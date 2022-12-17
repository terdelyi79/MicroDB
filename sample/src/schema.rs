use serde::{Serialize, Deserialize};
use core::fmt::{Display, Formatter};
use microdb::table::Table;

/// Custom data type to store flight departure and arrival time
/// Contains the hours [0-24] and minutes inside the hours [0-59] only
/// It demonstrates how easy to use custom types in fields with full transaction support
#[derive(Serialize, Deserialize)]
pub struct HoursAndMinutes
{
    pub hours: u8,
    pub minutes: u8
}

impl HoursAndMinutes
{
    pub fn new(hours: u8, minutes: u8) -> Self
    {
        return Self { hours, minutes };
    }
}

impl HoursAndMinutes
{
    pub fn get_total_minutes(&self) -> u16
    {
        self.hours as u16 * 60 + self.minutes as u16
    }
}

impl Display for HoursAndMinutes
{    
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result
    {        
        write!(f, "{:02}:{:02}", self.hours, self.minutes)
    }
}


/// Represents and airport containing an short code and user frienfly name
#[derive(Serialize, Deserialize)]
pub struct Airport
{
    pub code: String,
    pub name: String
}

impl Display for Airport
{    
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result
    {        
        write!(f, "{} ({})", self.name, self.code)
    }
}

/// Flight entity contains all needed information about a flight from one airport to another one
#[derive(Serialize, Deserialize)]
pub struct Flight
{
    pub flight_numer: String,
    pub departure_airport_id: usize,
    pub arrival_airport_id: usize,
    pub day_of_week: u8,
    pub departure_time_utc: HoursAndMinutes,
    pub arrival_time_utc: HoursAndMinutes,
    pub seats: usize
}

/// A reservation for a flight in a specific year and week
#[derive(Serialize, Deserialize, Clone)]
pub struct Reservation
{
    pub flight_id: usize,
    pub year: u16,
    pub week: u8,
    pub name: String
}

/// Stores the number of all reservations for a flight in a year and week
#[derive(Serialize, Deserialize)]
pub struct FlightReservationCount
{
    pub flight_id: usize,
    pub year: u16,
    pub week: u8,
    pub count: usize
}

/// Specifies a simplified database for an airline
/// It assumes that schedules of flights are the same every week and there are no overnight flights
pub struct AirlineDatabase
{
    pub airports: Table<Airport>,
    pub flights: Table<Flight>,
    pub reservations: Table<Reservation>,
    pub flight_reservation_counts: Table<FlightReservationCount>
}