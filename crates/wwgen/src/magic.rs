// neutronics toolbox
use ntools_mesh::{Geometry, Mesh, Voxel};
use ntools_weights::WeightWindow;

use log::warn;

/// Mesh tally to global weight windows with simple parameters
///
/// A constant power factor and error tolerance are applied to all energy/time
/// groups.
///
/// - `powers` - Softening factor used as ww=>ww^power
/// - `max_errors` - Errors above this are set to 0/analogue
/// - `total_only` - Only generate weights from [Group::Total](crate::mesh::Group)
///
/// Weights are calculated as `(0.5 * (v.result / flux_ref)).powf(power)`. For
/// example, applying a 0.7 de-tuning factor and setting voxels with errors
/// below 10% to analogue:
///
/// ```rust, no_run
/// # use ntools_mesh::read_meshtal_target;
/// # use ntools_wwgen::mesh_to_ww;
/// // Read tally 104 from a meshtal file
/// let mesh = read_meshtal_target("./data/meshes/fmesh_104.msht", 104).unwrap();
/// // Convert the mesh into a weight window set
/// let weight_window = mesh_to_ww(&mesh, 0.7, 0.10, false);
/// ```
///
/// By default, this generates weight windows for all time and energy groups.
/// To generate a simpler set of weight windows based only on the
/// [Group::Total](crate::mesh::Group), set the `total_only` boolean to `true`.
pub fn mesh_to_ww(mesh: &Mesh, power: f64, max_error: f64, total_only: bool) -> WeightWindow {
    let mut ww: WeightWindow = initialise_ww_from_mesh(mesh, total_only);
    ww.weights = compute_weights(mesh, &[power], &[max_error], total_only);
    ww
}

/// Mesh tally to global weight windows with fine de-tuning and errors
///
/// Same as [mesh_to_ww] but allows for individual de-tuning factors and error
/// tolerances for each group. If `powers` or `max_errors` have a single entry
/// this will be applied to all groups.
///
/// - `powers` - Softening factor used as ww=>ww^power
/// - `max_errors` - Errors above this are set to 0/analogue
///
/// A call to this may look like this, applying separate powers and errors to
/// a mesh with 3 energy groups:
///
/// ```rust, no_run
/// # use ntools_mesh::read_meshtal_target;
/// # use ntools_wwgen::mesh_to_ww_advanced;
/// // Read tally 104 from a meshtal file
/// let mesh = read_meshtal_target("./data/meshes/fmesh_104.msht", 104).unwrap();
/// // Convert the mesh into a set of weight windows, using different parameters per set
/// let ww = mesh_to_ww_advanced(&mesh,
///                              &[0.7, 0.5, 0.85],
///                              &[0.1, 0.1, 0.15]);
/// ```
///
/// The lists should be ordered such that they match the following nested order:
///
/// ```ignore
/// for energy in energy_groups {
///     for time in time_groups {
///         calculate weights...
///     }
/// }
/// ```
///
/// For example, the following energy and time groups are related to the groups
/// shown explicitly below.
///
/// ```text
/// Energy bin boundaries: 0.0 10.0 200.0
/// Time bin boundaries  : -1E+36 0.0 1E+16 1E+99
/// ```
///
/// ```text
/// 0 -> Energy(10.0)   Time(0.0)       powers[0]   max_errors[0]
/// 1 -> Energy(10.0)   Time(1E+16)     powers[1]   max_errors[1]
/// 2 -> Energy(10.0)   Time(1E+99)     powers[2]   max_errors[2]
/// 3 -> Energy(200.0)  Time(0.0)       powers[3]   max_errors[3]
/// 4 -> Energy(200.0)  Time(1E+16)     powers[4]   max_errors[4]
/// 5 -> Energy(200.0)  Time(1E+99)     powers[5]   max_errors[5]
/// ```
pub fn mesh_to_ww_advanced(mesh: &Mesh, powers: &[f64], max_errors: &[f64]) -> WeightWindow {
    let mut ww: WeightWindow = initialise_ww_from_mesh(mesh, false);
    ww.weights = compute_weights(mesh, powers, max_errors, false);
    ww
}

