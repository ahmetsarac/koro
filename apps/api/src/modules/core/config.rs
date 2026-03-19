//! Environment loading (repo root and crate-local `.env`).

pub fn load_dotenv() {
    dotenvy::from_filename("../../.env").ok(); // repo root
    dotenvy::dotenv().ok(); // apps/api/.env
}
