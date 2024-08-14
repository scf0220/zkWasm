// This file contains code copied from https://github.com/matter-labs/bellman
// Licensed under the APACHE License and MIT License.
// See LICENSE file for details.
use byteorder::BigEndian;
use byteorder::ReadBytesExt;
use pairing_bn256::bn256::G1Affine;
use pairing_bn256::group::GroupEncoding;
use pairing_ce::bn256::G1Affine as G1AffineCE;
use pairing_ce::bn256::G2Affine as G2AffineCE;
use pairing_ce::CurveAffine as CurveAffineCE;
use pairing_ce::EncodedPoint;
use std::io;
use std::io::Read;

// Read the zkSync key file and convert it to pairing_ce::G1Affine and G2Affine.
pub fn read_zk_sync_key<R: Read>(
    mut reader: R,
) -> anyhow::Result<(Vec<G1AffineCE>, Vec<G2AffineCE>)> {
    let mut g1_repr = <G1AffineCE as CurveAffineCE>::Uncompressed::empty();
    let mut g2_repr = <G2AffineCE as CurveAffineCE>::Uncompressed::empty();

    let num_g1 = reader.read_u64::<BigEndian>()?;

    let mut g1_bases = Vec::with_capacity(num_g1 as usize);

    for _ in 0..num_g1 {
        reader.read_exact(g1_repr.as_mut())?;
        let p = g1_repr
            .into_affine()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        g1_bases.push(p);
    }


    let (x0,y0)=g1_bases[0].as_xy();
    println!("x0={:?} y0={:?}",x0,y0);

    let (x1,y1)=g1_bases[1].as_xy();
    println!("x1={:?} y1={:?}",x1,y1);

    let num_g2 = reader.read_u64::<BigEndian>()?;
    assert!(num_g2 == 2u64);

    let mut g2_bases = Vec::with_capacity(num_g2 as usize);

    for _ in 0..num_g2 {
        reader.read_exact(g2_repr.as_mut())?;
        let p = g2_repr
            .into_affine()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        g2_bases.push(p);
    }
    println!("num_g2={:?} zero={:?} first={:?}",num_g2,g2_bases[0],g2_bases[1]);
    return Ok((g1_bases, g2_bases));
}

// Write the data required by Halo2 to a file.
pub fn write_halo2_params<W: io::Write>(
    writer: &mut W,
    k: u32,
    g: Vec<G1Affine>,
    g_lagrange: Vec<G1Affine>,
    additional_data: Vec<u8>,
) -> anyhow::Result<()> {
    writer.write_all(&k.to_le_bytes())?;

    println!("0 {:?} {:?}",g[0].x,g[0].y);
    println!("1 {:?} {:?}",g[1].x,g[1].y);
    println!("1000000 {:?} {:?}",g[1000000].x,g[1000000].y);
    println!("2000000 {:?} {:?}",g[2000000].x,g[2000000].y);
    for el in &g {
        writer.write_all(el.to_bytes().as_ref())?;
    }

    println!("0 {:?} {:?}",g_lagrange[0].x,g_lagrange[0].y);
    println!("1 {:?} {:?}",g_lagrange[1].x,g_lagrange[1].y);
    println!("1000000 {:?} {:?}",g_lagrange[1000000].x,g_lagrange[1000000].y);
    println!("2000000 {:?} {:?}",g_lagrange[2000000].x,g_lagrange[2000000].y);
    for el in &g_lagrange {
        writer.write_all(el.to_bytes().as_ref())?;
    }
    let additional_data_len = additional_data.len() as u32;
    writer.write_all(&additional_data_len.to_le_bytes())?;
    writer.write_all(&additional_data)?;


    println!("addiotion_data {:?} {:?}",additional_data_len,additional_data);

    return Ok(());
}
