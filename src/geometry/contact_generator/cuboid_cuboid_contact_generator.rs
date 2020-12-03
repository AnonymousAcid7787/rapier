use crate::geometry::contact_generator::PrimitiveContactGenerationContext;
use crate::geometry::{cuboid, sat, ContactManifold, CuboidFeature, KinematicsCategory};
use crate::math::Isometry;
#[cfg(feature = "dim2")]
use crate::math::Vector;
use ncollide::shape::Cuboid;

pub fn generate_contacts_cuboid_cuboid(ctxt: &mut PrimitiveContactGenerationContext) {
    if let (Some(cube1), Some(cube2)) = (ctxt.shape1.as_cuboid(), ctxt.shape2.as_cuboid()) {
        generate_contacts(ctxt.prediction_distance, cube1, cube2, ctxt.manifold);
    } else {
        unreachable!()
    }

    ctxt.manifold.update_warmstart_multiplier();
    ctxt.manifold.sort_contacts(ctxt.prediction_distance);
}

pub fn generate_contacts<'a>(
    prediction_distance: f32,
    mut cube1: &'a Cuboid<f32>,
    mut cube2: &'a Cuboid<f32>,
    manifold: &mut ContactManifold,
) {
    let mut pos12 = manifold.position1.inverse() * manifold.position2;
    let mut pos21 = pos12.inverse();

    if manifold.try_update_contacts(&pos12) {
        return;
    }

    /*
     *
     * Point-Face
     *
     */
    let sep1 = sat::cuboid_cuboid_find_local_separating_normal_oneway(cube1, cube2, &pos12, &pos21);
    if sep1.0 > prediction_distance {
        manifold.points.clear();
        return;
    }

    let sep2 = sat::cuboid_cuboid_find_local_separating_normal_oneway(cube2, cube1, &pos21, &pos12);
    if sep2.0 > prediction_distance {
        manifold.points.clear();
        return;
    }

    /*
     *
     * Edge-Edge cases
     *
     */
    #[cfg(feature = "dim2")]
    let sep3 = (-f32::MAX, Vector::x()); // This case does not exist in 2D.
    #[cfg(feature = "dim3")]
    let sep3 = sat::cuboid_cuboid_find_local_separating_edge_twoway(cube1, cube2, &pos12, &pos21);
    if sep3.0 > prediction_distance {
        manifold.points.clear();
        return;
    }

    /*
     *
     * Select the best combination of features
     * and get the polygons to clip.
     *
     */
    let mut swapped = false;
    let mut best_sep = sep1;

    if sep2.0 > sep1.0 && sep2.0 > sep3.0 {
        // The reference shape will be the second shape.
        std::mem::swap(&mut cube1, &mut cube2);
        std::mem::swap(&mut pos12, &mut pos21);
        manifold.swap_identifiers();
        best_sep = sep2;
        swapped = true;
    } else if sep3.0 > sep1.0 {
        best_sep = sep3;
    }

    // We do this clone to perform contact tracking and transfer impulses.
    // FIXME: find a more efficient way of doing this.
    let old_manifold_points = manifold.points.clone();
    manifold.points.clear();

    // Now the reference feature is from `cube1` and the best separation is `best_sep`.
    // Everything must be expressed in the local-space of `cube1` for contact clipping.
    let feature1 = cuboid::support_feature(cube1, best_sep.1);
    let mut feature2 = cuboid::support_feature(cube2, pos21 * -best_sep.1);
    feature2.transform_by(&pos12);

    match (&feature1, &feature2) {
        (CuboidFeature::Face(f1), CuboidFeature::Vertex(v2)) => {
            CuboidFeature::face_vertex_contacts(f1, &best_sep.1, v2, &pos21, manifold)
        }
        #[cfg(feature = "dim3")]
        (CuboidFeature::Face(f1), CuboidFeature::Edge(e2)) => CuboidFeature::face_edge_contacts(
            prediction_distance,
            f1,
            &best_sep.1,
            e2,
            &pos21,
            manifold,
            false,
        ),
        (CuboidFeature::Face(f1), CuboidFeature::Face(f2)) => CuboidFeature::face_face_contacts(
            prediction_distance,
            f1,
            &best_sep.1,
            f2,
            &pos21,
            manifold,
        ),
        #[cfg(feature = "dim3")]
        (CuboidFeature::Edge(e1), CuboidFeature::Edge(e2)) => {
            CuboidFeature::edge_edge_contacts(e1, &best_sep.1, e2, &pos21, manifold)
        }
        #[cfg(feature = "dim3")]
        (CuboidFeature::Edge(e1), CuboidFeature::Face(f2)) => {
            // Since f2 is also expressed in the local-space of the first
            // feature, the position we provide here is pos21.
            CuboidFeature::face_edge_contacts(
                prediction_distance,
                f2,
                &-best_sep.1,
                e1,
                &pos21,
                manifold,
                true,
            )
        }
        _ => unreachable!(), // The other cases are not possible.
    }

    manifold.local_n1 = best_sep.1;
    manifold.local_n2 = pos21 * -best_sep.1;
    manifold.kinematics.category = KinematicsCategory::PlanePoint;
    manifold.kinematics.radius1 = 0.0;
    manifold.kinematics.radius2 = 0.0;

    // Transfer impulses.
    super::match_contacts(manifold, &old_manifold_points, swapped);
}
