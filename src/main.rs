extern crate image;
use image::GenericImage;
use std::os;

fn main() {
    if os::args().len() < 2 {
        std::io::stderr().write_str(format!("ERROR:\nUsage: {} inputFile.gif\n", os::args()[0]).as_slice());
        return;
    }

    
    let imageFile = &os::args()[1];
    let image = image::open(&Path::new(imageFile)).unwrap();
    println!("Hello, world!\n{}", imageFile);
}
