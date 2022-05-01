use indicatif::ProgressBar;
use std::{fs::File, io::Write};

use anyhow::Result;
use clap::{Arg, Command};

use indicatif::ProgressStyle;
use ps_memory_reader::{io::Io, memory_card::MemoryCard};

fn main() {
    env_logger::init();

    let matches = Command::new("ps-memory-reader")
        .about("PlayStation MemoryCard reader")
        .version("0.1.0")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .author("mjhd <mjhd.devlion@gmail.com>")
        .subcommand(
            Command::new("read").arg(
                Arg::new("output")
                    .short('o')
                    .long("output")
                    .help("output file name")
                    .required(true)
                    .takes_value(true),
            ),
        )
        .subcommand(
            Command::new("dump").arg(
                Arg::new("output")
                    .short('o')
                    .long("output")
                    .help("output file name")
                    .required(true)
                    .takes_value(true),
            ),
        )
        .get_matches();

    let result = match matches.subcommand() {
        //Some(("read", params)) => read_data(params.value_of("output").unwrap().to_string(), false),
        Some(("dump", params)) => dump_data(params.value_of("output").unwrap().to_string()),
        _ => panic!("unknown command"),
    };

    result.unwrap();
}

//fn read_data(output: String, repl: bool) -> Result<()> {
//    println!("リーダーの初期化...");
//
//    let io = Io::new()?;
//    let mut memory_card = MemoryCard::new(io);
//
//    let blocks = memory_card.blocks()?;
//    let total = memory_card.size()?;
//
//    println!("ブロック数: {}, サイズ: {}", blocks, HumanBytes(total));
//
//    let mut file = File::create(&output)?;
//
//    let reading = ProgressBar::new(total);
//
//    reading.set_style(
//        ProgressStyle::default_bar()
//            .template("[{elapsed_precise}({eta})] {msg} [{bar:.cyan/blue}] {bytes}/{total_bytes}")
//            .progress_chars("#>-"),
//    );
//
//    println!("読み込み開始...");
//
//    for i in 1..blocks {
//        let block = memory_card.read_block(i as u8)?;
//
//        file.write(&block)?;
//
//        reading.inc(block.len() as u64);
//        reading.set_message(format!("BLOCK #{}", i));
//    }
//
//    println!("読み込み終了...");
//
//    file.flush()?;
//
//    println!("完了！");
//
//    reading.finish_and_clear();
//
//    Ok(())
//}

fn dump_data(output: String) -> Result<()> {
    println!("リーダーの初期化...");

    let io = Io::new()?;
    let mut memory_card = MemoryCard::new(io)?;

    let mut file = File::create(&output)?;

    let reading = ProgressBar::new(128 * 1024);

    reading.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}({eta})] {msg} [{bar:.cyan/blue}] {bytes}/{total_bytes}")
            .progress_chars("#>-"),
    );

    println!("読み込み開始...");

    for i in 0..16 {
        let block = memory_card.read_block(i as u8)?;

        file.write(&block)?;

        reading.inc(block.len() as u64);
        reading.set_message(format!("BLOCK #{}", i));
    }

    println!("読み込み終了...");

    file.flush()?;

    println!("完了！");

    reading.finish_and_clear();

    Ok(())
}
