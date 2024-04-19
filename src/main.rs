use std::env;

mod language_server_extender;
use crate::language_server_extender::LanguageServerExtender;
use crate::language_server_extender::ParsedMessage;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut server = LanguageServerExtender::new(
        //"/home/dechog/.dotfiles/Code/User/globalStorage/llvm-vs-code-extensions.vscode-clangd/install/17.0.3/clangd_17.0.3/bin/clangd",
        "clangd-15",
        &args)
            .expect("Error running server");

        server.add_handler("codeAction", Box::new(|_msg: &ParsedMessage| -> Result<String, std::io::Error> {
            Ok("".to_string())
        }));

        
        server.run();
}

