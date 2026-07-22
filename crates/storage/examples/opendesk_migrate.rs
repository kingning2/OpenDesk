//! Run pending `opendesk.db` Diesel migrations on a given path.
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-22

use std::env;
use std::path::PathBuf;

use storage::opendesk_db::OpendeskDb;

fn main() {
    let path = env::args()
        .nth(1)
        .map(PathBuf::from)
        .or_else(default_db_path)
        .expect("usage: opendesk_migrate [path-to-opendesk.db]");

    OpendeskDb::open(&path).expect("run migrations");
    println!("migrations applied: {}", path.display());
}

fn default_db_path() -> Option<PathBuf> {
    env::var("LOCALAPPDATA")
        .ok()
        .map(|root| PathBuf::from(root).join("OpenDesk").join("opendesk.db"))
}
