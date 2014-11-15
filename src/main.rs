extern crate rgsl;

use std::io;

struct Class {
    U : rgsl::MatrixF64,
    phi : rgsl::VectorF64
}

fn main() {
    if std::os::args().len() < 2 {
        io::stderr().write_str(format!("ERROR:\nUsage: {} inputFile.gif\n", std::os::args()[0]).as_slice());
    }

    let mut a : [f64, ..25] =
        [2f64, 0f64, 8f64, 6f64, 0f64,
        1f64, 6f64, 0f64, 1f64, 7f64,
        5f64, 0f64, 7f64, 4f64, 0f64,
        7f64, 0f64, 8f64, 5f64, 0f64,
        0f64, 10f64, 0f64, 0f64, 7f64];

    let mut m = rgsl::MatrixView::from_array(a, 5, 5);
    let v = rgsl::MatrixF64::new(5, 5).unwrap();
    let s = rgsl::VectorF64::new(5).unwrap();
    let w = rgsl::VectorF64::new(5).unwrap();
    rgsl::linear_algebra::SV_decomp(&m.matrix(), &v, &s, &w);

    println!("U:\n{}", m.matrix());
    println!("V:\n{}", v);
    println!("S:\n{}", s);

    let image_file = &std::os::args()[1];

    let class_strings = create_classes(
        std::io::fs::readdir(
            &Path::new(image_file))
            .unwrap().iter().map(
                |x|{x.as_str().unwrap().to_string()}).collect());


    let (tx, rx) = channel();

    for x in class_strings.iter(){
        let tx = tx.clone();
        let x = x.clone();
        spawn(proc(){
            // 1) load files (each load in seperate task?)
            // 1b) MAYBE preprocess

            let mut faces : Vec<Vec<u8>> = Vec::new();      // Each vec represents 1 face

            for s in x.iter() {
                let mut file = io::File::open(&Path::new(s)).unwrap();
                let contents = file.read_to_end().unwrap().into_ascii().into_string();
                let as_array = contents.as_slice().split(' ');
                faces.push(as_array.map(|x| {from_str::<u8>(x).unwrap()}).collect())
            }

            let m = faces[0].len() as u64;
            let n = faces.len() as u64;

            // 2) compute average face
            let N = faces.len();
            let mut phi : Vec<u8> = Vec::new();
            let mut cont = true;
            while cont {
                let mut sum = 0u32;
                for z in faces.iter_mut() {
                    match z.pop() {
                        Some(x) => {sum += (x as u32)}
                        None => cont = {false; break }
                    }
                }
                phi.push((sum / (N as u32)) as u8);
            }

            // 3) compute set of deviations from average face
            let mut A : Vec<f64> = Vec::new();
            for c in faces.into_iter(){
                for (i, l) in c.iter().enumerate(){
                    A.push((*l-phi[i]) as f64);
                }
            }

            // 4) use SVD to find U
            let mut u = rgsl::MatrixView::from_array(A.as_mut_slice(), m, n);
            let v = rgsl::MatrixF64::new(5, 5).unwrap();
            let s = rgsl::VectorF64::new(5).unwrap();
            let w = rgsl::VectorF64::new(5).unwrap();
            rgsl::linear_algebra::SV_decomp(&u.matrix(), &v, &s, &w);
            // 5) create class and communicate it back
            let phi = rgsl::VectorF64::from_slice(phi.into_iter().map(|x|{x as f64}).collect::<Vec<f64>>().as_slice()).unwrap();
            tx.send(Class{U: u.matrix(), phi: phi});
            // tx.send(class);
        })
    }

    let mut classes : Vec<Class> = Vec::new();
    for _ in range(0, class_strings.len()){
        classes.push(rx.recv());
    }

    let mut min : Option<f64> = None;
    for c in classes.iter(){
        // calculate norms and
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
        let
        n = from_str::<i8>(x.split('/').last().unwrap().split('_').next().unwrap());

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
