use core::panic;
use std::env;
#[allow(unused_imports)]
use std::fs::{OpenOptions, File, write};
use std::io::BufReader;
use std::io::prelude::*;
use rand::Rng;
use std::result::Result;
use num_bigint::{BigUint,ToBigUint};

const PRIMES_PATH: &str = "./primes.txt";

struct Key{
    unique: u64,
    shared: u64,
}
fn main() {
    let mut primes = read_primes();
    let args: Vec<String> = env::args().collect();

    let (public, private) = if args.len() > 1 &&args[1].eq("g") {
        generate_keys(&mut primes, 10000, 20000)
    }
    else{
        let perma_public = Key{
            unique:653585,
            shared:8737109,
        };
    
        let perma_private = Key{
            unique: 4645265,
            shared: 8737109
        };
        (perma_public,perma_private)
    };


    println!("keys = {{'public': ({},{}), 'private': ({},{})}}",public.unique,public.shared,private.unique,private.shared);

    let secret = char_to_cypher('😄', &public);
    let not_so_secret = cypher_to_char(&secret, &private);

    println!("Encoded: {:x}",secret);
    println!("Decoded: {}",not_so_secret);


    let big_secret = string_to_cypher(&"💩 poop cat 😹".to_string(), &private);
    let big_leak = cypher_to_string(&big_secret, &public);

    println!("Encoded big: {:?}",big_secret);
    println!("Decoded big: {}",big_leak);

    write_primes(&primes);
}

fn string_to_cypher(text: &String, key: &Key) -> Vec<BigUint>{
    let mut ret: Vec<BigUint> = Vec::new();
    for cha in text.chars(){
        ret.push(char_to_cypher(cha, &key))
    }
    return ret;
}

fn cypher_to_string(cypher: &Vec<BigUint>, key: &Key) -> String{
    let mut ret = "".to_string();
    for num in cypher {
        ret.push(cypher_to_char(&num, &key));
    }

    return ret;
}

fn char_to_cypher(plain: char, key: &Key) -> BigUint{
    let code = plain as u32;
    let code = code.to_biguint().unwrap();
    let unique = key.unique.to_biguint().unwrap();
    let shared = key.shared.to_biguint().unwrap();

    let cypher: BigUint = code.modpow(&unique,&shared);
    return cypher;
}

fn cypher_to_char(cypher: &BigUint, key: &Key) -> char{
    let unique = key.unique.to_biguint().unwrap();
    let shared = key.shared.to_biguint().unwrap();
    let decode: BigUint = cypher.modpow(&unique,&shared);

    let decode = u32::try_from(decode).expect("Error in decode, number too big");
    let decoded_char: char = char::from_u32(decode).expect("Error in decode, invalid character code");
    return decoded_char;
}

fn generate_keys(primes :&mut Vec<i32>, prime_min: i32, prime_max: i32) -> (Key,Key){
    let (p, q) = get_two_primes(prime_min, prime_max, primes).expect("ERROR: ");

    let n: i64 = (p*q) as i64;
    let totient: i64 = ((p-1) * (q-1)) as i64;
    let mut public = get_public(n, &primes);
    let mut private = get_private_from_public(public, totient); 

    while private == -1 {
        public = get_public(n, &primes);
        private = get_private_from_public(public, totient); 
    }


    return (
        Key{ unique: public as u64, shared: n as u64},
        Key{ unique: private as u64, shared: n as u64}
    )
}

fn get_private_from_public(public: i64, totient: i64) -> i64 {
    // ( public * private ) % totient == 1
    // so ( public * private ) = (totient * x) + 1
    // so that both sides, when divided by totient hve a remainder of 1
    // private = ((totient * x) + 1) / public
    for x in 2..totient {
        if ((totient * x) + 1) / public != public 
        && (((totient as f64 * x as f64) + 1.0) / public as f64) % 1.0 == 0.0 {
            return ((totient * x) + 1) / public;
        }
    }

    return -1;
}

