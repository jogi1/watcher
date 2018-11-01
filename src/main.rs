extern crate notify;

use notify::{RecommendedWatcher, Error, Watcher};
use std::sync::mpsc::channel;
use std::env;
use std::process::Command;
use std::{thread, time};

struct Process<'a> {
    child: std::process::Child,
    file: String,
    args: &'a[String],
}

fn start_process<'a>(f: String, args: &[String]) -> Process{
    let mut c = Command::new(f.clone());
    c.args(args);
    let process =  Process{file: f.clone(), args: args, child: c.spawn().expect("could not spawn process")};
    process
}

fn restart_process(watcher: &mut notify::RecommendedWatcher, process: &mut Process, _path: std::path::PathBuf, op: notify::Op) {
    if op != notify::op::CHMOD {
        return
    }
    println!("restarting {} {:?} {:?}", process.file, process.args, op);
    let duration = time::Duration::from_millis(150);
    thread::sleep(duration);
    process.child.kill().expect("could not kill process");
    let mut c = Command::new(&process.file);
    c.args(process.args);
    process.child =  c.spawn().expect("could not respawn process");
    let _ = watcher.watch(process.file.clone());
}


fn main() {
    // Create a channel to receive the events.
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Need at least 1 argument");
        return();
    }

    println!("Watchin {}", args[1]);
    let s: &String = &args[1];

    let mut process = start_process(s.clone(), &args[2..]);

    let (tx, rx) = channel();

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let w: Result<RecommendedWatcher, Error> = Watcher::new(tx);

    match w {
        Ok(mut watcher) => {
            let _r = watcher.watch(&args[1]);
            loop {
                match rx.recv() {
                    Ok(notify::Event{path: Some(path), op: Ok(op)}) => restart_process(&mut watcher, &mut process, path, op),
                    Ok(event) => println!("broken event: {:?}", event),
                    Err(e) => println!("watch error: {:?}", e),
                }
            }
        },
        Err(_) => println!("Error")
    }
}
