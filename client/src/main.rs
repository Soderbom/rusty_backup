use std::fs;
use std::io::prelude::*;
use std::net::TcpStream;
use chrono::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use walkdir::WalkDir;

struct FileData {
    foldername: String,
    zip_filename: String,
}

impl FileData {
    fn new(foldername: String) -> FileData {
        let utc: DateTime<Utc> = Utc::now();  
        let zip_filename = format!("filename-{}-{}.zip", utc.year(), utc.month());

        FileData { 
            foldername: foldername, 
            zip_filename: zip_filename,
        }        
    }

    fn zip_folder(&self, folder: String) {
        let path = Path::new(&self.zip_filename);
        let file = File::create(&path).unwrap();

        let walkdir = WalkDir::new(&folder);
        let folder_iter = walkdir.into_iter().filter_map(|e| e.ok());

        let mut zip = zip::ZipWriter::new(file);

        let mut buffer = Vec::new();

        for item in folder_iter {
            let path = item.path();
            let name = path.strip_prefix(Path::new(&folder)).unwrap();

            if path.is_file() {
                zip.start_file(name.to_str().unwrap(), Default::default()).unwrap();
                let mut f = File::open(path).unwrap();

                f.read_to_end(&mut buffer).unwrap();
                zip.write_all(&buffer).unwrap();
                buffer.clear();

            } else if !name.as_os_str().is_empty() {
                zip.add_directory(name.to_str().unwrap(), Default::default()).unwrap();
            }
        }
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
    let folders = locations();
    println!("[+] Config containing {} folders.", folders.len());

    for folder in folders {
        let foldername = String::from(folder.split('/').last().unwrap());

        println!("[+] Working on: {}", foldername);

        let file = FileData::new(foldername);
        file.zip_folder(folder);
        
        let success = connect_to_server(&file);   

        match success {
            Ok(()) => {
                fs::remove_file(file.zip_filename).unwrap();
            },
            Err(()) => println!("The transaction failed."),
        }
    }
}

fn locations() -> Vec<String> {
    let file = File::open("folders.conf").unwrap();
    let reader = BufReader::new(file);

    let mut folders = Vec::<String>::new();

    for line in reader.lines() {
        let line = line.unwrap();
        folders.push(line);
    }

    return folders
}

fn connect_to_server(file: &FileData) -> Result<(), ()> {
    let mut success: Result<(), ()> = Err(());

    //https://riptutorial.com/rust/example/4404/a-simple-tcp-client-and-server-application--echo
    match TcpStream::connect("localhost:3333") {
        Ok(mut stream) => {
            println!("Successfully connected");

            let (file_size, bytes) = file.get_zip_information();

            // Send metadata
            let metadata = [file.get_hash().as_bytes(), &file_size, &file.foldername.as_bytes()].concat();

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