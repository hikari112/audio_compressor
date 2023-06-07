fn main() {
    println!("Hello, world!");
    let bytes = include_bytes!("test.wav");
    println!("{:?}", bytes);
}


