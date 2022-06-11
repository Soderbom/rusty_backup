use std::io::prelude::*;
use chrono::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::fs::File;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:3333").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Connection established with: {}", stream.peer_addr().unwrap());
                handle_client(stream)
            },
            Err(e) => println!("An error occured: {}", e)
        }
    }
}


fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 69];

    // Recieve file hash and size
    stream.read(&mut buffer).unwrap();
    let hash = &buffer[..64];
    let file_size = u32::from_be_bytes(buffer[64..68].try_into().unwrap()) as usize;
    println!("Expecting file with hash {:?} and size {}B", String::from_utf8_lossy(&hash), &file_size);

    stream.write(b"true").unwrap();

    let mut zipfile = vec![0 as u8; file_size];

    match stream.read_exact(&mut zipfile) {
        Ok(_) => save_zip(zipfile, hash, &mut stream),
        Err(e) => println!("Error: {}", e),
    }
}

fn save_zip(zipfile: Vec<u8>, hash: &[u8], stream: &mut TcpStream) {
    // Get the date to name the outfile
    let utc: DateTime<Utc> = Utc::now();  
    let out_name = format!("{}-{}.zip", utc.year(), utc.month());

    let zip_hash = sha256::digest_bytes(&zipfile);

    if zip_hash == String::from_utf8_lossy(hash) {
        let mut file = File::create(out_name).unwrap();
        file.write_all(&zipfile).unwrap();
        stream.write(b"true ").unwrap();
        
    } else {
        println!("The file is corrupt!");
        stream.write(b"false").unwrap();
    }
    

}