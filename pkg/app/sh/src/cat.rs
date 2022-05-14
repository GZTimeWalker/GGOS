use lib::*;

pub fn cat(data: &[u8]) {
    let mut count = 0;
    for (idx, b) in data.iter().enumerate() {
        if count == 0 {
            print!("    ");
        }
        print!("{:02x}", b);
        count += 1;
        if count % 8 == 0 {
            print!(" ");
        }
        if count == 24 {
            print!(" | ");
            for i in idx - 23..=idx {
                let d = data[i];
                if (d as char).is_ascii_graphic() || d == 0x20 {
                    print!("{}", d as char);
                } else {
                    print!(".");
                }
            }
            println!();
            count = 0;
        }
    }
    if count > 0 {
        for _ in count..24 {
            print!("  ");
        }
        for _ in 0..3 - (count / 8) {
            print!(" ");
        }
        print!(" | ");
        for i in data.len() - count..data.len() {
            let d = data[i];
            if (d as char).is_ascii_graphic() || d == 0x20 {
                print!("{}", d as char);
            } else {
                print!(".");
            }
        }
        println!();
    }
}
