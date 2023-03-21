use convert::sized_mask;
use std::io::Write;

mod convert;
mod options;

fn main() {
    let options = options::get_options();

    let common_sized_mask = sized_mask(&options);
    std::io::stdout()
        .write_all(&common_sized_mask)
        .expect("Could not write result");
}
