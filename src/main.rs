#[macro_use] extern crate const_cstr;

use std::env;
use std::io;
use std::fs::File;
use std::io::Write;
use std::ptr::null_mut;
use libc;
use libc::*;
use byteorder::{LittleEndian, WriteBytesExt};
// use std::os::raw::{c_char};
// use std::ffi::{CString, CStr};

fn write_file(path: &str, begin: u64, end: u64) -> io::Result<()> {
    let mut buf = vec![];
    for num in begin..end {
        buf.write_u64::<LittleEndian>(num)?;
    }
    let mut f = File::create(path)?;
    f.write_all(&buf)?;
    Ok(())
}

fn write_files() -> io::Result<()> {
    println!("Writing files..");

    // Two 8 MiB files.
    write_file("a.bin", 0, 1048576)?;
    write_file("b.bin", 1048576, 2097152)?;
    Ok(())
}

fn page_size() -> usize {
    unsafe { sysconf(_SC_PAGESIZE) as usize }
}

fn mmap_files() -> io::Result<()> {
/*
    pub unsafe extern "C" fn mmap(
        addr: *mut c_void,
        len: size_t,
        prot: c_int,
        flags: c_int,
        fd: c_int,
        offset: off_t
    ) -> *mut c_void
*/
    // https://lo.calho.st/posts/black-magic-buffer/

    // size of either file.
    let p = page_size();
    println!("p: {}", p);
    let eight_mib : usize = 16 * 1024 * 1024;
    let master_size = 2 * eight_mib;

    // map virtual memory
    let mem : *mut c_void = unsafe {
            mmap(
                null_mut(),
                master_size,
                PROT_READ,
                MAP_PRIVATE | MAP_ANONYMOUS,
                -1,
                0)
        };
    println!("mem: {}", mem as usize);
    assert_ne!(mem, MAP_FAILED);
    assert_eq!((mem as usize) % p, 0);

    // let a_path = const_cstr!("a.bin");
    // let a_fd = unsafe { open(a_path.as_ptr(), O_RDWR) };
    // assert_ne!(a_fd, -1);
    // let a_begin = mem;
    // let a = unsafe {
    //         mmap(
    //             a_begin,
    //             eight_mib,
    //             PROT_READ | PROT_WRITE,
    //             MAP_FIXED | MAP_SHARED,
    //             a_fd,
    //             0)
    //     };
    // assert_ne!(a, MAP_FAILED);
    // assert_eq!(a, a_begin);

    let b_path = const_cstr!("b.bin");
    let b_fd = unsafe { open(b_path.as_ptr(), O_RDONLY) };
    assert_ne!(b_fd, -1);
    let b_start = unsafe { mem.add(eight_mib) };
    println!("b_start: {}", b_start as usize);
    let b = unsafe {
            mmap(
                b_start,
                eight_mib,
                PROT_READ,
                MAP_FIXED | MAP_SHARED,
                b_fd,
                0)
        };
    assert_ne!(b, MAP_FAILED);
    assert_eq!(b, b_start);
    assert_eq!((b as usize) % p, 0);

    let contig = mem as *mut u64;
    // for n in 0..(eight_mib / 8) {
    //     println!("contig[{}]: {}", n, unsafe { *contig.add(n) });
    // }

    println!("contig[0]: {}", unsafe { *contig });
    println!("contig[1]: {}", unsafe { *contig.add(1) });
    println!("contig[1048575]: {}", unsafe { *contig.add(1048575) });
    println!("contig[1048576]: {}", unsafe { *contig.add(1048576) });
    println!("contig[1048577]: {}", unsafe { *contig.add(1048577) });

    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    // println!("{:?}", args);
    if (args.len() == 2) && (args[1] == "write_files") {
        write_files()
    }
    else {
        mmap_files()
    }
}
