use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

fn handle_client(mut stream: TcpStream, total_requests: Arc<Mutex<u32>>, valid_requests: Arc<Mutex<u32>>){

    let peer_ip = stream.peer_addr().unwrap();
    println!("Received connection from: {}", peer_ip);

    let mut msg = String::new();

    let mut buffer = [0; 500];

    loop {
        // .read() with a 500 bite buffer
        let n = stream.read(&mut buffer[..]).unwrap();

        // Use std::str::from_utf8 to convert the buffer into a &str.
        let buffer = std::str::from_utf8(&buffer).unwrap();

        // Use the push_str() method to append it to the accumulated message.
        msg.push_str(buffer);

        // As the client is awaiting a reply, it will not close the connection. Even after it finishes its transmission, 
        // the read() will still block, waiting for more data. Since the http protocol specifies that a client’s message ends 
        // with the character sequence \r\n\r\n, once the accumulated message ends with that sequence, the loop can end. 
        // As some clients end with \n\n, the http specification allows servers to end with that sequence too.
        if msg.contains("\r\n\r\n") || msg.contains("\n\n") {
            break;
        }
    }
    println!("Received message: {}", msg);

    let request_line = msg.lines().next().unwrap();

    let path = request_line.split_whitespace().nth(1).unwrap();
    // increment total request counter
    let mut total_requests = total_requests.lock().unwrap();
    *total_requests += 1;

    let mut response = String::new();
    let path_buf = PathBuf::from(path);

    // Validating
    let mut candidate_path = std::env::current_dir().unwrap();
    for i in path_buf.components() {
        candidate_path.push(i);
    }
    // call current dir again and make sure it is a parent
    let current_dir = std::env::current_dir().unwrap();

    // println!("cp: {}", candidate_path.display());
    // if subordanate
    if candidate_path.starts_with(std::env::current_dir().unwrap()) {
        // if file does not exist
        if !candidate_path.exists() {
            response = "HTTP/1.1 404 Not Found\r\n\r\n".to_string();
        } else {
            let contents = std::fs::read_to_string(&candidate_path).unwrap();
            response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\nContent-Length: {}\r\n\r\n<html>\r\n<body>\r\n<h1>Message received</h1>\r\nRequested file: {}<br>\r\n</body>\r\n</html>\r\n",
                contents.len(), contents
                // SEND BACK THE CONTENTS OF THE FILE?
            );
            let mut valid_requests = valid_requests.lock().unwrap();
            *valid_requests += 1;
        }
    } else {
        response = "HTTP/1.1 403 Forbidden\r\n\r\n".to_string();
    }

    println!("Response: {}", response);

    // Writing to browser
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();

    // print both counts
    println!("Valid requests: {}", *valid_requests.lock().unwrap());
    println!("Total requests: {}", *total_requests);

}

fn main() -> std::io::Result<()> {

    let total_requests = Arc::new(Mutex::new(0));
    let valid_requests = Arc::new(Mutex::new(0));
    
    let listener = TcpListener::bind("127.0.0.1:8888")?;

    // accept connections and process them serially
    for stream in listener.incoming() {
        
        let stream = stream?;

        thread::spawn({
            let total_requests = total_requests.clone();
            let valid_requests = valid_requests.clone();
        
            move || {
                handle_client(stream, total_requests, valid_requests);
            }
        });
        
    }

    Ok(())
}
