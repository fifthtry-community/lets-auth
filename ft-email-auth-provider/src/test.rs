#[test]
fn basic() {
    let db = diesel::sqlite::SqliteConnection::establish(":memory:").unwrap();
}