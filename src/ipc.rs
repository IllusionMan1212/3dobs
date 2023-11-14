use std::{
    fs::File,
    io::{Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
    process::exit,
    sync::mpsc::Receiver,
    thread,
};

use fs4::FileExt;
use log::error;

fn create_named_pipe(pipe_path: PathBuf) -> UnixListener {
    if pipe_path.exists() {
        // Remove the existing pipe if it exists.
        std::fs::remove_file(&pipe_path).expect("Failed to remove existing pipe");
    }

    UnixListener::bind(&pipe_path).expect("Failed to create named pipe")
}

fn send_args_to_existing_instance(pipe_path: PathBuf, args_paths: Vec<PathBuf>) {
    let mut stream = UnixStream::connect(pipe_path).expect("Failed to connect to named pipe");
    let data = bincode::serialize(&args_paths).expect("Failed to serialize arguments");

    // Send the arguments to the first instance.
    stream.write_all(&data).expect("Failed to send arguments");
}

pub fn init(
    lock_file: &File,
    args_paths: Vec<PathBuf>,
    one_instance: bool,
) -> Option<Receiver<Vec<PathBuf>>> {
    if !one_instance {
        return None;
    }

    let pipe_name = "3dobs_pipe";

    let pipe_path = std::env::temp_dir().join(pipe_name);

    match lock_file.try_lock_exclusive() {
        Ok(_) => {}
        Err(_) => {
            println!("An instance of the program is already running");
            send_args_to_existing_instance(pipe_path, args_paths);
            exit(0);
        }
    }

    let pipe = create_named_pipe(pipe_path);
    let (ipc_tx, ipc_rx) = std::sync::mpsc::channel::<Vec<PathBuf>>();

    // thread is not joined because it blocks anyway
    // and there's no cleanup to do or anything
    let _ = thread::spawn(move || {
        for stream in pipe.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut serialized_paths = Vec::new();
                    let _ = stream.read_to_end(&mut serialized_paths);

                    let paths: Vec<PathBuf> = bincode::deserialize(&serialized_paths).unwrap();

                    ipc_tx.send(paths).unwrap();
                }
                Err(err) => {
                    error!("Error: {:?}", err);
                    break;
                }
            }
        }
    });

    Some(ipc_rx)
}
