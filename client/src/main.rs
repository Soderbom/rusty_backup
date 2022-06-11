use std::env;
use std::io::prelude::*;
use std::net::TcpStream;
use chrono::prelude::*;

fn main() {
    // Read the file to send with args
    let filename = env::args().nth(1).expect("");

    // Get the date to name the outfile
    let utc: DateTime<Utc> = Utc::now();  
    let out_name = format!("{}-{}.zip", utc.year(), utc.month());
    // Read the bytes
    let bytes = std::fs::read(&filename).unwrap();

    // Zip the file
    match zip_file(&filename, &out_name, &bytes) {
        Ok(_) => println!("File zipped successfully."),
        Err(e) => println!("Error {:?}", e),
    }

    let zip_content = std::fs::read(&out_name).unwrap();
    // Get the hash from the zip to verify the file transfer
    let zip_hash = sha256::digest_bytes(&zip_content);
    println!("Hash: {}", zip_hash);
    
    connect_to_server(&zip_hash, &zip_content);   

    // TODO Delete zip depending on result
}

fn zip_file(filename: &String, out_name: &String, bytes: &[u8]) -> zip::result::ZipResult<()> {
    // From https://github.com/zip-rs/zip/tree/bb230ef56adc13436d1fcdfaa489249d119c498f/examples
    let path = std::path::Path::new(&out_name);
    let file = std::fs::File::create(&path).unwrap();
    
    let mut zip = zip::ZipWriter::new(file);

    zip.start_file(filename, Default::default())?;
    zip.write_all(&bytes)?;
    Ok(())
}

// TODO make this return a result so that we can either delete the zip or try again
fn connect_to_server(zip_hash: &String, zip_content: &[u8]) {
    let file_size = (zip_content.len() as u32).to_be_bytes();

    println!("Len: {:?}", file_size);

    //https://riptutorial.com/rust/example/4404/a-simple-tcp-client-and-server-application--echo
    match TcpStream::connect("localhost:3333") {
        Ok(mut stream) => {
            println!("Successfully connected");

            // Send metadata
            let metadata = [zip_hash.as_bytes(), &file_size].concat();

            stream.write(&metadata).unwrap();

            let mut hash_ok = [0 as u8; 4];
            match stream.read(&mut hash_ok) {
                Ok(_) => {
                    if &hash_ok == b"true" {
                        println!("OK");

                        stream.write(&zip_content).unwrap();
                    } else {
                        println!("Fail");
                    }
                },
                Err(e) => println!("Error: {}", e),
            }

            let mut file_ok = [0 as u8; 5];
            match stream.read(&mut file_ok) {
                Ok(_) => {
                    if &file_ok == b"true " {
                        println!("File transfered successfully.");

                        stream.write(&zip_content).unwrap();
                    } else {
                        println!("File transfer failed.");
                    }
                },
                Err(e) => println!("Error: {}", e),
            }
        },
        Err(e) => println!("Something went wrong: {}", e),
    }
}