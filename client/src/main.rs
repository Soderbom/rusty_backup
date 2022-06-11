use std::{env, fs};
use std::io::prelude::*;
use std::net::TcpStream;
use chrono::prelude::*;

struct FileData {
    filename: String,
    bytes: Vec<u8>,
    zip_filename: String,
}

impl FileData {
    fn new(filename: String, file_bytes: Vec<u8>) -> FileData {
        let utc: DateTime<Utc> = Utc::now();  
        let zip_filename = format!("{}-{}.zip", utc.year(), utc.month());

        FileData { 
            filename: filename, 
            zip_filename: zip_filename,
            bytes: file_bytes, 
        }        
    }

    fn zip_file(&self) {
        // From https://github.com/zip-rs/zip/tree/bb230ef56adc13436d1fcdfaa489249d119c498f/examples
        let path = std::path::Path::new(&self.zip_filename);
        let file = std::fs::File::create(&path).unwrap();
        
        let mut zip = zip::ZipWriter::new(file);
    
        zip.start_file(&self.filename, Default::default()).unwrap();
        zip.write_all(&self.bytes).unwrap();
    }

    

    fn get_hash(&self) -> String {
        let zip_content = std::fs::read(&self.zip_filename).unwrap();
        // Get the hash from the zip to verify the file transfer
        sha256::digest_bytes(&zip_content)
    }

    fn get_zip_information(&self) -> ([u8; 4], Vec<u8>) {
        let zip_bytes = std::fs::read(&self.zip_filename).unwrap();
        ((zip_bytes.len() as u32).to_be_bytes(), zip_bytes)
        
    }
}

fn main() {
    // Read the file to send with args
    let filename = env::args().nth(1).expect("");
    let bytes = std::fs::read(&filename).unwrap();

    let file = FileData::new(filename, bytes);
    file.zip_file();
    
    let success = connect_to_server(&file);   

    match success {
        Ok(()) => {
            fs::remove_file(file.zip_filename).unwrap();
        },
        Err(()) => println!("The transaction failed."),
    }
}

fn connect_to_server(file: &FileData) -> Result<(), ()> {
    let mut success: Result<(), ()> = Err(());

    //https://riptutorial.com/rust/example/4404/a-simple-tcp-client-and-server-application--echo
    match TcpStream::connect("localhost:3333") {
        Ok(mut stream) => {
            println!("Successfully connected");

            let (file_size, bytes) = file.get_zip_information();

            // Send metadata
            let metadata = [file.get_hash().as_bytes(), &file_size].concat();

            stream.write(&metadata).unwrap();

            match talk_to_server(&mut stream) {
                Ok(_) => {
                    stream.write(&bytes).unwrap();
                    success = talk_to_server(&mut stream);
                },
                Err(()) => println!("An error occured."),
            }                        
        },
        Err(e) => println!("Something went wrong: {}", e),
    }
    return success
}

fn talk_to_server(stream: &mut TcpStream) -> Result<(), ()> {
    let mut response = [0 as u8; 3];

    match stream.read(&mut response) {
        Ok(_) => {
            if &response == b"200" {
                println!("OK");
                Ok(())
            } else {
                println!("500");
                Err(())
            }
        },
        Err(e) => {
            println!("Error: {}", e);
            Err(())
        }
    }     
    
}