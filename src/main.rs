mod maze_field;
mod maze_point;
mod maze_point_status;

//https://trap.jp/post/472/

//
//    「壁の延びていない『柱』の座標の集合」として配列nodesを、「探索済みの『柱』の座標の集合」として配列pathをそれぞれ用意する。
//
//    配列nodesからランダムに1つ「柱」を選択し取り出す。
//
//    選択中の「柱」の座標から上下左右いずれかに移動する。
//
//    移動した先が「探索済みの（＝pathに含まれる）柱」の時は方向を選び直す。上下左右全てで試した場合は現在選択中の「柱」をnodesに入れ直し、一つ前の「柱」へ戻って方向を選び直す。
//
//    移動した先が「未探索の（＝nodesに含まれる）柱」の時はその「柱」を選択し、nodesから取り出してpathに記録する。その後、3に戻って操作を繰り返す。
//
//    移動した先がすでに「壁（＝nodesに含まれない「柱」または外壁）」である時、ここまで移動してきた道のりを全て「壁」に置き換え、pathを空にする。続いて2まで戻り、一連の操作をnodesが空になるまで繰り返す。
//
//    nodesが空になったら終了する。
//

use clap::{Parser, arg, command};
use log::{LevelFilter, debug, error, info};
use rand::Rng;
use std::error::Error;
use std::{collections::HashMap, thread};
use sysinfo::{CpuRefreshKind, RefreshKind, System};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(
        short = 'x',
        long = "x_size",
        default_value = "641",
        help = "x方向のピクセル数を指定します。"
    )]
    x_size: u32,
    #[arg(
        short = 'y',
        long = "y_size",
        default_value = "481",
        help = "y方向のピクセル数を指定します。"
    )]
    y_size: u32,

    #[arg(short = 'd', long = "debug", help = "デバッグモードを有効にします。")]
    debug: bool,
}

fn get_workers_limit() -> usize {
    let core_count: usize = match num_cpus::get() {
        0 => 1,
        n => n,
    };
    let workers_limit: usize = (core_count - 1) / 4;
    if workers_limit == 0 {
        return 1;
    };
    workers_limit
}

fn main() {
    let args = Args::parse();

    // env_logger::Builderを使用した安全な方法
    let log_level = if args.debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    env_logger::Builder::from_default_env()
        .filter_level(log_level)
        .init();

    info!("Application started with args: {:?}", args);
}