/// Core function for setting up the weight mesh geometry
///
/// This initialises everything but the weights themselves, setting up all
/// geometry bounds and required parameters taken or inferred from the provided
/// [Mesh](crate::mesh::Mesh).
///
/// This is decoupled from the weights as it can be useful to just be able to
/// do the setup and weight calculations separately. However, the public API
/// brings these together to ensure they are used correctly.
fn initialise_ww_from_mesh(mesh: &Mesh, total_only: bool) -> WeightWindow {
    // for what this shit means look up appendix B of the mcnp6 manual
    let mut ww = WeightWindow {
        nr: match mesh.geometry {
            Geometry::Rectangular => 10,
            Geometry::Cylindrical => 16,
        },
        nwg: mesh.geometry as u8,
        nfx: mesh.iints,
        nfy: mesh.jints,
        nfz: mesh.kints,
        ncx: mesh.iints,
        ncy: mesh.jints,
        ncz: mesh.kints,
        x0: mesh.origin[0],
        y0: mesh.origin[1],
        z0: mesh.origin[2],
        x1: mesh.axs[0],
        y1: mesh.axs[1],
        z1: mesh.axs[2],
        x2: mesh.vec[0],
        y2: mesh.vec[1],
        z2: mesh.vec[2],
        // total only will just be the maximum energy recorded I suppose
        e: if total_only {
            vec![*mesh.emesh.last().unwrap()]
        } else {
            mesh.emesh[1..].to_vec()
        },
        qps_x: qps_tuples(&mesh.imesh),
        qps_y: qps_tuples(&mesh.jmesh),
        qps_z: qps_tuples(&mesh.kmesh),
        particle: mesh.particle.id(),
        ..Default::default()
    };

    // number of energy bins
    ww.ne = ww.e.len();

    // only bother including time info if relevant
    if mesh.tbins() > 1 && !total_only {
        ww.iv = 2;
        ww.nt = mesh.tbins();
        ww.t = mesh.tmesh[1..].to_vec();
    }

    ww
}

/// Core function for turning a flux mesh into weights
///
/// Processes each group in order because each has to be normalised to itself.
/// Because of this, it is easy to apply different power factors and error
/// tolerences for each group. The signature therefore takes lists for both
/// parameters.
///
/// For the typical functionality the `powers` and `max_errors` list may be just
/// one value long, which will be applied to every group.
fn compute_weights(mesh: &Mesh, powers: &[f64], max_errors: &[f64], total_only: bool) -> Vec<f64> {
    let (energy_groups, time_groups) = relevant_groups_idx(mesh, total_only);

    // set up the weights vector
    let n_groups = energy_groups.len() * time_groups.len();
    let n_voxels = n_groups * mesh.iints * mesh.jints * mesh.kints;
    let mut weights: Vec<f64> = Vec::with_capacity(n_voxels);

    // collect the powers to be used for each group
    let powers = collect_power_values(powers, n_groups);
    let mut powers_iter = powers.iter();

    // collect the error tolerance to be used for each group
    let errors = collect_error_values(max_errors, n_groups);
    let mut errors_iter = errors.iter();

    // Loop over all the requested groups and apply the appropriate factors
    for e_idx in &energy_groups {
        for t_idx in &time_groups {
            // really want slice by idx
            let voxels = mesh.slice_voxels_by_idx(*e_idx, *t_idx).unwrap();
            weights.extend(weight_from_voxels(
                mesh,
                voxels,
                *powers_iter.next().unwrap(),
                *errors_iter.next().unwrap(),
            ));
        }
    }

    weights
}

/// Calculates the weights for a set of voxels
///
/// This is done in groups so that each energy/time group can be normalised
/// properly. As a nice side effect this makes it very easy to have different
/// power factors and error tolerances for each group.
///
/// Weights are calculated as `(0.5 * (v.result / flux_ref)).powf(power)`
fn weight_from_voxels(mesh: &Mesh, voxels: &[Voxel], power: f64, max_error: f64) -> Vec<f64> {
    // find maximum of the energy/time group set
    let flux_ref = voxels
        .iter()
        .map(|v| v.result)
        .max_by(|a, b| a.total_cmp(b))
        .unwrap();

    // guard against groups with no results
    if flux_ref == 0.0 {
        return vec![0.0; voxels.len()];
    }

    // Main calculation, very simple
    let mut wgt: Vec<(usize, f64)> = Vec::with_capacity(voxels.len());
    for (i, v) in voxels.iter().enumerate() {
        let mut w = if v.error <= max_error {
            (0.5 * (v.result / flux_ref)).powf(power)
        } else {
            0.0
        };

        // ensure the value is reasonable (looking at you CuV)
        w = constrain_weights(w);
        wgt.push((mesh.voxel_index_to_cell_index(i), w));
    }

    wgt.sort_by(|a, b| a.0.cmp(&b.0));
    wgt.into_iter().map(|r| r.1).collect()
}

