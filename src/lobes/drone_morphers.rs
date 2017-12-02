use std::rc::Rc;

use cortical;
use rand::random;
use sc2;
use sc2::data::{
    UnitType, UnitTypeData, Point2, Vector2, ActionTarget
};

use super::{ Budget, LobeBudget, KeliConstraint, KeliData };

pub struct RandomDroneMorpherLobe {
    hdl:            Option<cortical::NodeHdl>,

    unit_type:      UnitType,
    data:           Option<Rc<UnitTypeData>>,
    one_and_done:   bool,

    spent:          Budget,

    commands:       Vec<sc2::Command>
}

impl RandomDroneMorpherLobe {
    pub fn new(unit_type: UnitType) -> Self {
        Self {
            hdl: None,

            unit_type: unit_type,
            data: None,
            one_and_done: false,

            spent: Budget::default(),

            commands: vec![ ]
        }
    }

    pub fn one_and_done(unit_type: UnitType) -> Self {
        Self {
            hdl: None,

            unit_type: unit_type,
            data: None,
            one_and_done: true,

            spent: Budget::default(),

            commands: vec![ ]
        }
    }

    fn morph_drone(
        &self, input: &RandomDroneMorpherInput, data: &UnitTypeData
    )
        -> Option<sc2::Command>
    {
        if self.one_and_done {
            let existing = input.frame.state.filter_units(
                |u| u.unit_type == self.unit_type
            );

            // only allow one to be built at a time
            if existing.len() >= 1 {
                return None
            }
        }

        let budget = {
            if let Some(ref budget) = input.budget {
                budget.budget
            }
            else {
                return None
            }
        };

        if data.mineral_cost > budget.minerals
            || data.vespene_cost > budget.vespene
            || data.food_required > budget.food as f32
        {
            return None
        }

        let drones = input.frame.state.filter_units(
            |u| u.unit_type == UnitType::ZergDrone
        );
        let hatcheries = input.frame.state.filter_units(
            |u| u.unit_type == UnitType::ZergHatchery
        );

        if drones.len() < 1 {
            return None
        }

        if hatcheries.len() < 1 {
            return None
        }

        let h = random::<usize>() % hatcheries.len();
        let mut location = Point2::new(
            hatcheries[h].pos.x, hatcheries[h].pos.y
        );

        location += 10.0 * Vector2::new(
            random::<f32>() - 0.5, random::<f32>() - 0.5
        );

        Some(
            sc2::Command::Action {
                units: vec![
                    Rc::clone(&drones[random::<usize>() % drones.len()])
                ],
                ability: data.ability,
                target: Some(ActionTarget::Location(location))
            }
        )
    }
}

create_lobe_data! {
    module: random_drone_morpher,

    req frame: Rc<sc2::FrameData>,
    opt budget: LobeBudget,

    out commands: Vec<sc2::Command>,

    fbk out spent: LobeBudget,
}

pub use self::random_drone_morpher::{
    Input as RandomDroneMorpherInput,
    Output as RandomDroneMorpherOutput,
    FeedbackInput as RandomDroneMorpherFeedbackInput,
    FeedbackOutput as RandomDroneMorpherFeedbackOutput,
};

constrain_lobe! {
    lobe: RandomDroneMorpherLobe,
    constraint: KeliConstraint,
    data: KeliData,

    input: RandomDroneMorpherInput,
    output: RandomDroneMorpherOutput,
    feedback_input: RandomDroneMorpherFeedbackInput,
    feedback_output: RandomDroneMorpherFeedbackOutput,

    req frame: FrameData,
    opt budget: Budget,

    out commands: Commands,

    fbk out spent: Budget,
}

impl cortical::Lobe for RandomDroneMorpherLobe {
    type Input = RandomDroneMorpherInput;
    type Output = RandomDroneMorpherOutput;
    type FeedbackInput = RandomDroneMorpherFeedbackInput;
    type FeedbackOutput = RandomDroneMorpherFeedbackOutput;

    fn start(
        &mut self,
        hdl: cortical::NodeHdl,
        _: Vec<cortical::NodeHdl>,
        _: Vec<cortical::NodeHdl>
    )
        -> cortical::Result<()>
    {
        self.hdl = Some(hdl);

        Ok(())
    }

    fn update(&mut self, input: Self::Input) -> cortical::Result<()> {
        self.spent = Budget::default();

        if self.data.is_none() {
            if let Some(ref data) = input.frame.data.unit_type_data.get(
                &self.unit_type
            ) {
                self.data = Some(Rc::clone(data));
            }
        }

        let mut commands = vec![ ];

        if let Some(ref data) = self.data {
            if let Some(command) = self.morph_drone(&input, &data) {
                commands.push(command);

                self.spent = Budget {
                    minerals: data.mineral_cost,
                    vespene: data.vespene_cost,
                    food: data.food_required.ceil() as u32,

                    ..Budget::default()
                }
            }
        }
        else {
            bail!("unable to get UnitTypeData for {:?}", self.unit_type);
        }

        self.commands = commands;

        Ok(())
    }

    fn tailor_output(&mut self, _: cortical::NodeHdl)
        -> cortical::Result<Self::Output>
    {
        Ok(
            RandomDroneMorpherOutput {
                commands: self.commands.clone()
            }
        )
    }

    fn tailor_feedback(&mut self, _: cortical::NodeHdl)
        -> cortical::Result<Self::FeedbackOutput>
    {
        Ok(
            RandomDroneMorpherFeedbackOutput {
                spent: LobeBudget {
                    lobe: self.hdl.unwrap(),
                    budget: self.spent,
                }
            }
        )
    }
}
