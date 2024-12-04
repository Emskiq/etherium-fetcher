use diesel_migrations::{EmbeddedMigrations, embed_migrations, MigrationHarness};
use diesel::PgConnection;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub fn run_migrations(conn: &mut PgConnection) {
    conn.run_pending_migrations(MIGRATIONS).unwrap();
}
