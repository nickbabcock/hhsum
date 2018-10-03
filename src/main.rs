#[macro_use]
extern crate structopt;
extern crate highway;
extern crate memmap;
extern crate rayon;
#[macro_use]
extern crate failure;

mod errors;

use errors::HashError;
use failure::Fail;
use highway::{HighwayBuilder, HighwayHash, Key};
use memmap::Mmap;
use rayon::prelude::*;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(name = "FILE", parse(from_os_str))]
    files: Vec<PathBuf>,
}

const MEMMAP_SIZE_BARRIER: u64 = 32000;

struct Checksum {
    path: String,
    lo: u128,
    hi: u128,
}

fn calculate_hash_of_path(path: &PathBuf) -> Result<Checksum, HashError> {
    let key = Key::default();
    let pathed = path.to_string_lossy().into_owned();
    let meta = path.metadata().map_err(|err| HashError::Metadata {
        file: pathed.clone(),
        error: err,
    })?;

    let mut file = File::open(path).map_err(|err| HashError::File {
        file: pathed.clone(),
        error: err,
    })?;

    if meta.len() < MEMMAP_SIZE_BARRIER {
        let mut data = [0; 0x2000];
        let mut builder = HighwayBuilder::new(&key);
        let mut done = false;
        while !done {
            let read = file.read(&mut data[..]).map_err(|err| HashError::Read {
                file: pathed.clone(),
                error: err,
            })?;

            builder.append(&data[..read]);
            done = read < data.len();
        }

        let (lo, hi) = builder.finalize256();
        Ok(Checksum {
            path: pathed,
            lo,
            hi,
        })
    } else {
        let mmap = unsafe { Mmap::map(&file) }.map_err(|err| HashError::Memmap {
            file: pathed.clone(),
            error: err,
        })?;

        let (lo, hi) = HighwayBuilder::new(&key).hash256(&mmap[..]);
        Ok(Checksum {
            path: pathed,
            lo,
            hi,
        })
    }
}

fn main() {
    let opt = Opt::from_args();
    let results: Vec<Result<Checksum, HashError>> =
        opt.files.par_iter().map(calculate_hash_of_path).collect();

    for result in results {
        match result {
            Ok(sum) => {
                println!("{:#016x}{:016x} {}", sum.hi, sum.lo, sum.path);
            }
            Err(ref e) => {
                for c in Fail::iter_chain(e) {
                    eprintln!("\t{}", c);
                }
            }
        }
    }
}
