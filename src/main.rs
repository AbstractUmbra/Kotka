use kotka::bif::BIF;

fn main() {
    let bif = BIF::new(Some("example_files/Game/".to_owned()), None, &mut None).unwrap();

    println!("{:#?}", bif);
}
