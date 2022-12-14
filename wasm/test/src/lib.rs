//  fn main(input: &String, pattern: &String) -> Box<String> {
//     Box::new(input.lines().filter(|it| it.contains(pattern)).collect())
// }

use rusqlite::NO_PARAMS;
use rusqlite::{Connection, Result};

#[no_mangle]
extern "C" fn main() -> Result<()> {
    let conn = Connection::open("cats.db")?;

    conn.execute(
        "create table if not exists cat_colors (
             id integer primary key,
             name text not null unique
         )",
        NO_PARAMS,
    )?;
    conn.execute(
        "create table if not exists cats (
             id integer primary key,
             name text not null,
             color_id integer not null references cat_colors(id)
         )",
        NO_PARAMS,
    )?;

    Ok(())
}
