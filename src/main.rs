use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::channel;
use std::time::Duration;

fn try_compile(file: PathBuf, compiler: &str) {
	let gppflags =
		vec!["-fdiagnostics-color=always", "-O2", "-Wall", "-std=c++17"];
	let clangflags = vec!["-fcolor-diagnostics", "-O2", "-Wall", "-std=c++17"];

	let flags = match compiler {
		"g++" => gppflags,
		"clang++" => clangflags,
		_ => Vec::new(),
	};

	if file.is_file() {
		if let Some(ext) = file.extension() {
			if ext == "cpp" || ext == "cc" || ext == "c" {
				let output = Command::new(&compiler)
					.args(&flags)
					.arg(&file)
					.arg("-o")
					.arg(file.file_stem().unwrap())
					.stderr(Stdio::piped())
					.output();

				let output = match output {
					Ok(o) => o,
					Err(_) => {
						println!("failed to run the compiler");
						return; // exit function immediately
					}
				};

				if output.stderr.is_empty() {
					println!("{:?} was compiled successfully", file);
				} else {
					println!(
						"{}",
						String::from_utf8(output.stderr).expect("not UTF-8")
					);
				}
			}
		}
	}
}

fn handle(event: DebouncedEvent, compiler: &str) {
	match event {
		DebouncedEvent::Write(file) => {
			println!("WRITE: {:?} is written", &file);
			try_compile(file, compiler);
		}
		DebouncedEvent::Create(file) => {
			println!("CREATE: {:?} is created", &file);
			try_compile(file, compiler);
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
	if env::set_current_dir(&root).is_err() {
		println!("Invalid file or directory");
		return;
	}

	let possible_compilers = vec!["g++", "clang++"];

	let mut compiler = "";

	for c in possible_compilers {
		let test = Command::new("which").arg(c).output();
		if test.is_ok() {
			compiler = c;
		}
	}

	if compiler.is_empty() {
		println!("you don't have g++ or clang++ installed.");
		return;
	}
	println!(
		"The watcher will use {} as the compiler for C/C++",
		&compiler
	);

	println!("Setting up watcher.");

	let (tx, rx) = channel();

	let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();

	watcher
		.watch(path_or_file, RecursiveMode::Recursive)
		.unwrap();

	println!("Watcher setup success!");

	loop {
		match rx.recv() {
			Ok(event) => handle(event, compiler),
			Err(e) => println!("watch error: {:?}", e),
		}
	}
}
