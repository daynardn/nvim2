use std::{fs::File, io::{Read, Write}};

use gpui::SharedString;


/** returns vec of shared string */
pub fn load_file(path: String) -> Vec<SharedString>{
    let file = File::open(path.clone());
    if file.is_err() {
        println!("{}", path + " NOT FOUND");
    }

    let mut file = file.unwrap();
    let mut buf  = String::new();
    let _ = file.read_to_string(&mut buf);

    let parts: Vec<String> = buf.split(&"\n".to_string()).map(str::to_string).collect();
    // convert to shared string
    return parts.into_iter().map(Into::into).collect(); 
}

pub fn save(path: String, lines: Vec<SharedString>) {
    let file =  File::create(path.clone());
    if file.is_err() {
        println!("{}", path + " NOT FOUND");
    }
    let mut file = file.unwrap();
    for line in lines {
        let mut string: String = line.into();
        string += "\n";
        println!("{}", string);
        file.write(string.as_bytes()).expect("Unable to write data");
    }
}