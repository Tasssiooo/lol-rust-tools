use std::{fs::File, io::BufReader};

mod inibin2;
mod inibin_fix;

use inibin2::read;

pub fn troybin_2_troy(infile: BufReader<File>, outfile: File) {
  let ibin = read(infile);
}
