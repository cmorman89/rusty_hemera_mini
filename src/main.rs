use std::thread;
use std::time;

fn main() {
    let v_a: Vec<u8> = vec![0,1,0,1,1,0,1,1,1,0,1,1,1,1,0,1,1,1,1,1];
    let v_b: Vec<u8> = vec![0,0,1,0,0,1,1,0,0,1,1,1,0,0,1,1,1,1,0,0];
    let v_c: Vec<u8> = vec![0,0,0,1,0,0,0,1,1,0,0,0,1,1,1,0,0,0,1,1];
    let v_d: Vec<u8> = vec![0,0,0,0,1,0,0,0,0,1,1,0,0,0,0,1,1,1,0,0];
    let v_e: Vec<u8> = vec![0,0,0,0,0,1,0,0,0,0,1,1,0,0,0,0,0,1,1,1];
    let v_f: Vec<u8> = vec![0,0,0,0,0,0,1,0,0,0,0,1,1,0,0,0,0,0,1,1];
    let v_g: Vec<u8> = vec![0,0,0,0,0,0,0,1,0,0,0,0,1,1,0,0,0,0,0,1];
    let v_h: Vec<u8> = vec![0,0,0,0,0,0,0,0,1,0,0,0,0,1,1,0,0,0,0,0];
    let v_i: Vec<u8> = vec![0,0,0,0,0,0,0,0,0,1,0,0,0,0,1,1,0,0,0,0];
    let v_j: Vec<u8> = vec![0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,1,1,0,0,0];
    let v_k: Vec<u8> = vec![0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,1,1,0,0];
    let v_l: Vec<u8> = vec![0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,1,1,0];
    let v_m: Vec<u8> = vec![0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,1,1];
    let v_n: Vec<u8> = vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,1];
    let v_o: Vec<u8> = vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,1];
    let v_p: Vec<u8> = vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,1];
    let v_q: Vec<u8> = vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,1];
    let v_r: Vec<u8> = vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1];
    let v_s: Vec<u8> = vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1];

    let mut v : Vec<Vec<u8>> = Vec::new();
    v.push(v_a);
    v.push(v_b);
    v.push(v_c);
    v.push(v_d);
    v.push(v_e);
    v.push(v_f);
    v.push(v_g);
    v.push(v_h);
    v.push(v_i);
    v.push(v_j);
    v.push(v_k);
    v.push(v_l);
    v.push(v_m);
    v.push(v_n);
    v.push(v_o);
    v.push(v_p);
    v.push(v_q);
    v.push(v_r);
    v.push(v_s);


    let mut fg_color: u8 = 0;
    let mut bg_color: u8 = 0;
    let fg_ansi_str = String::from("\x1b[38;5;");
    let bg_ansi_str = String::from("\x1b[48;5;");
    let ansi_reset = String::from("\x1b[0m");
    let ansi_to_origin = String::from("\x1b[H");
    let ansi_clear = String::from("\x1b[2J");

    let mut i : u8 = 0;
    let mut j : u8 = 6;

    print!("{}", ansi_clear);
    loop {
        print!("{}", ansi_to_origin);

        for row in v.iter() {
            print!("{}{}m", fg_ansi_str, fg_color);
            print!("{}{}m", bg_ansi_str, bg_color);

            print_row(row);

            println!("{}", ansi_reset);

            fg_color = fg_color.wrapping_add(1);
            bg_color = bg_color.wrapping_add(1);
        }
        i = i.wrapping_add(1);
        j = j.wrapping_add(1);
        // i = i % 255;
        // j = j % 255;
        fg_color = i;
        bg_color = j;
        thread::sleep(time::Duration::from_millis(1));
    }
}

fn mirror_row(row: &Vec<u8>) -> Vec<u8> {
    let mut reversed_row = row.clone();
    reversed_row.reverse();
    reversed_row
}

fn print_row(row: &Vec<u8>) {
    let reversed_row: Vec<u8> = mirror_row(row);
    let joined_row: Vec<u8> = row.iter().chain(reversed_row.iter()).copied().collect();
    let joined_row = joined_row.iter().chain(joined_row.iter());
    for pixel in joined_row {
        if *pixel == 0 {
            print!(" ");
        } else {
            print!("█");
        }
    }
}