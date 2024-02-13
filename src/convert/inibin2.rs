use std::collections::HashMap;
use std::fs::File;
use std::io::{stderr, BufReader, Read, Result, Seek, SeekFrom, Write};

pub fn read_2(
    buffer: &mut BufReader<File>,
    target: &mut HashMap<String, HashMap<u32, Vec<String>>>,
) {
    fn read_numbers(
        buffer: &mut BufReader<File>,
        fmt: &str,
        count: usize,
        mul: f32,
    ) -> HashMap<u32, Vec<String>> {
        let mut result = HashMap::new();

        let num = {
            let mut num_buf = [0; 2];
            buffer.read(&mut num_buf).unwrap();
            u16::from_le_bytes(num_buf)
        };

        let mut keys = Vec::new();

        for _ in 0..num {
            let mut buf = [0; 4];
            buffer.read(&mut buf).unwrap();
            keys.push(u32::from_le_bytes(buf));
        }
        for x in 0..num {
            let mut tmp = Vec::with_capacity(count);
            for _ in 0..count {
                let mut is_float: f32 = 0.0;
                /* TODO: Needs optimization */
                let val = match fmt {
                    "<i" => {
                        let mut i32_buf = [0; 4];
                        buffer.read(&mut i32_buf).unwrap();
                        i32::from_le_bytes(i32_buf) as u32
                    }
                    "<f" => {
                        let mut f32_buf = [0; 4];
                        buffer.read(&mut f32_buf).unwrap();
                        is_float = f32::from_le_bytes(f32_buf);
                        0
                    }
                    "<B" => {
                        let mut u8_buf = [0; 1];
                        buffer.read(&mut u8_buf).unwrap();
                        u8::from_le_bytes(u8_buf) as u32
                    }
                    "<h" => {
                        let mut i16_buf = [0; 2];
                        buffer.read(&mut i16_buf).unwrap();
                        i16::from_le_bytes(i16_buf) as u32
                    }
                    "<q" => {
                        let mut i64_buf = [0; 8];
                        buffer.read(&mut i64_buf).unwrap();
                        i64::from_le_bytes(i64_buf) as u32
                    }
                    "<H" => {
                        let mut u16_buf = [0; 2];
                        buffer.read(&mut u16_buf).unwrap();
                        u16::from_le_bytes(u16_buf) as u32
                    }
                    _ => panic!("Unsupported format"),
                };
                if is_float != 0.0 {
                    tmp.push((is_float * mul).to_string());
                } else {
                    tmp.push((val * mul as u32).to_string());
                }
            }
            result.insert(keys[x as usize], tmp);
        }
        result
    }
    fn read_bools(buffer: &mut BufReader<File>) -> HashMap<u32, Vec<String>> {
        let mut result = HashMap::new();

        let num = {
            let mut num_buf = [0; 2];
            buffer.read(&mut num_buf).unwrap();
            u16::from_le_bytes(num_buf)
        };

        let mut keys = Vec::new();

        for _ in 0..num {
            let mut buf = [0; 4];
            buffer.read(&mut buf).unwrap();
            keys.push(u32::from_le_bytes(buf));
        }
        let bytes_count = (num / 8) as usize + (if num % 8 > 0 { 1 } else { 0 });
        let mut bools = vec![0; bytes_count];
        buffer.read(&mut bools).unwrap();

        assert_eq!(bytes_count, bools.len());

        for x in 0..num {
            let index = (x / 8) as usize;
            let value = vec![(((bools[index] >> (x % 8)) & 1) as u32).to_string()];
            result.insert(keys[x as usize], value);
        }
        result
    }
    fn read_strings(buffer: &mut BufReader<File>, len: u16) -> HashMap<u32, Vec<String>> {
        let mut result = HashMap::new();

        let offsets = read_numbers(buffer, "<H", 1, 1.0);

        let mut data = vec![0; len.into()];
        buffer.read(&mut data).unwrap();
        assert_eq!(data.len(), len.into());

        for (key, offset) in offsets {
            let mut o = offset[0].parse::<u32>().unwrap() as usize;
            let mut t = String::new();

            while data[o] != 0 {
                t.push(data[o] as char);
                o += 1;
            }

            result.insert(key, vec![t]);
        }
        result
    }

    let strings_length = {
        let mut len_buf = [0; 2];
        buffer.read(&mut len_buf).unwrap();
        u16::from_le_bytes(len_buf)
    };

    let mut flags = {
        let mut flags_buf = [0; 2];
        buffer.read(&mut flags_buf).unwrap();
        u16::from_le_bytes(flags_buf)
    };

    flags = {
        let current_pos = buffer.get_ref().seek(SeekFrom::Current(0)).unwrap();
        let total_size = buffer.get_ref().metadata().unwrap().len();

        if flags == 0 {
            if current_pos != total_size {
                let mut new_buf = [0; 2];
                buffer.read(&mut new_buf).unwrap();
                u16::from_le_bytes(new_buf)
            } else {
                panic!("look over here!");
            }
        } else {
            flags
        }
    };

    let read_conf = [
        ("<i", 1, 1.0),         //0  - 1 x int
        ("<f", 1, 1.0),         //1  - 1 x float
        ("<B", 1, 0.1),         //2  - 1 x byte * 0.1
        ("<h", 1, 1.0),         //3  - 1 x short
        ("<B", 1, 1.0),         //4  - 1 x byte
        ("bool", 1, 1.0),       //5  - 1 x bools
        ("<B", 3, 0.1),         //6  - 3 x byte * 0.1
        ("<f", 3, 1.0),         //7  - 3 x float
        ("<B", 2, 0.1),         //8  - 2 x byte * 0.1
        ("<f", 2, 1.0),         //9  - 2 x float
        ("<B", 4, 0.1),         //10 - 4 x byte * 0.1
        ("<f", 4, 1.0),         //11 - 4 x float
        ("string_len", 1, 1.0), //12 - strings
        /* TODO: are strings stored at the end of file allways?? */
        ("<q", 1, 1.0), //13 - long long
    ];
    if flags & (1 << 13) != 0 {
        writeln!(stderr(), "Found long long!").unwrap();
    };
    assert!(!(flags & (1 << 14) != 0));
    assert!(!(flags & (1 << 15) != 0));

    let mut val = Vec::new();

    for x in 0..16 {
        if flags & (1 << x) != 0 {
            if x < 13 {
                let element = read_conf[x];

                if element.0 == "bool" {
                    val.push(read_bools(buffer));
                } else if element.0 == "string_len" {
                    val.push(read_strings(buffer, strings_length));
                } else {
                    val.push(read_numbers(buffer, element.0, element.1, element.2));
                }
            }
        }
    }

    target.insert(String::from("UNKNOWN_HASHES"), val.into_iter().flat_map(|x| {
        x
    }).collect());
}

pub fn read_1(_buffer: &mut BufReader<File>) -> Result<()> {
    Ok(())
}

pub fn read(mut buffer: BufReader<File>) -> HashMap<String, HashMap<u32, Vec<String>>> {
    let mut result = HashMap::new();

    let version = {
        let mut version_buf = [0; 1];
        buffer.read(&mut version_buf).unwrap();
        version_buf[0]
    };

    if version == 2 {
        read_2(&mut buffer, &mut result);
    } else if version == 1 {
        read_1(&mut buffer).unwrap();
    } else {
        panic!("Unknown version!");
    }

    let current_pos = buffer.seek(SeekFrom::Current(0)).unwrap();
    let total_size = buffer.get_ref().metadata().unwrap().len();

    if current_pos == total_size {
        println!("{current_pos} is equal to {total_size}");
    } else {
        println!("It is not equal :(")
    };

    result
}
