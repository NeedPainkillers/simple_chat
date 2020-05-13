use chrono::{DateTime, TimeZone, NaiveDateTime, Local};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Message
{
    pub id: String,    //ip address of connection
    pub body: String,  //text of message
    pub name: String,
    pub datetime: DateTime<Local>
}

impl Message
{
    pub fn new() -> Message
    {
        Message
            {
                id: String::new(),
                body: String::new(),
                name: String::new(),
                datetime: Local::now()
            }
    }

    pub fn update_time(&mut self)
    {
        self.datetime = Local::now();
    }
}