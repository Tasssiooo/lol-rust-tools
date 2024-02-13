use std::{collections::HashMap, vec};

const COMMENTS: [&str; 2] = ["", "'"];

mod constants;

use constants::FIELD_NAMES;
use constants::FLUID_NAMES;
use constants::GROUP_NAMES;
use constants::PART_FIELD_NAMES;
use constants::PART_FLUID_NAMES;
use constants::PART_GROUP_NAMES;
use constants::SYSTEM_NAMES;

fn ihash(value: &str, ret: u32) -> u32 {
    let mut ret = ret;
    for c in value.chars() {
        ret =
            ((c.to_ascii_lowercase() as u32).wrapping_add(65599u32.wrapping_mul(ret))) & 0xffffffff;
    }
    ret
}

fn a_ihash(sections: &Vec<String>, names: &[&str]) -> Vec<(String, String, u32)> {
    let mut results = Vec::new();
    for section in sections {
        let sectionhash = ihash("*", ihash(section, 0));
        for rawname in names {
            for com in COMMENTS {
                let name = format!("{}{}", com, rawname);
                let ret = ihash(&name, sectionhash);
                results.push((section.to_string(), name, ret));
            }
        }
    }
    results
}

fn get_values(
    tbin: &mut HashMap<String, HashMap<u32, Vec<String>>>,
    sections: &Vec<String>,
    names: &[&str],
) -> Vec<String> {
    let mut results = Vec::new();
    let unks = &tbin["UNKNOWN_HASHES"];
    for (_section, _name, h) in a_ihash(sections, names) {
        if unks.contains_key(&h) {
            results.push(unks[&h].clone().into_iter().flat_map(|x| vec![x]).collect());
        }
    }
    results
}

fn get_fixdict(
    tbin: &mut HashMap<String, HashMap<u32, Vec<String>>>,
) -> HashMap<u32, (String, String)> {
    let groups = get_values(tbin, &vec!["System".to_string()], &PART_GROUP_NAMES);
    let fields = get_values(tbin, &groups, &PART_FIELD_NAMES);
    let fluids = get_values(tbin, &groups, &PART_FLUID_NAMES);

    let mut fixdict = HashMap::new();
    for (s, n, h) in a_ihash(&groups, &GROUP_NAMES)
        .into_iter()
        .chain(a_ihash(&fields, &FIELD_NAMES))
        .chain(a_ihash(&fluids, &FLUID_NAMES))
        .chain(a_ihash(&vec!["System".to_string()], &SYSTEM_NAMES))
    {
        fixdict.insert(h, (s, n));
    }
    fixdict
}

pub fn fix(
    tbin: &mut HashMap<String, HashMap<u32, Vec<String>>>,
) -> HashMap<String, HashMap<String, Vec<String>>> {
    let fixd = get_fixdict(tbin);

    let mut vals = HashMap::new();
    let unks = tbin.get_mut("UNKNOWN_HASHES").unwrap();

    for (h, (s, n)) in fixd {
        if unks.contains_key(&h) {
            if !vals.contains_key(&s) {
                let mut subhashmap = HashMap::new();
                subhashmap.insert(n.clone(), unks[&h].clone());
                vals.insert(s.to_string(), subhashmap);
            }
            vals.get_mut(&s).unwrap().insert(n, unks[&h].clone());

            unks.remove(&h);
        }
    }
    vals
}
