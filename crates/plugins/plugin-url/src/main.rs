use std::io::{self, BufRead, Write};

use plugin_url::{Query, search};

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    for line in stdin.lock().lines() {
        let line = line?;
        let q: Query = match serde_json::from_str(&line) {
            Ok(q) => q,
            Err(_) => continue,
        };
        let results = search(&q.query);
        writeln!(out, "{}", serde_json::to_string(&results).unwrap())?;
        out.flush()?;
    }
    Ok(())
}
