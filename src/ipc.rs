use std::env;
use std::io::{BufRead, BufReader};
use std::os::unix::net::{UnixListener, UnixStream};
use std::thread;

pub fn create_socket() {
    let socket_dir = env::var("XDG_RUNTIME_DIR").unwrap_or("/tmp".to_string());
    let listener = UnixListener::bind(format!("{socket_dir}/gestures.sock")).unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| handle_connection(stream));
            }
            Err(err) => {
                eprintln!("Got error while handling IPC connection: {err}");
                break;
            }
        }
    }
    std::fs::remove_file(format!("{socket_dir}/gestures.sock")).unwrap();
}

fn handle_connection(stream: UnixStream) {
    let stream = BufReader::new(stream);

    for line in stream.lines() {
        if line.unwrap().contains("reload") {}
    }
}
