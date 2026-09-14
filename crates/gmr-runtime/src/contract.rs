pub use gmr_core::{
    Binding, Claim, Derivation, Expr, FactAddress, LinkKind, Observes, Openness, Ref, SaidId,
    Source, Verifiability, Version,
};

pub use gmr_content::ContentErrorCode;

pub use crate::bind::Landed;
pub use crate::cite::{Cited, Drifted, Footprint, Look, Rests, Stands, Upheld, Why};
pub use crate::edges::{Edge, Edges, Raised};
pub use crate::link::{Inbound, Links, Reached};
pub use crate::open::{OpenRequest, Opened, Supersede};
pub use crate::read::{
    AnchorView, Anchored, Asked, Before, Blind, Depends, Evidence, Footing, Grounded, Grounding,
    Holding, Instructions, Knowledge, Linked, MemoryView, SaidView, Sample, Shown, Standing,
    Warrant,
};

pub const CONTRACT: &str = "gmr.contract.v13.0";

pub const SHAPE: &str = "sha256:e4d88f2aaaef5aff7dedd6cd2e7720840033f75ddc3154700c21fc1191b23ea7";
