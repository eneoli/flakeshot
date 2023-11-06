use flakeshot::Error;

fn run() ->Result<(), Error> {
    Err(Error::NotImplemented)
}

fn main() {

    let result = run();

    if let Err(err) = result {
        match err {
        }
    }
}
