use std::{
    env, fs,
    os::unix,
    path::{Path, PathBuf},
    process::Command,
    sync::{mpsc, Arc, Mutex},
    thread,
};

use notify::{
    event::{AccessKind, AccessMode, ModifyKind, RenameMode},
    Event, Result, Watcher,
};

fn get_env(key: &str, default: &str) -> String {
    env::var(key).unwrap_or(default.to_string())
}

fn get_int_env(key: &str, default: &str) -> u32 {
    get_env(key, default).parse::<u32>().unwrap()
}

fn process(
    receiver: Arc<Mutex<mpsc::Receiver<PathBuf>>>,
    args: &[String],
    watch_dir: &str,
    transcoding_dir: &str,
    complete_dir: &str,
    hwaccel: &str,
    puid: u32,
    pgid: u32,
) {
    loop {
        let full_path = receiver.lock().unwrap().recv().unwrap();
        let file_path = full_path.to_str().unwrap();
        let relative_path = file_path.strip_prefix(watch_dir).unwrap_or(&file_path);
        let transcoding_path = format!("{}/{}", transcoding_dir, relative_path);
        let output_path = format!("{}/{}", complete_dir, relative_path);

        let status = Command::new("ffmpeg")
            .arg("-hwaccel")
            .arg(hwaccel)
            .arg("-i")
            .arg(&file_path)
            .args(args)
            .arg(&transcoding_path)
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status()
            .expect("failed to execute process");

        if status.success() {
            println!("Transcoding successful: {}", relative_path);
            fs::rename(transcoding_path, &output_path).unwrap();
            unix::fs::chown(&Path::new(&output_path), Some(puid), Some(pgid)).unwrap();
        } else {
            println!("Transcoding failed: {}", relative_path);
            fs::remove_file(&transcoding_path).ok();
        }
    }
}

fn main() {
    let watch_dir = get_env("WATCH_DIR", "/data/watch");
    let complete_dir = get_env("COMPLETE_DIR", "/data/complete");
    let transcoding_dir = get_env("TRANSCODING_DIR", "/data/transcoding");
    let allowed_extensions: Vec<_> = get_env("ALLOWED_EXTENSIONS", "mkv,mp4,avi,mov,flv")
        .split(",")
        .map(String::from)
        .collect();
    let hwaccel = get_env("HWACCEL", "auto");
    let puid = get_int_env("PGID", "1000");
    let pgid = get_int_env("PUID", "1000");
    let args = env::args().skip(1).collect::<Vec<String>>();

    fs::create_dir_all(&watch_dir).unwrap();
    fs::create_dir_all(&complete_dir).unwrap();
    fs::create_dir_all(&transcoding_dir).unwrap();

    let (sender, receiver) = mpsc::channel::<PathBuf>();
    let receiver = Arc::new(Mutex::new(receiver));

    let input_dir = watch_dir.clone();
    let consumer = thread::spawn(move || {
        process(
            receiver,
            &args,
            &input_dir,
            &transcoding_dir,
            &complete_dir,
            &hwaccel,
            puid,
            pgid,
        );
    });

    let (watcher_sender, watcher_receiver) = mpsc::channel::<Result<Event>>();

    let mut watcher = notify::recommended_watcher(watcher_sender).unwrap();
    watcher
        .watch(&Path::new(&watch_dir), notify::RecursiveMode::Recursive)
        .unwrap();

    println!("Watching for files in {}", watch_dir);

    for res in watcher_receiver {
        match res {
            Ok(event) => match event.kind {
                notify::EventKind::Access(AccessKind::Close(AccessMode::Write))
                | notify::EventKind::Modify(ModifyKind::Name(RenameMode::To)) => {
                    for path in event.paths {
                        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                            if allowed_extensions.contains(&ext.to_string()) {
                                println!("File found: {:?}", path);
                                sender.send(path.clone()).unwrap();
                            }
                        }
                    }
                }
                _ => {
                    println!("{:?}", event.kind)
                }
            },
            Err(e) => {
                println!("watch error: {:?}", e);
            }
        }
    }
    consumer.join().unwrap();
}
