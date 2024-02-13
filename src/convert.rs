use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Write},
};

mod inibin2;
mod inibin_fix;

use inibin2::read;
use inibin_fix::fix;

fn write_ini(ibin: HashMap<String, HashMap<String, Vec<String>>>, outfile: &mut File) {
    let mut sections: Vec<&String> = ibin.keys().collect();
    sections.sort_by(|a, b| b.partial_cmp(a).unwrap());

    for section in sections {
        outfile
            .write_all(format!("[{}]\n", section).as_bytes())
            .unwrap(); //TODO: Error handling;

        let mut names: Vec<&String> = ibin[section].keys().collect();
        names.sort();

        for name in names {
            let value = &ibin[section][name].join(" ");
            outfile
                .write_all(format!("{}={}\n", name, value).as_bytes())
                .unwrap();
        }
    }
}

pub fn troybin_2_troy(infile: BufReader<File>, mut outfile: File) {
    let mut ibin = read(infile);
    let fixedtbin = fix(&mut ibin);
    write_ini(fixedtbin, &mut outfile);
}
