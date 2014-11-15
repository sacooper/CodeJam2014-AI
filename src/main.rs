extern crate image;
extern crate rgsl;
use image::GenericImage;
use std::os;

struct Class {
    U : rgsl::MatrixF64,
    phi : rgsl::VectorF64
}

fn main() {
    if os::args().len() < 2 {
        std::io::stderr().write_str(format!("ERROR:\nUsage: {} inputFile.gif\n", os::args()[0]).as_slice());
        return;
    }

    let mut a : [f64, ..25] =
        [2f64, 0f64, 8f64, 6f64, 0f64,
        1f64, 6f64, 0f64, 1f64, 7f64,
        5f64, 0f64, 7f64, 4f64, 0f64,
        7f64, 0f64, 8f64, 5f64, 0f64,
        0f64, 10f64, 0f64, 0f64, 7f64];

    let mut m = rgsl::MatrixView::from_array(a, 5, 5);
    let mut v = rgsl::MatrixF64::new(5, 5).unwrap();
    let mut s = rgsl::VectorF64::new(5).unwrap();
    let mut w = rgsl::VectorF64::new(5).unwrap();
    rgsl::linear_algebra::SV_decomp(&m.matrix(), &v, &s, &w);

    println!("U:\n{}", m.matrix());
    println!("V:\n{}", v);
    println!("S:\n{}", s);

    let imageFile = &os::args()[1];
    // let image = image::open(&Path::new(imageFile)).unwrap();
    // println!("Hello, world!\n{}", imageFile);

    let class_strings = create_classes(
        std::io::fs::readdir(
            &Path::new(imageFile))
            .unwrap().iter().map(
                |x|{x.as_str().unwrap().to_string()}).collect());

    for x in class_strings.iter(){
        spawn(proc(){
            // 1) load files (each load in seperate task?)
            // 1b) MAYBE preprocess
            // 2) compute average face
            // 3) compute A = set of deviations from average face
            // 4) use SVD to find U
            // 5) create class and communicate it back
        })
    }

    // NORM: dnrm2(VectorF64)
}

fn create_classes(list : Vec<String>) -> Vec<Vec<String>> {
    let mut l = list.clone();
    let mut list2 = l.as_mut_slice();
    list2.sort();
    let mut result = Vec::new();
    let mut temp = Vec::new();
    let mut current = -1i8;
    for x in list2.iter() {
        // println!("{}", list2[0].as_slice().split('/').last().unwrap().split('_').next().unwrap());
        let n = from_str::<i8>(x.split('/').last().unwrap().split('_').next().unwrap());

        current = match n {
            Some(n) => {
                if n==current {
                    temp.push(x.clone());
                    current
                } else if current == -1i8 {
                    temp.push(x.clone());
                    n
                } else {
                    println!("{}", temp);
                    result.push(temp.clone());
                    temp.clear();
                    temp.push(x.clone());
                    n}},
            None => current}};
    result
}