fn get_public(n: i64, primes : &Vec<i32>) -> i64{
    let mut rng = rand::thread_rng();
    let mut exp = rng.gen_range(2..n);
    while  !are_relatively_prime(exp, n, &primes){
        exp = rng.gen_range(2..n);
    }
    return exp;
}

fn get_two_primes(min: i32, max: i32, primes :&mut Vec<i32>) -> Result<(i32,i32),String>{
    let mut rng = rand::thread_rng();
    if min >= max {panic!("Minimum more than maximum");}

    if max > primes[primes.len()-1] {generate_primes(primes, max);}

    let mut min_index: i32 = -1;
    let mut max_index: i32 = -1;

    for (pos, elem) in primes.iter().enumerate() {
        if elem >= &min && min_index == -1 { 
            min_index = pos as i32;   
        }
        if elem >= &max && max_index == -1 {
            max_index = pos as i32; 
        }
        if min_index != -1 && max_index != -1 {
            break;
        }
    }
    if (primes.len() as i32) - min_index < 2 {
        panic!("min and max are too close")
    }
    
    let a = rng.gen_range(min_index..=(max_index));
    let mut b = rng.gen_range(min_index..=(max_index));

    while a == b {
        b = rng.gen_range(min_index..=(max_index));
    }

    return Ok((primes[a as usize],primes[b as usize]));
}

fn write_primes(primes: &Vec<i32>){
    let mut prime_string = format!("{:?}",&primes);
    prime_string = prime_string[1..prime_string.len()-1].to_string();

    write(PRIMES_PATH, &prime_string).expect("Unable to write file")
}

fn read_primes() -> Vec<i32> {
    let file: File = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(PRIMES_PATH)
        .expect("Error on open and / or creation");
    

    let mut buf_reader = BufReader::new(file);

    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents).expect("file cannot be read");

    return contents.split(", ").map(|x| x.parse::<i32>().expect("File format error")).collect();
}

fn are_relatively_prime(a: i64, b: i64, primes : &Vec<i32>) -> bool {
    for num in primes{
        let num = *num as i64;
        if a % num == 0 && b % num == 0 {return false;}
        else if num > a || num > b {break;}
    }
    return true;
}

fn append_next_prime(primes :&mut Vec<i32>){
    primes.push(get_next_prime(&primes))
}

fn get_next_prime(primes :& Vec<i32>) -> i32{
    match primes.len() {
        0 => return 2,
        1 => return 3,
        2 => return 5,
        _ => return {
            let final_prime = primes[primes.len()-1];
    
            let final_n = if (final_prime-1) % 6 == 0  {(final_prime-1) / 6} 
                        else {(final_prime+1) / 6};

            // for number in between the n such that n*6 +- 1 = the largest prime
            // and the largest i32 L such that L*6 + 1 < the maximum of i32 
            let mut ret = 0;
            for num in (final_n)..= ((i32::MAX-1)/6) {
                let mut a = (num*6) + 1;
                let mut b = (num*6) - 1;
                for p in primes{
                    if a % *p == 0 { a = 0; }
                    if b % *p == 0 { b = 0; }
                    if a == 0 && b == 0 {break;}
                }
                if b != 0 {ret = b; break;}
                if a != 0 {ret = a; break;}
            }
            ret
        }
    } // 2, 3, 5, 7, 11,
    
}

fn generate_primes(primes :&mut Vec<i32>, maximum: i32){
    // first 2 primes are 2 and 3, all others are either 1 more or 1 less
    // than a multiple of 6. this function can use an existing primes list
    // to avoid redoing calculations, but checks to ensure 2 and 3 are in there
    if primes.len() < 3 { 
        primes.clear(); 
        primes.push(2);
        primes.push(3);
        primes.push(5); // for the final_n value to be found correctly
    }

    while primes[primes.len()-1] < maximum {
        append_next_prime(primes);
    }


}