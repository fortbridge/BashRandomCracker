extern crate chrono;
use std::thread;

use clap::Parser;
use crossbeam_channel::unbounded;
use chrono::{DateTime, NaiveDateTime, Utc};

use bashrand::{
    cli::{Args, SubCommands, Version},
    log,
    random::{Random, BASH_RAND_MAX},
    CollisionCracker, MultiResultCracker, MultiResultVersionCracker, New1Cracker, New2Cracker,
    New3Cracker, Old1Cracker, Old2Cracker, Old3Cracker, OneResultCracker, Result,
};

fn main() {
    let args = Args::parse();

    if let Err(e) = do_main(args) {
        log::error(e);
    }
}

fn print_seed_and_clone(seed: u32, skip: usize, is_old: bool, number: usize) {
    println!(
        "Seed: {seed}{} ({})",
        match skip {
            0 => String::from(""),
            _ => format!(" +{skip}"),
        },
        if is_old { "old" } else { "new" }
    );

    let mut rng = Random::new(seed, is_old);
    rng.skip(skip);

    match number {
        0 => (),
        1 => println!("  Next value: {}", rng.next_16()),
        _ => println!("  Next {} values: {:?}", number, rng.next_16_n(number)),
    }
}

fn do_main(args: Args) -> Result<()> {
    let (version_old, version_new) = match args.version {
        Version::Old => (true, false),
        Version::New => (false, true),
        Version::Both => (true, true),
    };


    match args.command {
        SubCommands::Crack { numbers } => {
            if numbers.iter().any(|n| *n > BASH_RAND_MAX) {
                return Err(
                    format!("Numbers must be at most 15 bits (max: {})", BASH_RAND_MAX).into(),
                );
            };

            match numbers.len() {
                // Certain (one possible seed)
                3 => {
                    let numbers = [numbers[0], numbers[1], numbers[2]];

                    log::progress("Searching for seeds...".to_string());

                    let (mut seed, mut is_old) = (None, false);

                    if version_new {
                        let cracker = New3Cracker::new(numbers);
                        seed = cracker.find();
                    }
                    if version_old && seed.is_none() {
                        let cracker = Old3Cracker::new(numbers);
                        seed = cracker.find();
                        is_old = true;
                    }

                    print_seed_and_clone(seed.ok_or("Couldn't find seed")?, 3, is_old, args.number);

                    log::success("Finished!");
                }
                // Uncertain (multiple possible seeds)
                2 => {
                    let numbers = [numbers[0], numbers[1]];

                    let (tx, rx) = unbounded();

                    log::progress("Searching for seeds...".to_string());

                    thread::spawn(move || {
                        if version_new {
                            let cracker = New2Cracker::new(numbers);
                            cracker.find(&tx);
                        }
                        if version_old {
                            let cracker = Old2Cracker::new(numbers);
                            cracker.find(&tx);
                        }
                    });

                    // Stream all found seeds
                    let mut count = 0;
                    for (seed, old) in rx {
                        print_seed_and_clone(seed, 2, old, args.number);
                        count += 1;
                    }

                    if count == 0 {
                        return Err("Couldn't find seed".into());
                    } else {
                        log::success(format!("Finished! ({count} seeds)"));
                    }
                }
                _ => unreachable!(),
            }
        }
        SubCommands::Get { seed, skip } => {
            if version_new {
                print_seed_and_clone(seed, skip, false, args.number);
            }
            if version_old {
                print_seed_and_clone(seed, skip, true, args.number);
            }
        }
        SubCommands::Seeds { seed } => {
            // Seed generation is the same for both versions
            let mut rng = Random::new(seed, false);
            let seeds = rng.next_seed_n(args.number);
            println!("Next {} seeds: {:?}", args.number, seeds);
        }
        SubCommands::Collide { n } => {
            let (tx, rx) = unbounded();

            log::progress("Searching for seeds...".to_string());

            thread::spawn(move || match (version_new, version_old) {
                (true, true) => {
                    let cracker = CollisionCracker::new(n);
                    cracker.find(&tx);
                }
                (true, false) => {
                    let cracker = New1Cracker::new(n);
                    cracker.find(&tx);
                }
                (false, true) => {
                    let cracker = Old1Cracker::new(n);
                    cracker.find(&tx);
                }
                (_, _) => unreachable!("No version selected"),
            });

            // Stream all found seeds
            let mut count = 0;
            for seed in rx {
                println!("Seed: {seed} = {n}");
                count += 1;
            }

            if count == 0 {
                return Err("Couldn't find seed".into());
            } else {
                log::success(format!("Finished! ({count} seeds)"));
            }
        }

        SubCommands::Password { password } => {
            println!("Received password: {}, len= {}", password, password.len());
            let matrix = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
            let mut is_match ;
            let mut seed = 0u64;
            let now = Utc::now();  // Get the current UTC time

            // Convert to a Unix timestamp in seconds
            let current = now.timestamp() as u64;
           
            
            while seed < current  {
                let mut _rng = Random::new(seed.try_into().unwrap(), true);
                is_match = true;
                
                for (i, char_expected) in password.chars().enumerate() {
                    let n = _rng.next_16();
                    let c = matrix.chars().nth(n as usize % matrix.len()).unwrap();
        
                    if c != char_expected {
                        is_match = false;
                        break;
                    }
                    if i == (password.len() - 1) && is_match {
                        println!("Matching seed found: {}", seed);
                        let naive_datetime = NaiveDateTime::from_timestamp(seed as i64, 0);
                        let datetime: DateTime<Utc> = DateTime::from_utc(naive_datetime, Utc);
                        let formatted_date = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
                        println!("Formatted seed date and time: {}", formatted_date);
                    }
                }
        
                seed += 1;
            }
           
        },

        SubCommands::GenPass { numbers } => {
            // Check if any number exceeds the maximum allowed for BASH_RAND_MAX
            if numbers.iter().any(|n| *n > BASH_RAND_MAX) {
                return Err(format!("Numbers must be at most 15 bits (max: {})", BASH_RAND_MAX).into());
            }
        
            let matrix = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
        
            // Iterate over the provided numbers to generate the password
            for &i in numbers.iter() {
                let c = matrix.chars().nth((i as usize) % matrix.len()).unwrap(); // Safely get the character
                print!("{}", c);  // Print the character as part of the password generation
            }
            
            println!("\n");
            //Ok(())  // Return Ok if everything goes well
        }
        

    }
    Ok(())
}
