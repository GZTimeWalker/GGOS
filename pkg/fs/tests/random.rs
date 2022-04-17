use ggfs::device::*;

#[cfg(test)]
#[test]
fn test_random() {

    println!("random num: 0x{:x}", Random::rand_u64());

    let mut buf = [0u8; 64];
    Random::new().read(&mut buf, 0, 64).unwrap();

    for i in 0..8 {
        for j in 0..8 {
            print!("0x{:02x}, ", buf[i * 8 + j]);
        }
        println!();
    }
}
