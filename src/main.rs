extern crate rgsl;

use std::io;
use std::io::process::Command;
use std::io::process;

struct Class {
    U : rgsl::MatrixF64,
    phi : rgsl::VectorF64
}

fn main() {
    if std::os::args().len() < 2 {
        io::stderr().write_str(format!("ERROR:\nUsage: {} inputFile.gif\n", std::os::args()[0]).as_slice());
    }

    let mut prog = Command::new("python");
    prog.args(["database.py", "img"]);
    prog.stdin(process::InheritFd(0));
    prog.stdout(process::InheritFd(1));
    prog.stderr(process::InheritFd(2));
    prog.spawn();

    // let mut a : [f64, ..25] =
    //     [2f64, 0f64, 8f64, 6f64, 0f64,
    //     1f64, 6f64, 0f64, 1f64, 7f64,
    //     5f64, 0f64, 7f64, 4f64, 0f64,
    //     7f64, 0f64, 8f64, 5f64, 0f64,
    //     0f64, 10f64, 0f64, 0f64, 7f64];
    //
    // let mut m = rgsl::MatrixView::from_array(a, 5, 5);
    // let v = rgsl::MatrixF64::new(5, 5).unwrap();
    // let s = rgsl::VectorF64::new(5).unwrap();
    // let w = rgsl::VectorF64::new(5).unwrap();
    // rgsl::linear_algebra::SV_decomp(&m.matrix(), &v, &s, &w);
    //
    // println!("U:\n{}", m.matrix());
    // println!("V:\n{}", v);
    // println!("S:\n{}", s);

    let image_file = &std::os::args()[1];

    let class_strings = create_classes(
        std::io::fs::readdir(
            &Path::new("img"))
            .unwrap().iter().filter_map(|x|{
                let s = x.as_str().unwrap();
                if s.contains(".csv") {Some(s.to_string())} else {None}
                }).collect());
            // .unwrap().iter().map(
            //     |x|{x.as_str().unwrap().to_string()}).collect());


    let (tx, rx) = channel();

    for x in class_strings.clone().iter(){
        let tx = tx.clone();
        let x = x.clone();
        spawn(proc(){
            let mut faces : Vec<Vec<u8>> = Vec::new();      // Each vec represents 1 face

            for s in x.iter() {
                // println!("{}", s);
                let mut file = io::File::open(&Path::new(s)).unwrap();
                let contents = file.read_to_end().unwrap().into_ascii().into_string();
                let contents = contents.trim();
                let as_array = contents.as_slice().split(' ');
                // let _ = slice_or_fail(&1000u, &6000u)
                let a : Vec<u8> = as_array.map(|x| {from_str::<u8>(x).unwrap_or(255u8)}).collect();
                // let mut b = Vec::new();
                // b.push_all(a.as_slice().slice_or_fail(&1000u, &71000u));
                // b.push_all(a.as_slice());
                faces.push(a);
                io::fs::unlink(&Path::new(s));
                println!("removed {}", s)
            }

            println!("Read files");

            let n = faces[0].len() as u64;
            let m = faces.len() as u64;

            // 2) compute average face
            let N = faces.len();
            let mut phi : Vec<u8> = Vec::new();

            for i in range(0, n as uint){
                let mut sum = 0u32;
                for j in range(0, m as uint){
                    sum += faces[j][i] as u32;
                }
                phi.push((i / m as uint) as u8);
            }

            println!("Computed average");

            // 3) compute set of deviations from average face
            let mut AT : Vec<f64> = Vec::new();
            for c in faces.clone().into_iter(){
                for (i, l) in c.clone().iter().enumerate(){
                    AT.push((*l-phi[i]) as f64)
                }
            }

            let mut A : Vec<f64> = Vec::new();

            for i in range(0, n as uint){
                for j in range(0, m as uint){
                    A.push((faces[j][i] - phi[i]) as f64)
                }
            }

            println!("Computed deviations: m={}, n={}", m, n);
            // 4) use SVD to find U
            let mut A = box rgsl::MatrixView::from_array(A.as_mut_slice(), n, m).matrix();
            let mut AT = box rgsl::MatrixView::from_array(AT.as_mut_slice(), m, n).matrix();
            let mut u = rgsl::MatrixF64::new(m, m).unwrap();
            rgsl::blas::level3::dgemm(rgsl::cblas::NoTrans,rgsl::cblas::NoTrans, 1.0f64, &*AT, &*A, 0f64, &mut u);
            let v = rgsl::MatrixF64::new(m, m).unwrap();
            let s = rgsl::VectorF64::new(m).unwrap();
            let w = rgsl::VectorF64::new(m).unwrap();
            rgsl::linear_algebra::SV_decomp(&u, &v, &s, &w);

            println!("Computed SVD");

            // 5) create class and communicate it back
            let phi = rgsl::VectorF64::from_slice(phi.into_iter().map(|x|{x as f64}).collect::<Vec<f64>>().as_slice()).unwrap();
            tx.send(Class{U: u, phi: phi});
            // return
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
    let mut current = -1i32;
    for x in list2.iter() {
        // println!("{}", list2[0].as_slice().split('/').last().unwrap().split('_').next().unwrap());
        let n = from_str::<i32>(x.split('/').last().unwrap().split('_').next().unwrap());
        println!("{}", n);
        current = match n {
            Some(n) => {
                if n==current {
                    temp.push(x.clone());
                    current
                } else if current == -1i32 {
                    temp.push(x.clone());
                    n
                } else {
                    println!("{}", temp);
                    result.push(temp.clone());
                    temp.clear();
                    temp.push(x.clone());
                    n}},
            None => current}};
    result.push(temp.clone());
    result
}
