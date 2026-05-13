fn main() {
    if let Err(err) = trash_cli::app::run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
