use sea_orm_migration::prelude::*;

#[async_std::main]
async fn main() {
    let fallback = "postgres://lorenzo:lorenzo@localhost:5432/concord";
    std::env::set_var("DATABASE_URL", fallback);
    cli::run_cli(migration::Migrator).await;
}
