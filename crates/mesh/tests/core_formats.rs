//! Integration tests for core output types

use ntools_mesh::{read_meshtal_target, Mesh};
use rstest::{fixture, rstest};

#[fixture]
fn ref_single() -> Mesh {
    read_meshtal_target("./data/meshes/fmesh_104.msht", 104).unwrap()
}

#[fixture]
fn ref_multi() -> Mesh {
    read_meshtal_target("./data/meshes/fmesh_114.msht", 114).unwrap()
}

#[rstest]
#[case("./data/meshes/fmesh_124.msht", 124)] // case 1
#[case("./data/meshes/fmesh_204.msht", 204)] // case 2
#[case("./data/meshes/fmesh_224.msht", 224)] // case 3
#[case("./data/meshes/fmesh_304.msht", 304)] // case 4
#[case("./data/meshes/fmesh_324.msht", 324)] // case 5
#[case("./data/meshes/fmesh_404.msht", 404)] // case 6
#[case("./data/meshes/fmesh_424.msht", 424)] // case 7
#[case("./data/meshes/fmesh_504.msht", 504)] // case 8
#[case("./data/meshes/fmesh_524.msht", 524)] // case 9
fn parse_meshtal_simple(ref_single: Mesh, #[case] path: &str, #[case] id: u32) {
    let test = read_meshtal_target(path, id).unwrap();
    for (a, b) in ref_single.voxels.iter().zip(test.voxels.iter()) {
        assert_eq!(a.result, b.result);
        assert_eq!(a.error, b.error);
    }
}

#[rstest]
#[case("./data/meshes/fmesh_134.msht", 134)] // case 1
#[case("./data/meshes/fmesh_214.msht", 214)] // case 2
#[case("./data/meshes/fmesh_234.msht", 234)] // case 3
#[case("./data/meshes/fmesh_314.msht", 314)] // case 4
#[case("./data/meshes/fmesh_334.msht", 334)] // case 5
#[case("./data/meshes/fmesh_414.msht", 414)] // case 6
#[case("./data/meshes/fmesh_434.msht", 434)] // case 7
#[case("./data/meshes/fmesh_514.msht", 514)] // case 8
#[case("./data/meshes/fmesh_534.msht", 534)] // case 9
fn parse_meshtal_multigroup(ref_multi: Mesh, #[case] path: &str, #[case] id: u32) {
    let test = read_meshtal_target(path, id).unwrap();
    for (a, b) in ref_multi.voxels.iter().zip(test.voxels.iter()) {
        assert_eq!(a.result, b.result);
        assert_eq!(a.error, b.error);
    }
}
