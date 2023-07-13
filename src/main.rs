use std::{ path::Path, env, fs, thread, time::Duration, sync::{Arc, atomic::{Ordering, AtomicBool}}};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher, Event, EventKind};
use notify::event::ModifyKind::Any;

fn main() {
    let found = Arc::new(AtomicBool::new(false));
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default()).unwrap();
    let temp_dir = env::var("TEMP").unwrap();
    watcher.watch(Path::new(&temp_dir), RecursiveMode::Recursive).unwrap();
    println!("Listening for workshop folder..");

    for res in rx {
        match res {
            Ok(event) => handle_change(event, found.clone()),
            Err(error) => println!("Error: {error:?}"),
        }
    }
}

fn handle_change(event: Event, found: Arc<AtomicBool>) {
    if found.load(Ordering::Relaxed) {
        return;
    }

    if event.kind == EventKind::Modify(Any) {
        let path = event.paths[0].parent().unwrap();
        let file_name = event.paths[0].file_name().unwrap();
        let folder_name = path.file_name().unwrap();
        if file_name == "icon_background.png" && folder_name.to_str().unwrap().starts_with("tmp") {
            found.store(true, Ordering::Relaxed);
            println!("Temp Workshop found: {:?}", path.file_name().unwrap());
            handle_copy(path, "icon_background.png");
            handle_copy(path, "icon.png");
            thread::spawn( move || {
                thread::sleep(Duration::from_secs(5));
                found.store(false, Ordering::Relaxed);
            });
        }
    }
}

fn handle_copy(folder: &Path, file: &str) {
    let source_path = Path::new("./to_inject").join(file);
    let target_path = Path::new(folder).join(file);
    
    if source_path.exists() {
        match fs::copy(source_path, target_path) {
            Ok(_) => println!("Successfully copied: {} ", file),
            Err(err) => println!("Failed to copy file: {}", err),
        }
    }
}