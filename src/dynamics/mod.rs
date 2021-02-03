//! Structures related to dynamics: bodies, joints, etc.

pub use self::integration_parameters::IntegrationParameters;
pub(crate) use self::joint::JointIndex;
#[cfg(feature = "dim3")]
pub use self::joint::RevoluteJoint;
pub use self::joint::{
    BallJoint, FixedJoint, Joint, JointHandle, JointParams, JointSet, PrismaticJoint,
};
pub use self::rigid_body::{ActivationStatus, BodyStatus, RigidBody, RigidBodyBuilder};
pub use self::rigid_body_set::{BodyPair, RigidBodyHandle, RigidBodySet};
pub use parry::mass_properties::MassProperties;
// #[cfg(not(feature = "parallel"))]
pub use self::coefficient_combine_rule::CoefficientCombineRule;
pub use self::island_set::{Island, IslandSet};

pub(crate) use self::joint::JointGraphEdge;
pub(crate) use self::rigid_body::RigidBodyChanges;
#[cfg(not(feature = "parallel"))]
pub(crate) use self::solver::IslandSolver;
#[cfg(feature = "parallel")]
pub(crate) use self::solver::ParallelIslandSolver;

mod coefficient_combine_rule;
mod integration_parameters;
mod island_set;
// mod island_set2;
mod joint;
mod rigid_body;
mod rigid_body_set;
mod solver;
