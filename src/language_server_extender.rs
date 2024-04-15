use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::{Mutex, Arc};
use std::thread;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, Write, Result, Error};
use serde_json::{Value};

type CodeActionMap = Arc<Mutex<HashMap<u32, Option<String>>>>;

fn invalid_data(msg: &str) -> Error
{
    std::io::Error::new(std::io::ErrorKind::InvalidData, msg)
}

fn other_error(msg: &str) -> Error
{
    std::io::Error::new(std::io::ErrorKind::Other, msg)
}

pub struct LanguageServerExtender {
    _language_server: Child,
    _pending_message_extensions: CodeActionMap,

    to_language_server: BufWriter<ChildStdin>,
    from_language_server: BufReader<ChildStdout>,
    to_ide: BufWriter<std::io::Stdout>,
    from_ide: BufReader<std::io::Stdin>,
}

pub struct ParsedMessage {
    full_message: String,
    _method: String,
    _json_content: serde_json::Value,
}

impl LanguageServerExtender {
    pub fn new(language_server_path: &str, args: &[String]) -> std::io::Result<Self>
    {
        let mut language_server = Command::new(language_server_path)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let to_language_server = BufWriter::new(
            language_server.stdin.take()
                .ok_or_else(|| other_error("Failed to open stdin"))?
        );

        let from_language_server = BufReader::new(
            language_server.stdout.take()
                .ok_or_else(|| other_error("Failed to open stdout"))?
        );

        Ok(LanguageServerExtender {
            _language_server: language_server,
            _pending_message_extensions: Arc::new(Mutex::new(HashMap::new())),
            to_language_server,
            from_language_server,
            to_ide: BufWriter::new(std::io::stdout()),
            from_ide: BufReader::new(std::io::stdin()),
        })
    }

    pub fn run(self)
    {
        let mut from_ide           = self.from_ide; 
        let mut to_language_server = self.to_language_server;

        let input_thread = thread::spawn(move || {
            loop {
                if let Err(e) = Self::run_once(&mut from_ide,
                                               &mut to_language_server) {
                    eprintln!("Error in input thread: {}", e);
                    break;
                }
            }
        });

        let mut from_language_server = self.from_language_server;
        let mut to_ide               = self.to_ide;

        let output_thread = thread::spawn(move || {
            loop {
                if let Err(e) = Self::run_once(&mut from_language_server,
                                               &mut to_ide) {
                    eprintln!("Error in output thread: {}", e);
                    break;
                }
            }
        });

        input_thread.join().unwrap();
        output_thread.join().unwrap();

        //ensure the process is dead now.
    }

    //fn run_once(read_stream: &mut impl BufRead, write_stream: &mut BufWriter<std::io::Stdout>) -> std::io::Result<()> {
    //    let response = Self::parse_message(read_stream)?;
    //    write_stream.write_all(response.full_message.as_bytes())?;
    //    write_stream.flush()?;
    //    Ok(())
    //}

    fn run_once<R: BufRead, W: Write>(read_stream: &mut R, write_stream: &mut W) -> std::io::Result<()> {
        let response = Self::parse_message(read_stream)?;
        write_stream.write_all(response.full_message.as_bytes())?;
        write_stream.flush()?;
        Ok(())
    }

    fn _run_input_once(&mut self)
    {
        match Self::parse_message(&mut self.from_ide) {
            Ok(response) => {
                self.to_language_server.write_all(response.full_message.as_bytes())
                    .expect("Fail to write");
            },
            Err(e) => eprintln!("Error reading message from IDE : {}", e),
        }
    }

    fn _run_output_once(&mut self)
    {
        match Self::parse_message(&mut self.from_language_server) {
            Ok(response) => {
                self.to_ide.write_all(response.full_message.as_bytes())
                    .expect("Fail to write");
            },
            Err(e) => eprintln!("Error reading message from language server: {}", e),
        }
    }

    fn parse_message(input_stream: &mut impl BufRead) -> Result<ParsedMessage>
    {
        let mut full_message = String::new();

        if input_stream.read_line(&mut full_message)? <= 0 {
            return Err(other_error("No input read"));
        }

        let content_length: usize = full_message.trim()
            .split(':')
            .nth(1)
            .ok_or_else(|| invalid_data("Content-Length header not found"))?
            .trim()
            .parse()
            .map_err(|_| invalid_data("Failed to parse content length"))?;
    
        let mut separator = vec![0; 2]; 
        let _ = input_stream.read_exact(&mut separator);
        let str_separator = String::from_utf8(separator)
            .map_err(|_| invalid_data("Failed to convert content to String"))?;
        full_message.push_str(&str_separator);
    
        let mut content = vec![0; content_length];
        let _ = input_stream.read_exact(&mut content)?;
        let content = String::from_utf8(content)
            .map_err(|_| invalid_data("JSON parsing failed"))?;
    
        full_message.push_str(&content);
        
        let parsed_content: Value = serde_json::from_str(&content)?;
        
        let method: String = parsed_content.get("method")
            .and_then(Value::as_str)
            .map_or_else(
                ||{"unknown".to_string()},
                |s|{s.to_string()},
            );

        return Ok(ParsedMessage {
            full_message: full_message,
            _method: method,
            _json_content: parsed_content,
        });
    }
}

