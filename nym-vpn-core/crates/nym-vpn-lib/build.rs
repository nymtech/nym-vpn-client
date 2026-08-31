// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::env;

use sqlx::{Connection, SqliteConnection};
use vergen_gitcl::{BuildBuilder, CargoBuilder, Emitter, GitclBuilder, RustcBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let database_path = format!("{out_dir}/adblocker.db");

    let mut conn = SqliteConnection::connect(&format!("sqlite://{database_path}?mode=rwc"))
        .await
        .expect("Failed to create SQLx database connection");

    sqlx::migrate!("./src/adblocker/db/migrations")
        .run(&mut conn)
        .await
        .expect("Failed to perform SQLx migrations");

    println!("cargo:rustc-env=DATABASE_URL=sqlite://{}", database_path);

    Emitter::default()
        .add_instructions(&BuildBuilder::all_build()?)?
        .add_instructions(&CargoBuilder::all_cargo()?)?
        .add_instructions(&GitclBuilder::all_git()?)?
        .add_instructions(&RustcBuilder::all_rustc()?)?
        .emit()?;

    Ok(())
}
