use std::io::prelude::*;
use chrono::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::fs::File;
use std::str;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:3333").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {                
                handle_client(stream);
            },
            Err(e) => println!("An error occured: {}", e)
        }
    }
}

fn handle_client(mut stream: TcpStream){
    let mut buffer = [0; 100];

    // Recieve file hash and size
    match stream.read(&mut buffer) {
        Ok(_) => {
            println!("Connection established with: {}", stream.peer_addr().unwrap());
            let hash = &buffer[..64];
            let file_size = u32::from_be_bytes(buffer[64..68].try_into().unwrap()) as usize;
            let folder_name = str::from_utf8(&buffer[68..]).unwrap().replace("\0", "");

            println!("Expecting file with hash {:?} and size {}B", String::from_utf8_lossy(&hash), &file_size);

            stream.write(b"200").unwrap();

            let mut zipfile = vec![0 as u8; file_size];

            match stream.read_exact(&mut zipfile) {
                Ok(_) => save_zip(folder_name, zipfile, hash, &mut stream),
                Err(e) => println!("Error: {}", e),
            } 
        },
        Err(e) => println!("Connection lost with error: {}", e),
    }
}

fn save_zip(folder_name: String, zipfile: Vec<u8>, hash: &[u8], stream: &mut TcpStream) {
    // Get the date to name the outfile
    let utc: DateTime<Utc> = Utc::now();  
    let out_name = format!("{}_{}-{}.zip", folder_name, utc.year(), utc.month());
    println!("{}", out_name);

    let zip_hash = sha256::digest_bytes(&zipfile);

    if zip_hash == String::from_utf8_lossy(hash) {
        let mut file = File::create(out_name).unwrap();
        file.write_all(&zipfile).unwrap();
        stream.write(b"200").unwrap();
        
    } else {
        println!("The file is corrupt!");
        stream.write(b"500").unwrap();
    }
}