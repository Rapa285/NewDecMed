use std::env;

pub struct Sql {
    db_url: String,
}

impl Sql {

    fn new () -> Self {
        let db_url = env::var("DB_URL").expect("DB_URL environment variable not set");
        Sql { db_url }
    }
    
    pub async fn add (&self) {
        
    }
}