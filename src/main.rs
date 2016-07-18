extern crate winapi;
extern crate kernel32;
extern crate advapi32;
extern crate userenv;

use std::error::Error;
use std::ffi::OsString;
use std::io;
use std::io::Write;
use std::os::windows::ffi::OsStringExt;
use std::process::Command;
use kernel32::{CloseHandle, GetCurrentProcess};
use advapi32::OpenProcessToken;
use userenv::{CreateEnvironmentBlock, DestroyEnvironmentBlock};

unsafe fn next_null(wchars: *const u16, start: isize) -> isize {
    let mut pos = start;

    while *wchars.offset(pos) != 0 {
        pos += 1;
    }

    pos
}

unsafe fn next_equals(wchars: *const u16, start: isize, end: isize) -> Option<isize> {
    for i in start..end {
        if *wchars.offset(i) == b'=' as u16 {
            return Some(i);
        }
    }

    None
}

unsafe fn to_os_str(wchars: *const u16, start: isize, end: isize) -> OsString {
    let slice = std::slice::from_raw_parts(wchars.offset(start), (end - start) as usize);
    OsString::from_wide(slice)
}

unsafe fn parse_environment(environment_block: *const winapi::c_void) -> Vec<(OsString, OsString)> {
    let mut pairs = Vec::new();

    let wchars = environment_block as *const u16;
    let mut start = 0;

    loop {
        let end = next_null(wchars, start);

        if let Some(equals) = next_equals(wchars, start, end) {
            let name = to_os_str(wchars, start, equals);
            let value = to_os_str(wchars, equals + 1, end);
            pairs.push((name, value));
        }

        start = end + 1;

        if *wchars.offset(start) == 0 {
            break;
        }
    }

    pairs
}

fn get_clean_env() -> Result<Vec<(OsString, OsString)>, io::Error> {
    unsafe {
        let mut token = std::ptr::null_mut();

        if OpenProcessToken(GetCurrentProcess(), winapi::TOKEN_READ, &mut token) == 0 {
            return Err(io::Error::last_os_error());
        }

        let mut environment_block = std::ptr::null_mut();

        if CreateEnvironmentBlock(&mut environment_block, token, winapi::FALSE) == 0 {
            CloseHandle(token);
            return Err(io::Error::last_os_error());
        }

        let clean_env = parse_environment(environment_block);

        DestroyEnvironmentBlock(environment_block);
        CloseHandle(token);
        Ok(clean_env)
    }
}

fn print_usage() {
    let text = "Usage: with-clean-env cmd [arg1 arg2 ...]

Summary:

    Runs a command with a clean environment. In particular, the
    command does not inherit the current process environment.

For example:

    with-clean-env cmd /c echo hello";

    writeln!(std::io::stderr(), "{}", text).unwrap();
}

fn die(msg: &str) -> ! {
    writeln!(std::io::stderr(), "with-clean-env: {}", msg).unwrap();
    std::process::exit(1);
}

fn main() {
    let mut args = std::env::args_os();

    if args.len() < 2 {
        print_usage();
        std::process::exit(2);
    }

    let mut command = Command::new(args.nth(1).unwrap());

    for arg in args {
        command.arg(arg);
    }

    let clean_env = match get_clean_env() {
        Ok(pairs) => pairs,
        Err(err) => die(err.description()),
    };

    for pair in &clean_env {
        command.env(&pair.0, &pair.1);
    }

    match command.status() {
        Ok(status) => {
            if let Some(code) = status.code() {
                std::process::exit(code);
            }

            die("could not read processs exit code");
        }
        Err(err) => die(err.description()),
    }
}
