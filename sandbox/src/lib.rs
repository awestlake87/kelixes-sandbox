#![warn(missing_docs)]

extern crate futures;
extern crate gdk_pixbuf;
extern crate gtk;
extern crate nalgebra as na;
extern crate rand;
extern crate relm;
extern crate sc2;
extern crate tantrum;

#[macro_use]
extern crate cortical;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate relm_derive;
#[macro_use]
extern crate serde_derive;

mod budgeters;
mod debug_window;
mod errors;
mod drone_morphers;
mod nudge_base_locator;

use cortical::{ CortexBuilder };
use tantrum::{
    BotCortex,
    BotConstraint,
    BotData,

    ResourceCluster,
    ResourceLobe,
    ResourceInput,
    ResourceOutput,
    ResourceFeedbackInput,
    ResourceFeedbackOutput,

    FrameForwarderLobe,
    FrameForwarderInput,
    FrameForwarderOutput,
    FrameForwarderFeedbackInput,
    FrameForwarderFeedbackOutput,

    CommandMergerLobe,
    CommandMergerInput,
    CommandMergerOutput,
    CommandMergerFeedbackInput,
    CommandMergerFeedbackOutput,
};

pub use budgeters::*;
pub use debug_window::*;
pub use errors::*;
pub use drone_morphers::*;
pub use nudge_base_locator::*;

create_cortex! {
    module: keli_cortex,
    constraints: {
        FrameData:                  Rc<sc2::FrameData>,
        Resources:                  Rc<Vec<ResourceCluster>>,
        PotentialBaseLocations:     Rc<Vec<sc2::data::Point2>>,
        Budget:                     LobeBudget,
        Commands:                   Vec<sc2::Command>
    },
    input: FrameData,
    output: Resources
}

pub use self::keli_cortex::{
    Cortex as KeliCortex, Constraint as KeliConstraint, Data as KeliData
};

pub fn create_keli_bot(cortex: KeliCortex) -> Result<BotCortex> {
    let mut bot_builder = CortexBuilder::new();

    let keli_lobe = bot_builder.add_node(Box::new(cortex));

    bot_builder.set_input(keli_lobe);
    bot_builder.set_output(keli_lobe);

    Ok(BotCortex(bot_builder.build()?))
}

constrain_cortex! {
    cortex: KeliCortex,

    inner_constraint: KeliConstraint,
    inner_data: KeliData,

    outer_constraint: BotConstraint,
    outer_data: BotData,

    mapping: {
        FrameData => FrameData,
        Resources => ResourceClusters,
        PotentialBaseLocations => PotentialBaseLocations,
        Commands => Commands
    }
}

constrain_lobe! {
    lobe: ResourceLobe,
    constraint: KeliConstraint,
    data: KeliData,

    input: ResourceInput,
    output: ResourceOutput,
    feedback_input: ResourceFeedbackInput,
    feedback_output: ResourceFeedbackOutput,

    req frame: FrameData,

    out clusters: Resources,
    out debug_commands: Commands,
}

constrain_lobe! {
    lobe: FrameForwarderLobe,
    constraint: KeliConstraint,
    data: KeliData,

    input: FrameForwarderInput,
    output: FrameForwarderOutput,
    feedback_input: FrameForwarderFeedbackInput,
    feedback_output: FrameForwarderFeedbackOutput,

    req frame: FrameData,
    out frame: FrameData,
}

constrain_lobe! {
    lobe: CommandMergerLobe,
    constraint: KeliConstraint,
    data: KeliData,

    input: CommandMergerInput,
    output: CommandMergerOutput,
    feedback_input: CommandMergerFeedbackInput,
    feedback_output: CommandMergerFeedbackOutput,

    var commands: Commands,
    out commands: Commands,
}
