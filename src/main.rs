#![allow(dead_code)]
#![allow(unused_imports)]

mod book;
mod info;
mod term;

use book::{*};
use info::{*};
use term::{*};

use TSPL::Parser;
use highlight_error::highlight_error;
use std::env;
use std::fs::File;
use std::io::Write;
use std::process::Command;

TSPL::new_parser!(KindParser);

fn generate_check_hvm1(book: &Book, command: &str, arg: &str) -> String {
  //let used_defs = book.defs.keys().collect::<Vec<_>>().iter().map(|name| format!("(Pair \"{}\" Book.{})", name, name)).collect::<Vec<_>>().join(" ");
  let kind_hvm1 = include_str!("./kind2.hvm1");
  let book_hvm1 = book.to_hvm1();
  let main_hvm1 = match command {
    "check" => format!("Main = (API.check Book.{})\n", arg),
    //"check" => format!("Main = (API.check.many [{}])\n", used_defs),
    "run"   => format!("Main = (API.normal Book.{})\n", arg),
    _       => panic!("invalid command"),
  };
  let hvm1_code = format!("{}\n{}\n{}", kind_hvm1, book_hvm1, main_hvm1);
  return hvm1_code;
}

fn main() {
  let args: Vec<String> = env::args().collect();

  if args.len() < 3 {
    show_help();
  }

  let cmd = &args[1];
  let arg = &args[2];

  //println!("loading");
  match cmd.as_str() {
    "check" | "run" => {
      match Book::load(arg) {
        Ok(book) => {
          println!("{}", book.show());

          // Generates the HVM1 checker.
          let check_hvm1 = generate_check_hvm1(&book, cmd, arg);
          let mut file = File::create(".check.hvm1").expect("Failed to create '.check.hvm1'.");
          file.write_all(check_hvm1.as_bytes()).expect("Failed to write '.check.hvm1'.");

          // Calls HVM1 and get outputs.
          let output = Command::new("hvm1").arg("run").arg("-t").arg("1").arg("-c").arg("-f").arg(".check.hvm1").arg("(Main)").output().expect("Failed to execute command");
          let stdout = String::from_utf8_lossy(&output.stdout);
          let stderr = String::from_utf8_lossy(&output.stderr);

          // Parses and print stdout infos.
          let parsed = KindParser::new(&stdout).parse_infos();
          match parsed {
            Ok(msgs) => {
              for msg in &msgs {
                println!("{}", msg.pretty(&book))
              }
              if msgs.len() == 0 {
                println!("check!");
              }
            }
            Err(err) => println!("{}", err),
          }
          
          // Prints stdout errors and stats.
          eprintln!("{}", stderr);
        },
        Err(e) => {
          eprintln!("{}", e);
          std::process::exit(1);
        },
      }
    },
    _ => {
      show_help();
    },
  }
}

fn show_help() {
  eprintln!("Usage: kind2 [check|run] <name>");
  std::process::exit(1);
}
