// standard library
use std::fs::File;
use std::io::{BufWriter, Write};

// internal modules
use crate::weight_window::WeightWindow;

// ntools modules
use ntools_utils::f;

/// Convenience function for writing a [WeightWindow] into a single wwout file
///
/// A [WeightWindow] instance corresponds to a full set of weight windows for a
/// single particle type.
///
/// These can be combined into a multi-particle wwout using the
/// [write_multi_particle()] function.
///
/// ```rust, no_run
/// # use ntools_weights::{WeightWindow, write_single_particle};
/// let mut ww_set = WeightWindow {
///     weights: vec![0.2, 0.15, 0.4],
///     ..Default::default()
/// };
///
/// // Write to file
/// write_single_particle(&ww_set, "wwout");
/// ```
pub fn write_single_particle(weight_window: &WeightWindow, output: &str) {
    weight_window.write(output);
}

/// Combine multiple weight window sets into a single wwout file
///
/// A [WeightWindow] instance corresponds to a full set of weight windows for a
/// single particle type. This can be written simply using the
/// [write_single_particle()] function or calling
/// [write()](WeightWindow::write) directly.
///
/// This function attempts to combine a list of weight windows into a single
/// wwout file for multiple particles.
///
/// Note that:
/// - Weight window sets are re-sorted by particle type
/// - Duplicate particle types are removed
/// - Sets with inconsistent geometries are removed
///
/// The remaining weight window sets that can be combined will be written to
/// the path provided as `output`.
///
/// ```rust, no_run
/// # use ntools_weights::{WeightWindow, write_multi_particle};
/// let mut neutron = WeightWindow {
///     weights: vec![0.2, 0.15, 0.4],
///     particle: 1,                    // Particle::Neutron
///     ..Default::default()
/// };
///
/// let mut photon = WeightWindow {
///     weights: vec![0.2, 0.15, 0.4],
///     particle: 2,                    // Particle::Photon
///     ..Default::default()
/// };
///
/// // Write a combined NP weight window file
/// let ww_sets = [photon, neutron];
/// let weight_window = write_multi_particle(&ww_sets, "wwout_NP", false);
/// ```
pub fn write_multi_particle(weight_windows: &[WeightWindow], output: &str, padded: bool) {
    let ww_list = preprocess_set(weight_windows);

    // assume fine >2 meshes for now
    let f = File::create(output).expect("Unable to create file");
    let mut f = BufWriter::new(f);

    // block 1
    f.write_all(combined_header(&ww_list, padded).as_bytes())
        .unwrap();
    f.write_all(ww_list[0].block_1().as_bytes()).unwrap();

    // block 2
    f.write_all(ww_list[0].block_2().as_bytes()).unwrap();

    // block 3
    for ww in ww_list {
        f.write_all(ww.block_3().as_bytes()).unwrap();
    }
}

/// Sort by particle type, remove duplicates, and ensure geometry match
fn preprocess_set(weight_windows: &[WeightWindow]) -> Vec<&WeightWindow> {
    let mut ww_list = weight_windows.iter().collect::<Vec<&WeightWindow>>();

    // Sort by particle type and get rid of any duplicates
    ww_list.sort_by_key(|&k| k.particle);
    ww_list.dedup_by_key(|k| k.particle);

    // Get rid of any that do not mach the mesh geometry
    let target = ww_list[0];
    ww_list.retain(|&ww| is_geometry_match(ww, target));
    ww_list.retain(|&ww| ww.particle != 0);
    ww_list
}

/// The wwout file forces the same geometry for every weight set
fn is_geometry_match(a: &WeightWindow, b: &WeightWindow) -> bool {
    if a.nr  != b.nr  // words (meshtype)
        || a.nfx != b.nfx
        || a.nfy != b.nfy
        || a.nfz != b.nfz
        || a.x0  != b.x0
        || a.y0  != b.y0
        || a.z0  != b.z0
        || a.x1  != b.x1
        || a.y1  != b.y1
        || a.z1  != b.z1
        || a.qps_x != b.qps_x
        || a.qps_y != b.qps_y
        || a.qps_z != b.qps_z
    {
        return false;
    }

    true
}

/// Generate the appropriate header values for a multi-particle file
///
/// Header fromat is outlined below. The number or particle types `ni`, flag for
/// time bins `iv`, and number of time `nt` and energy `ne` bins lists need to
/// be updated from the single particle case.
///
/// | Format            | Variables                 |
/// | ----------------- | ------------------------- |
/// | 4i10, 20x, a19    |  if iv ni nr probid       |
/// | 7i10              |  nt(1)...nt(ni) if iv=2   |
/// | 7i10              |  ne(1)...ne(ni)           |
///
/// The 7i suggests a maximum of 7 particle types per line, and this will need
/// zero padding to ensure all the correct particle types are used.
fn combined_header(weight_windows: &[&WeightWindow], padded: bool) -> String {
    let base = weight_windows[0];
    let iv = if weight_windows.iter().any(|ww| ww.iv == 2) {
        2
    } else {
        1
    };

    let (nt, ne) = match padded {
        true => particle_lists_padded(weight_windows),
        false => particle_lists_unpadded(weight_windows),
    };

    // if iv ni nr probid
    let mut s = f!("{:>10}{:>10}{:>10}{:>10}\n", base.f, iv, ne.len(), base.nr,);

    // nt(1) ... nt(ni) [if iv=2]
    let mut count: u8 = 1;
    if iv == 2 {
        for n_times in nt {
            s += &f!("{:>10}", n_times);
            s += track_newlines(&mut count, 7); // 7i10 => split on 7
        }
        if !s.ends_with('\n') {
            s += "\n";
        }
    }

    // ne(1) ... ne(ni)
    count = 1;
    for n_energies in ne {
        s += &f!("{:>10}", n_energies);
        s += track_newlines(&mut count, 7); // 7i10 => split on 7
    }

    if !s.ends_with('\n') {
        s += "\n";
    }

    s
}

/// List of number of particle time/energy groups
fn particle_lists_unpadded(weight_windows: &[&WeightWindow]) -> (Vec<usize>, Vec<usize>) {
    (
        weight_windows.iter().map(|ww| ww.nt).collect(),
        weight_windows.iter().map(|ww| ww.ne).collect(),
    )
}

/// List of number of particle time/energy groups, with 0 for missing types
fn particle_lists_padded(weight_windows: &[&WeightWindow]) -> (Vec<usize>, Vec<usize>) {
    let max = weight_windows
        .iter()
        .max_by_key(|ww| ww.particle)
        .unwrap()
        .particle as usize;

    let mut nt = vec![0_usize; max];
    let mut ne = vec![0_usize; max];

    for ww in weight_windows {
        let idx = (ww.particle - 1) as usize;
        nt[idx] = ww.nt;
        ne[idx] = ww.ne;
    }

    (nt, ne)
}

/// Return a newline character once the counter reaches a target
///
/// Unfortunately textwrap is extremely slow on large lines, so more efficient
/// to just split as we go manually.
pub(crate) fn track_newlines(count: &mut u8, target: u8) -> &str {
    if *count == target {
        *count = 1;
        "\n"
    } else {
        *count += 1;
        ""
    }
}
