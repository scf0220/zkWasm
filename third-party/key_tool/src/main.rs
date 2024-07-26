mod fs_util;
mod zksync_key;

use crate::fs_util::read_zk_sync_key;
use crate::fs_util::write_to_halo2;
use halo2_proofs::pairing::arithmetic::CurveAffine;
use halo2_proofs::pairing::bn256::Fq;
use halo2_proofs::pairing::bn256::Fq2;
use halo2_proofs::pairing::bn256::G1Affine;
use halo2_proofs::pairing::bn256::G2Affine;
use num_traits::Num;
use pairing_bn256::group::ff::PrimeField;
use pairing_bn256::group::GroupEncoding;
use pairing_ce::bn256::Fq as FqCE;
use pairing_ce::bn256::G2Affine as G2AffineCE;
use pairing_ce::CurveAffine as CurveAffineCE;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::thread;

static DEFAULT_K: u32 = 23;

// Convert pairing_ce::G2Affine to pairing::G2Affine.
fn trans_g2(g2: G2AffineCE) -> G2Affine {
    let (x_ce, y_ce) = g2.as_xy();

    let mut x = Fq2::default();
    x.c0 = trans_fq(x_ce.c0);
    x.c1 = trans_fq(x_ce.c1);
    let mut y = Fq2::default();
    y.c0 = trans_fq(y_ce.c0);
    y.c1 = trans_fq(y_ce.c1);
    return G2Affine::from_xy(x, y).unwrap();
}

// Convert pairing_ce::Fq to pairing::Fq.
fn trans_fq(x: FqCE) -> Fq {
    let pp = Fq::from_str_vartime(&*extract_decimal_from_string(&x.to_string())).unwrap();
    return pp;
}

// Convert fq to a positive decimal.
// input="Fq(0x24fc1e1c263a7de7abec5edaeea87625890c96a018bb8c60522333fa206f70c3)"
// output=16728715820616582450594109459208172618408974327542441440317506932429837791427
fn extract_decimal_from_string(s: &str) -> String {
    let hex_str = &s[5..s.len() - 1];
    let tt = num_bigint::BigUint::from_str_radix(hex_str, 16)
        .unwrap()
        .to_string();
    return tt;
}

// Confirm the value of k from the command line.
fn get_k_from_args(args: Vec<String>) -> u32 {
    if args.len() < 2 {
        return DEFAULT_K;
    }
    let k: u32 = args[1]["--k=".len()..]
        .parse()
        .unwrap_or_else(|_| DEFAULT_K);
    println!("k is {:?}", k);
    return k;
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let k = get_k_from_args(args);

    let k_extra_info = zksync_key::ZKSYNC_KEY_URL.get(&k);
    if k_extra_info.is_none() {
        panic!("k={:?} is not support", k)
    }

    let k_extra_info = k_extra_info.unwrap();
    let check_status = k_extra_info.clone().check_setup_key_file();
    if check_status.is_err() {
        panic!("check setup failed err={:?}", check_status.err())
    }

    let mut buf_reader_lagrange = BufReader::with_capacity(
        1 << 29,
        File::open(k_extra_info.clone().get_local_lagrange_path()).unwrap(),
    );
    let (lagrange_key, g2_base) = read_zk_sync_key(&mut buf_reader_lagrange).unwrap();

    let mut buf_reader_monomial = BufReader::with_capacity(
        1 << 29,
        File::open(k_extra_info.clone().get_local_monomial_path()).unwrap(),
    );

    let (monomial_key, _) = read_zk_sync_key(&mut buf_reader_monomial).unwrap();

    let handle_lagrange = thread::spawn(move || {
        let mut g_lagrange = Vec::new();
        for index in 0..lagrange_key.len() {
            let (x, y) = lagrange_key[index].as_xy();
            g_lagrange.push(G1Affine::from_xy(trans_fq(*x), trans_fq(*y)).unwrap());
        }
        return g_lagrange;
    });

    let handle_normal = thread::spawn(move || {
        let mut g = Vec::new();
        for index in 0..monomial_key.len() {
            let (x, y) = monomial_key[index].as_xy();
            g.push(G1Affine::from_xy(trans_fq(*x), trans_fq(*y)).unwrap());
        }
        return g;
    });

    let g_lagrange = handle_lagrange.join().unwrap();
    let g_monomial_key = handle_normal.join().unwrap();
    let additional_data = trans_g2(g2_base[1]).to_bytes().as_ref().to_vec();
    println!("finish read zksync key k={:?}", k);
    let file_path = format!("./K{:?}.params", k);
    let mut fd = File::create(file_path.clone()).unwrap();
    write_to_halo2(&mut fd, k, g_monomial_key, g_lagrange, additional_data)
        .expect("TODO: panic message");
    println!(
        "finish write halo2 params  k={:?} file_path={:?}",
        k, file_path
    )
}
