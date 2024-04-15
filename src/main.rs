use std::env;

mod language_server_extender;
use crate::language_server_extender::LanguageServerExtender;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    // /home/dechog/.dotfiles/Code/User/globalStorage/llvm-vs-code-extensions.vscode-clangd/install/17.0.3/clangd_17.0.3/bin/clangd
    //let mut clangd = Command::new("clangd-15")
    let server = LanguageServerExtender::new(
        "/home/dechog/.dotfiles/Code/User/globalStorage/llvm-vs-code-extensions.vscode-clangd/install/17.0.3/clangd_17.0.3/bin/clangd",
        &args)
            .expect("Error running server");
        
        server.run();
    }

