use convert::sized_mask;
use std::io::Write;

mod convert;
mod options;

fn main() {
    // println!("Target: {}", args.color_scheme == ColorSchemeOrAuto::Auto);
    let options = options::get_options();
    // println!("{:?}", options);

    let common_sized_mask = sized_mask(&options);
    std::io::stdout()
        .write_all(&common_sized_mask)
        .expect("Could not write result");
}
