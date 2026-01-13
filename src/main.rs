use rust_pipe::run;

fn main() {
    if let Err(e) = run() {
        e.termination();
    }
}
