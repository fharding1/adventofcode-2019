fn main() {
    let lo = 138307;
    let hi = 654504;

    let mut ok = 0;

    'bruteforce: for i in lo..hi {
        let s: Vec<char> = i.to_string().chars().collect();

        let mut found_seq = false;
        let mut seq_count = 0;

        let mut i = 0;
        while i < s.len() - 1 {
            let cur = s[i];
            let next = s[i + 1];

            if next == cur {
                seq_count += 1;
            } else {
                if seq_count == 1 {
                    found_seq = true;
                }

                seq_count = 0;
            }

            if next < cur {
                continue 'bruteforce;
            }

            i += 1;
        }

        if !found_seq && seq_count != 1 {
            continue;
        }

        ok += 1;
    }

    println!("{}", ok);
}
