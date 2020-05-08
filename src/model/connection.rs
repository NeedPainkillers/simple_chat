// TODO: possibly set sender here?
#[derive(Clone)]
pub struct Connection
{
    pub ip: String
}

impl Connection
{
    pub fn new() -> Connection
    {
        Connection
            {
                ip: String::new()
            }
    }
}