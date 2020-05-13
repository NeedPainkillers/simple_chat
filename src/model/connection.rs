// TODO: possibly set sender here?
#[derive(Clone)]
pub struct Connection
{
    pub ip: String,
    pub port: String
}

impl Connection
{
    pub fn new() -> Connection
    {
        Connection
            {
                ip: String::new(),
                port: String::new()
            }
    }
}