/// Generate a list of error cuts for every group in the weight window mesh
///
/// The power valuse are assumed to be in the correct order corresponding to the
/// following processing loops:
///
/// ```ignore
/// for energy in energy_groups
///     for time in time_groups
///         calculate weights...
/// ```
///
/// There are several cases that this handles.
///
/// - Multiple factors - apply each power to its respective group
/// - Single factor - apply the same power factor to evey group
/// - Empty list - default to 0.7
///
/// If the length of the multiple powers list is not valid, the first factor is
/// applied to all groups and a warning raised.
fn collect_power_values(powers: &[f64], n_groups: usize) -> Vec<f64> {
    let n_powers = powers.len();

    match n_powers {
        0 => {
            warn!("Warning: No power factor provided, defaulting to 0.7");
            [0.7].repeat(n_groups)
        }
        1 => powers.repeat(n_groups),
        _ => {
            if n_powers == n_groups {
                powers.to_vec()
            } else {
                warn!("Warning: Power factors != number of groups");
                warn!("  - Expected {}, found {}", n_groups, n_powers);
                warn!("  - Setting all groups to 0.7");
                [0.7].repeat(n_groups)
            }
        }
    }
}

/// Generate a list of power factors for every group in the weight window mesh
///
/// Like the powers, the energy tolerances are also assumed to be in the correct
/// order corresponding to the following processing loops:
///
/// ```ignore
/// for energy in energy_groups
///     for time in time_groups
///         calculate weights...
/// ```
///
/// There are several cases that this handles.
///
/// - Multiple errors - apply each error tolerance to its respective group
/// - Single error - apply the same error tolerance to evey group
/// - Empty list - default to 1.0 (100%)
///
/// If the length of the multiple errors list is not valid, the first tolerance
/// is applied to all groups and a warning raised.
fn collect_error_values(errors: &[f64], n_groups: usize) -> Vec<f64> {
    let n_errors = errors.len();

    match n_errors {
        0 => {
            warn!("Warning: No error tolerance provided, defaulting to 1.0");
            [1.0].repeat(n_groups)
        }
        1 => errors.repeat(n_groups),
        _ => {
            if n_errors == n_groups {
                errors.to_vec()
            } else {
                warn!("Warning: Error tolerences != number of groups");
                warn!("  - Expected {}, found {}", n_groups, n_errors);
                warn!("  - Setting all error cuts to 1.0");
                [1.0].repeat(n_groups)
            }
        }
    }
}

/// Builds required tuples for coarse mesh bounds
///
/// The "qps" variables are for the coarse mesh bounds:
/// - q = Fine mesh ratio (1 always) in each coarse mesh
/// - p = Coarse mesh coordinates for (x,y,z), (r,z,t), or (r,p,t)
/// - s = Number of fine meshes in each coarse mesh for (x,y,z), (r,z,t), or (r,p,t)
fn qps_tuples(mesh_bounds: &[f64]) -> Vec<[f64; 3]> {
    mesh_bounds[1..]
        .iter()
        .map(|bound| [1.0, *bound, 1.0])
        .collect()
}

/// Collect up the relevant energy and time groups indicies
///
/// Either just returns the `Total` for total-only selections, or will every
/// valued group if there are multiple.
fn relevant_groups_idx(mesh: &Mesh, total_only: bool) -> (Vec<usize>, Vec<usize>) {
    let ebins = mesh.ebins();
    let tbins = mesh.tbins();

    match total_only {
        true => (vec![ebins - 1], vec![tbins - 1]),
        false => {
            // either total or the valued groups of emesh
            let energies = if ebins > 1 {
                (0..ebins - 1).collect()
            } else {
                vec![ebins - 1]
            };

            // either total or the valued groups of tmesh
            let times = if tbins > 1 {
                (0..tbins - 1).collect()
            } else {
                vec![tbins - 1]
            };

            (energies, times)
        }
    }
}

/// Fix ridiculous values that may happen for CuV
fn constrain_weights(weight: f64) -> f64 {
    if weight < 1.0e-99 {
        0.0
    } else if weight >= 1.0e+100 {
        9.999e99
    } else {
        weight
    }
}
