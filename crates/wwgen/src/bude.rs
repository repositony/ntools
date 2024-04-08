//! Module for combining meshes for Build Up Density Extrapolation

// crate modules
use crate::error::Result;

// neutronics toolbox
use ntools_mesh::Mesh;

// standard library
use core::iter::zip;

// external crates
use itertools::izip;

/// Generate new mesh using the BUDE method
pub fn extrapolate_density(
    vd: &Mesh,
    rd: &Mesh,
    uc: &Mesh,
    gamma: f64,
    ratio: f64,
) -> Result<Mesh> {
    // todo: some sort of common sense checks for compatability
    // todo: smart way of minimising allocations and clones

    // get the buildup flux
    let buildup = buildup_flux(rd, uc, gamma, ratio);

    // calculate the forward flux
    let forward = forward_flux(&buildup, uc, vd, ratio);
    Ok(forward)
}

/// Calculate the intermediate buildup flux mesh
fn buildup_flux(reduced_density: &Mesh, uncollided: &Mesh, gamma: f64, density_ratio: f64) -> Mesh {
    let mut buildup = reduced_density.clone(); // avoid this
    let factor = gamma * density_ratio;

    for (bu, uc) in zip(&mut buildup.voxels, &uncollided.voxels) {
        // reduced density / uncollided, with errors sorted out automatically
        *bu /= *uc;
        // then raise the result to gamma * density_ratio
        *bu = bu.powf(factor)
    }

    buildup
}

/// Calculate the forward flux mesh
fn forward_flux(buildup: &Mesh, uncollided: &Mesh, void: &Mesh, density_ratio: f64) -> Mesh {
    let mut forward_flux = buildup.clone();

    for (fw, bu, uc, vd) in izip!(
        &mut forward_flux.voxels,
        &buildup.voxels,
        &uncollided.voxels,
        &void.voxels
    ) {
        *fw = (*uc / *vd).powf(density_ratio);
        *fw *= *bu * *vd;
    }

    forward_flux
}
