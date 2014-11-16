extern crate rgsl;

use std::io;
use std::io::process::Command;
use std::io::process;

struct Class {
    U : rgsl::MatrixF64,
    psi : rgsl::VectorF64,
    id : i32
}

fn main() {
    if std::os::args().len() < 2 {
        io::stderr().write_str(format!("ERROR:\nUsage: {} inputFile.gif\n", std::os::args()[0]).as_slice());
    }

    let image_file = &std::os::args()[1];
    let img = image_file.clone();
    let (txa, rxa) = channel();

    spawn(proc(){
        let mut prog = Command::new("python");
        prog.args(["database.py", img.as_slice()]);
        prog.stdin(process::InheritFd(0));
        prog.stdout(process::InheritFd(1));
        prog.stderr(process::InheritFd(2));
        let _ = prog.spawn();

        let mut file = io::File::open(&Path::new(img.replace(".gif", ".csv"))).unwrap();
        let contents = file.read_to_end().unwrap().into_ascii().into_string();
        let _ = io::fs::unlink(&Path::new(img.replace(".gif", ".csv")));
        let contents = contents.trim();
        let as_array = contents.as_slice().split(' ');
        let a : Vec<f64> = as_array.map(|x| {from_str::<f64>(x).unwrap_or(0f64)}).collect();
        txa.send(rgsl::VectorF64::from_slice(a.as_slice()).unwrap())
    });

    let mut prog = Command::new("python");
    prog.args(["database.py", "img"]);
    prog.stdin(process::InheritFd(0));
    prog.stdout(process::InheritFd(1));
    prog.stderr(process::InheritFd(2));
    let _ = prog.spawn();

    let class_strings = create_classes(
        std::io::fs::readdir(
            &Path::new("img"))
            .unwrap().iter().filter_map(|x|{
                let s = x.as_str().unwrap();
                if s.contains(".csv") {Some(s.to_string())} else {None}
                }).collect());
            // .unwrap().iter().map(
            //     |x|{x.as_str().unwrap().to_string()}).collect());


    // let (m, n) = (8u64, 70000u64);
    let (tx, rx) = channel();
    for class in class_strings.clone().iter(){
        let (x, id) : (Vec<String>, i32) = class.clone();
        let tx = tx.clone();
        spawn(proc(){
            let mut faces : Vec<Vec<u8>> = Vec::new();      // Each vec represents 1 face

            for s in x.iter() {
                let mut file = io::File::open(&Path::new(s)).unwrap();
                let contents = file.read_to_end().unwrap().into_ascii().into_string();
                let contents = contents.trim();
                let as_array = contents.as_slice().split(' ');
                // let _ = slice_or_fail(&1000u, &6000u)
                let a : Vec<u8> = as_array.map(|x| {from_str::<u8>(x).unwrap()}).collect();
                // let mut b = Vec::new();
                // b.push_all(a.as_slice().slice_or_fail(&1000u, &71000u));
                // b.push_all(a.as_slice());
                faces.push(a);
                let _ = io::fs::unlink(&Path::new(s));
            }

            let n = faces[0].len() as u64;
            let m = faces.len() as u64;

            // 2) compute average face
            let mut psi : Vec<u8> = Vec::new();

            for i in range(0, n as uint){
                let mut sum = 0u64;
                for j in range(0, m as uint){
                    if i < faces[j].len(){sum += faces[j][i] as u64}
                }
                psi.push((sum / m as u64) as u8);
            }

            // 3) compute set of deviations from average face
            // let mut AT : Vec<f64> = Vec::new();
            // for c in faces.clone().into_iter(){
            //     for (i, l) in c.clone().iter().enumerate(){
            //         if i < psi.len() {AT.push((*l-psi[i]) as f64)}
            //     }
            // }

            let mut A : Vec<f64> = Vec::new();

            for i in range(0, n as uint){
                for j in range(0, m as uint){
                    if i < psi.len() {A.push((faces[j][i] - psi[i]) as f64)}
                }
            }

            // 4) use SVD to find U
            let A = box rgsl::MatrixView::from_array(A.as_mut_slice(), n, m).matrix();
            let mut u = rgsl::MatrixF64::new(m, m).unwrap();
            rgsl::blas::level3::dgemm(rgsl::cblas::Trans,rgsl::cblas::NoTrans, 1.0f64, &*A, &*A, 0f64, &mut u);
            let v = rgsl::MatrixF64::new(m, m).unwrap();
            let s = rgsl::VectorF64::new(m).unwrap();
            let w = rgsl::VectorF64::new(m).unwrap();
            rgsl::linear_algebra::SV_decomp(&u, &v, &s, &w);

            let mut T = box rgsl::MatrixF64::new(n, m).unwrap();
            rgsl::blas::level3::dgemm(rgsl::cblas::NoTrans, rgsl::cblas::NoTrans, 1f64, &*A, &u, 0f64, &mut *T);
            // 5) create class and communicate it back
            let psi = rgsl::VectorF64::from_slice(psi.into_iter().map(|x|{x as f64}).collect::<Vec<f64>>().as_slice()).unwrap();
            tx.send(Class{U: *T, psi: psi, id:id});
            // return
        })}

    let mut classes : Vec<Class> = Vec::new();
    for _ in range(0, class_strings.len()){
        classes.push(rx.recv());
    }

    let mut min : Option<f64> = None;
    let mut min_id : Option<i32> = None;

    let gamma_orig : rgsl::VectorF64 = rxa.recv();

    for c in classes.iter(){
        let gamma = gamma_orig.clone().unwrap();
        // println!("{}, {}, {}, {}", c.id, gamma.len(), c.U.size1(), c.U.size2());
        // assert!(gamma.len() == c.U.size2());
        gamma.sub(&c.psi);
        let mut y = rgsl::VectorF64::new(c.U.size2()).unwrap();
        rgsl::blas::level2::dgemv(rgsl::cblas::Trans, 1f64, &c.U, &gamma, 0f64, &mut y);

        let mut omega = rgsl::VectorF64::new(c.U.size2()).unwrap();
        rgsl::blas::level2::dgemv(rgsl::cblas::Trans, 1f64, &c.U, &c.psi, 0f64, &mut omega);

        y.sub(&omega);

        let ep = rgsl::blas::level1::dnrm2(&y);
        println!("{} {} {}", c.id, rgsl::blas::level1::dnrm2(&omega), ep);

        match min {
            Some(x)=>{if ep < x {min = Some(ep); min_id = Some(c.id)}}
            None=>{min = Some(ep); min_id = Some(c.id)}
        }
        // calculate norms and
    }

    println!("{}", min_id.unwrap());
}

fn create_classes(list : Vec<String>) -> Vec<(Vec<String>, i32)> {
    let mut l = list.clone();
    let mut list2 = l.as_mut_slice();
    list2.sort();
    let mut result = Vec::new();
    let mut temp = Vec::new();
    let mut current = -1i32;
    for x in list2.iter() {
        let n = from_str::<i32>(x.split('/').last().unwrap().split('_').next().unwrap());
        current = match n {
            Some(n) => {
                if n==current {
                    temp.push(x.clone());
                    current
                } else if current == -1i32 {
                    temp.push(x.clone());
                    n
                } else {
                    result.push((temp.clone(), current));
                    temp.clear();
                    temp.push(x.clone());
                    n}},
            None => current}};
    result.push((temp.clone(), current));
    result
}
