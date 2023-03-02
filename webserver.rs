// NOTES from class

// Test using web browser or Webget (but introduce correct port nums to Webget)

// Important thing: localhost:8888

// Look at TcpListener https://doc.rust-lang.org/std/net/struct.TcpListener.html
// First example is the exact structure we should use
// don't forget for stream in listener.incoming()...

// Things become difficult quickly if we have to deal with multiple clients
// We could do a fork in the for loop in main to handle multiple requests
// But we could do a cheaper approach (less space and cpu) called threading
// Threading the server is the assignment
// Reference thread_demo.rs for the threading part of this assignment

// Introduce lock system like thread demo
// When lock goes out of scope, it is released

// to start get command line arguments in a vec

// we will be talking a lot ab atomic 

// it is hard to get the message from the client
// use read directly on the socket
// only want to return a slice of the buffer so 
// look for a sequence of two new lines (\r\n\r\n or \n\n)

// Don't have loops after you claim a lock



use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::{Read, Write};
use std::path::PathBuf;

fn handle_client(mut stream: TcpStream) {
    let mut total_requests = 0;
    let mut valid_requests = 0;

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
    total_requests += 1;

    let mut response = String::new();
    let path_buf = PathBuf::from(path);

    // Validating
    let mut candidate_path = std::env::current_dir().unwrap();
    for i in path_buf.components() {
        candidate_path.push(i);
    }
    // call current dir again and make sure it is a parent
    let current_dir = std::env::current_dir().unwrap();

    // if subordanate
    if candidate_path.starts_with(std::env::current_dir().unwrap()) {
        // if file does not exist
        if !candidate_path.exists() {
            response = "HTTP/1.1 404 Not Found\r\n\r\n".to_string();
        } else {
            let contents = std::fs::read_to_string(&candidate_path).unwrap();
            response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\nContent-Length: {}\r\n\r\n<html>\r\n<body>\r\n<h1>Message received</h1>\r\nRequested file: {}<br>\r\n</body>\r\n</html>\r\n",
                msg.len(), contents
                // SEND BACK THE CONTENTS OF THE FILE?
            );
            valid_requests += 1;
        }
    } else {
        response = "HTTP/1.1 403 Forbidden\r\n\r\n".to_string();
        // response = format!(
        //     "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\nContent-Length: {}\r\n\r\n<html>\r\n<body>\r\n<h1>Message received</h1>\r\nRequested file: {}<br>\r\n</body>\r\n</html>\r\n",
        //     msg.len(), path
        //     // SEND BACK THE CONTENTS OF THE FILE?
        // );
    }

    println!("Response: {}", response);

    // Writing to browser
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();

    // print both counts
    println!("Valid requests: {}", valid_requests);
    println!("Total requests: {}", total_requests);

}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8888")?;

    // accept connections and process them serially
    for stream in listener.incoming() {
        
        let stream = stream?;

        thread::spawn(move || {
            handle_client(stream);
        });

        // if this is a valid request and a request for a file, send back the contents in a .txt file

    }

    Ok(())
}



// OG CODE!!! -- Ferrer says it works on his end

// use std::net::{TcpListener, TcpStream};

// fn handle_client(stream: TcpStream) {
//     let peer_ip = stream.peer_addr().unwrap();
//     println!("Received connection from: {}", peer_ip);
// }
// fn main() -> std::io::Result<()> {
//     let listener = TcpListener::bind("127.0.0.1:8888")?;
//     // accept connections and process them serially
//     for stream in listener.incoming() {
//         handle_client(stream?);
//     }
//     Ok(())
// }