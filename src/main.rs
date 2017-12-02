
extern crate ctrlc;
extern crate docopt;
extern crate sandbox;
extern crate sc2;
extern crate tantrum;

#[macro_use]
extern crate cortical;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate serde_derive;

mod args;
mod errors;

use std::sync::Arc;
use std::sync::atomic::{ AtomicBool, Ordering };

use cortical::{ CortexBuilder };
use docopt::Docopt;
use sc2::{ Coordinator, User };
use sc2::data::{ UnitType, PlayerSetup, Difficulty, Race };
use tantrum::{
    ResourceLobe,
    CommandMergerLobe,
    FrameForwarderLobe,
};

use sandbox::{
    create_keli_bot,
    KeliCortex,
    KeliConstraint,
    NudgeBaseLocatorLobe,
    WholeBudgetLobe,
    EvenSplitLedgerLobe,
    RandomDroneMorpherLobe,
    DebugWindowLobe,
};

use args::{
    USAGE, VERSION, get_coordinator_settings, get_game_settings, Args
};
use errors::{ Result };

quick_main!(|| -> Result<()> {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit())
    ;

    if args.flag_version {
        println!("kelixes-sandbox version {}", VERSION);
        return Ok(())
    }

    let coordinator_settings = get_coordinator_settings(&args)?;
    let game_settings = get_game_settings(&args)?;

    let mut coordinator = Coordinator::from_settings(coordinator_settings)?;

    let player = PlayerSetup::Player { race: Race::Zerg };
    let cpu = PlayerSetup::Computer {
        race: Race::Terran, difficulty: Difficulty::VeryEasy
    };

    let mut keli_builder = CortexBuilder::new();

    let frame_forwarder_lobe = keli_builder.add_node(
        Box::new(FrameForwarderLobe::new())
    );

    let debug_window_lobe = keli_builder.add_node(
        Box::new(DebugWindowLobe::new())
    );
    let whole_budget_lobe = keli_builder.add_node(
        Box::new(WholeBudgetLobe::new())
    );
    let even_split_ledger_lobe = keli_builder.add_node(
        Box::new(EvenSplitLedgerLobe::new())
    );
    let resource_lobe = keli_builder.add_node(
        Box::new(ResourceLobe::with_debug())
    );
    let base_locator_lobe = keli_builder.add_node(
        Box::new(NudgeBaseLocatorLobe::with_debug())
    );

    let spawning_pool_morpher_lobe = keli_builder.add_node(
        Box::new(
            RandomDroneMorpherLobe::one_and_done(UnitType::ZergSpawningPool)
        )
    );
    let evolution_chamber_morpher_lobe = keli_builder.add_node(
        Box::new(
            RandomDroneMorpherLobe::one_and_done(
                UnitType::ZergEvolutionChamber
            )
        )
    );

    let command_merger_lobe = keli_builder.add_node(
        Box::new(CommandMergerLobe::new())
    );


    keli_builder.connect(
        frame_forwarder_lobe,
        debug_window_lobe,
        vec![ KeliConstraint::FrameData ]
    )?;
    keli_builder.connect(
        frame_forwarder_lobe,
        whole_budget_lobe,
        vec![ KeliConstraint::FrameData ]
    )?;
    keli_builder.connect(
        frame_forwarder_lobe,
        spawning_pool_morpher_lobe,
        vec![ KeliConstraint::FrameData ]
    )?;
    keli_builder.connect(
        frame_forwarder_lobe,
        evolution_chamber_morpher_lobe,
        vec![ KeliConstraint::FrameData ]
    )?;
    keli_builder.connect(
        frame_forwarder_lobe,
        resource_lobe,
        vec![ KeliConstraint::FrameData ]
    )?;

    keli_builder.connect(
        whole_budget_lobe,
        even_split_ledger_lobe,
        vec![ KeliConstraint::Budget ]
    )?;

    keli_builder.connect(
        even_split_ledger_lobe,
        spawning_pool_morpher_lobe,
        vec![ KeliConstraint::Budget ]
    )?;
    keli_builder.feedback(
        spawning_pool_morpher_lobe,
        even_split_ledger_lobe,
        vec![ KeliConstraint::Budget ]
    )?;

    keli_builder.connect(
        even_split_ledger_lobe,
        evolution_chamber_morpher_lobe,
        vec![ KeliConstraint::Budget ]
    )?;
    keli_builder.feedback(
        evolution_chamber_morpher_lobe,
        even_split_ledger_lobe,
        vec![ KeliConstraint::Budget ]
    )?;

    keli_builder.connect(
        resource_lobe,
        base_locator_lobe,
        vec![ KeliConstraint::Resources ]
    )?;

    keli_builder.connect(
        spawning_pool_morpher_lobe,
        command_merger_lobe,
        vec![ KeliConstraint::Commands ]
    )?;
    keli_builder.connect(
        evolution_chamber_morpher_lobe,
        command_merger_lobe,
        vec![ KeliConstraint::Commands ]
    )?;
    keli_builder.connect(
        resource_lobe,
        command_merger_lobe,
        vec![ KeliConstraint::Commands ]
    )?;
    keli_builder.connect(
        base_locator_lobe,
        command_merger_lobe,
        vec![ KeliConstraint::Commands ]
    )?;

    keli_builder.set_input(frame_forwarder_lobe);
    keli_builder.set_output(command_merger_lobe);

    let bot = create_keli_bot(KeliCortex(keli_builder.build()?))?;

    coordinator.launch_starcraft(
        vec![
            (player, Some(User::Agent(Box::new(bot)))),
            (cpu, None)
        ]
    )?;

    println!("launched!");

    // intercept CTRL-C and SIGTERM so that sub-process can shutdown gracefully
    // this is mainly for Wine, because I think the linux headless exe and
    // windows both shutdown the game instance cleanly upon CTRL-C
    let done = Arc::new(AtomicBool::new(false));
    let ctrlc_done = done.clone();

    ctrlc::set_handler(move || ctrlc_done.store(true, Ordering::SeqCst))?;

    coordinator.start_game(game_settings)?;

    if done.load(Ordering::SeqCst) {
        return Ok(())
    }

    println!("game started!");

    while !done.load(Ordering::SeqCst) {
         match coordinator.update() {
             Ok(_) => (),
             Err(e) => {
                 eprintln!("update failed: {}", e);
                 break
             }
         };
    }

    Ok(())
});
