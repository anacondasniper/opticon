use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use sqlx::sqlite::SqlitePool;

const USERS: &[(&str, &str)] = &[
    ("MR_fpp", "pass1"),
    ("NG_fpp", "pass2"),
    ("KH_fpp", "pass3"),
    ("JP_fpp", "pass4"),
];

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let db = SqlitePool::connect(
        &std::env::var("DATABASE_URL").expect("DATABASE_URL not set"),
    )
    .await
    .expect("failed to connect");

    for (username, password) in USERS {
        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .expect("hashing failed!")
            .to_string();
        let result = sqlx::query(
            "INSERT OR IGNORE INTO users (username, password_hash) VALUES (?, ?)",
        )
        .bind(username)
        .bind(&hash)
        .execute(&db)
        .await
        .expect("insert failed");

        if result.rows_affected() > 0 {
            println!("created user: {username}");
        } else {
            println!("use already exists, skipped: {username}");
        }
    }

    let doorbell_token: String = {
        use argon2::password_hash::rand_core::{RngCore, OsRng};
        let mut bytes = [0u8; 32];
        OsRng.fill_bytes(&mut bytes);
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    };

    sqlx::query(
        "INSERT OR IGNORE INTO doorbells (id, name, token) VALUES (?, ?, ?)",
    )
    .bind("front-1")
    .bind("Front Door")
    .bind(&doorbell_token)
    .execute(&db)
    .await
    .expect("doorbell insert failed!");

    println!("doorbell front-1 token: {doorbell_token}");

    println!("seeding done.");
}
