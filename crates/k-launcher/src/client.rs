use std::io::Write;

pub fn send_show() -> Result<(), Box<dyn std::error::Error>> {
    let runtime_dir =
        std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".to_string());
    let socket_path = format!("{runtime_dir}/k-launcher.sock");
    let mut stream = std::os::unix::net::UnixStream::connect(&socket_path)?;
    stream.write_all(b"show\n")?;
    Ok(())
}
