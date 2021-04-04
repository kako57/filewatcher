extern crate notify;

use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::channel;
use std::time::Duration;

fn try_compile(file: PathBuf) {
    let cxxflags = ["-fdiagnostics-color=always", "-O2", "-Wall", "-std=c++17"];
    if file.is_file() {
        if let Some(ext) = file.extension() {
            if ext == "cpp" || ext == "cc" || ext == "c" {
                let output = Command::new("g++")
                    .args(&cxxflags)
                    .arg(&file)
                    .arg("-o")
                    .arg(file.file_stem().unwrap())
                    .stderr(Stdio::piped())
                    .output();
                let output = match output {
                    Ok(o) => o,
                    Err(_) => {
                        println!("failed to run g++\ndo you even have g++?");
                        return (); // exits function immediately
                    }
                };
                if output.stderr.len() > 0 {
                    println!("{}", String::from_utf8(output.stderr).expect("not UTF-8"));
                } else {
                    println!("{:?} was compiled successfully", file);
                }
            }
        }
    }
}

fn handle(event: DebouncedEvent) {
    match event {
        DebouncedEvent::Write(file) => {
            println!("WRITE: {:?} is written", &file);
            try_compile(file);
        }
        DebouncedEvent::Create(file) => {
            println!("CREATE: {:?} is created", &file);
            try_compile(file);
        }
        DebouncedEvent::Remove(file) => {
            println!("REMOVE: {:?} was removed", file);
        }
        DebouncedEvent::Rename(oldname, newname) => {
            println!("RENAME {:?} was renamed to {:?}", oldname, newname);
        }
        DebouncedEvent::Chmod(file) => {
            println!("CHMOD: file permissions of {:?} has been changed", file);
        }
        DebouncedEvent::Rescan => {
            println!("RESCAN: problem detected\nrescanned file/directory.");
        }
        DebouncedEvent::Error(e, path) => {
            println!("ERROR: error {:?} at {:?}", e, path);
        }
        _ => {}
    }
}

fn main() {
    let mut args = env::args().skip(1);

    let path_or_file = match args.next() {
        Some(p) => p,
        _ => String::from("."),
    };

    let root = Path::new(&path_or_file);
    match env::set_current_dir(&root) {
        Err(_) => {
            println!("Invalid file or directory");
            return ();
        }
        _ => {}
    }

    println!("Setting up watcher.");

    let (tx, rx) = channel();

    let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();

    watcher
        .watch(path_or_file, RecursiveMode::Recursive)
        .unwrap();

    println!("Setup success!");

    loop {
        match rx.recv() {
            Ok(event) => handle(event),
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}
