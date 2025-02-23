use mongodb::{Client, Collection, Database};

pub struct Resource {
    mongo: Client,
    database_name: String,
}

impl Resource {
    pub fn new(mongo: Client, database_name: String) -> Resource {
        Resource {
            mongo,
            database_name,
        }
    }

    pub fn database(&self) -> Database {
        self.mongo.database(&self.database_name)
    }

    pub fn collection<T>(&self, name: &str) -> Collection<T> {
        self.database().collection::<T>(name)
    }
}
