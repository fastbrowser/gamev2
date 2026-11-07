use rand::random;

fn main() {
    let a:u8 = random();
    println!("{}",a&0xf);
    println!("{}",a>>4);
}