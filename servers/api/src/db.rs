use sea_orm::DatabaseConnection;

pub struct Database {
    pub connection: DatabaseConnection,
}

impl Database {
    pub async fn new() -> Self {
        let fallback = "postgres://lorenzo:lorenzo@localhost:5432/concord";
        let connection = sea_orm::Database::connect(fallback)
            .await
            .expect("Could not connect to database");

        Database { connection }
    }

    pub fn get_connection(&self) -> &DatabaseConnection {
        &self.connection
    }
}
