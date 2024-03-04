use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::{thread};

fn log_char(character: char, log_filename: &str)  {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_filename)
        .expect("Failed to open log file");
    writeln!(file, "{}", character).expect("Failed to write char to log file");
}

fn log_message(message: &str, log_filename: &str) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_filename)
        .expect("Failed to open log file");
    writeln!(file, "{}", message).expect("Failed to write message to log file");
}

fn read_message(input_stream: &mut impl BufRead) -> io::Result<String> {
    let mut header_line = String::new();
    if input_stream.read_line(&mut header_line)? > 0 {
        let content_length: usize = header_line.trim()
            .split(':')
            .nth(1)
            .expect("Content-Length header not found")
            .trim()
            .parse()
            .expect("Failed to parse content length");

        let mut separator = vec![0; 2]; 
        let _ = input_stream.read_exact(&mut separator);
        let separator = String::from_utf8(separator)
            .expect("Failed to read separator");
        header_line.push_str(&separator);

        let mut content = vec![0; content_length];
        let _ = input_stream.read_exact(&mut content)?;
        let content = String::from_utf8(content)
            .expect("Failed to convert content to String");
        header_line.push_str(&content);

        return Ok(header_line);
    }
    Ok(String::new())
}

fn move_input(input_stream: &mut impl BufRead, to_clangd: &mut std::process::ChildStdin){
    let mut header_line = String::new();
    if input_stream.read_line(&mut header_line).expect("la") > 0 {
        let content_length: usize = header_line.trim()
            .split(':')
            .nth(1)
            .expect("Content-Length header not found")
            .trim()
            .parse()
            .expect("Failed to parse content length");

        let mut separator = vec![0; 2]; 
        let _ = input_stream.read_exact(&mut separator);
        let separator = String::from_utf8(separator)
            .expect("Failed to read separator");
        header_line.push_str(&separator);

        let mut content = vec![0; content_length];
        input_stream.read_exact(&mut content).expect("le");
        let content = String::from_utf8(content)
            .expect("Failed to convert content to String");
        header_line.push_str(&content);

        to_clangd.write_all(header_line.as_bytes()).expect("Failed to write");
    }
}

fn move_output(from_clangd : &mut BufReader<std::process::ChildStdout>){
    match read_message(from_clangd) {
        Ok(response) => {
            println!("{}", response);
        },
        Err(e) => eprintln!("Error reading message from clangd: {}", e),
    }
}

fn main() {
    let mut clangd = Command::new("clangd-15")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start clangd");

    let mut to_clangd = clangd.stdin.take().expect("Failed to open stdin to clangd");
    let mut from_clangd = BufReader::new(clangd.stdout.take().expect("Failed to open stdout from clangd"));

    let mut input = BufReader::new(io::stdin());

    let input_thread = thread::spawn(move || {
        loop {
            move_input(&mut input, &mut to_clangd);
        }
    });

    let output_thread = thread::spawn(move || {
        loop {
            move_output(&mut from_clangd);
        }
    });

    input_thread.join().unwrap();
    output_thread.join().unwrap();

    clangd.wait().expect("Failed to wait on clangd");
}

