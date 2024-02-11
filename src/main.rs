use rfd::FileDialog;
use std::{
    fs::File,
    io::{BufReader, ErrorKind},
    path::{Path, PathBuf},
};

mod convert;

use convert::troybin_2_troy;

fn main() {
    /* Starts a ask open file dialog */
    let ask_open = FileDialog::new()
        .add_filter("Troy", &["troybin", "troy"])
        .set_directory("/")
        .set_title("Open .troybin file")
        .pick_file();

    /* Checks if picker return the file path, if not, it panics */
    let inname = match ask_open {
        Some(path) => path,
        None => panic!("None file given!"),
    };

    /*  Set the name of the output file, first checks if the input exists,
    if not, it panics.
        Second, checks if extract the file name from path returns something, if so,
    checks if it ends with .troybin, if so, changes the .troybin to .troy (in short, it changes the extension).*/
    let defout_name = {
        let path = Path::new(&inname);

        if path.is_file() {
            let mut defoutname = path.file_name().unwrap().to_os_string();
            if let Some(name) = defoutname.to_str() {
                if name.ends_with(".troybin") {
                    defoutname = PathBuf::from(name.replace(".troybin", ".troy")).into_os_string();
                    defoutname
                } else {
                    defoutname.push(".troybin");
                    defoutname
                }
            } else {
                let default_name = PathBuf::from("Converted_troybin.troy").into_os_string();
                default_name
            }
        } else {
            panic!("Input is not a file!");
        }
    }
    .into_string()
    .unwrap(); //TODO: Needs optimization!!!

    let ask_save = FileDialog::new()
        .add_filter("troy files", &["troy"])
        .set_file_name(defout_name)
        .set_title("Save .troy file")
        .save_file();

    let outname = match ask_save {
        Some(path) => path,
        None => panic!("You must save the file to be written!"),
    };

    let infile = File::open(inname).unwrap_or_else(|error| {
        if error.kind() == ErrorKind::UnexpectedEof {
            panic!("Not enough data to read!");
        } else {
            panic!("Problem opening the file: {:?}", error);
        }
    });

    let outfile = File::create(outname).unwrap_or_else(|error| {
        panic!("It was not possible to create the file: {:?}", error);
    });

    let file_buffer = BufReader::new(infile);

    troybin_2_troy(file_buffer, outfile);
}
