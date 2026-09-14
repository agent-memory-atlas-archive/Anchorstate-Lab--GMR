pub use gmr_core::{
    Binding, Claim, Derivation, Expr, FactAddress, Footprint, LinkKind, Observes, Openness, Ref,
    Rests, SaidId, Source, Verifiability, Version,
};

pub use gmr_content::ContentErrorCode;

pub use crate::bind::Landed;
pub use crate::cite::{Cited, Drifted, Look, Stands, Upheld, Why};
pub use crate::edges::{Edge, Edges, Raised};
pub use crate::link::{Inbound, Links, Reached};
pub use crate::open::{OpenRequest, Opened, Supersede};
pub use crate::read::{
    AnchorView, Anchored, Asked, Before, Blind, Depends, Evidence, Footing, Grounded, Grounding,
    Holding, Instructions, Knowledge, Linked, MemoryView, SaidView, Sample, Shown, Standing,
    Warrant,
};

pub const CONTRACT: &str = "gmr.contract.v13.0";

pub const SHAPE: &str = "sha256:5fbd7b85ac3ee573c841aea74dc4517a56f198274e96cbe05b1fe4ecf07785d6";
