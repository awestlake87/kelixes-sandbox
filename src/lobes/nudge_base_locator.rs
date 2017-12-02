
use std::rc::Rc;

use cortical;
use na::{ normalize, distance_squared };
use sc2;
use sc2::data::{ Point2, Vector2, Point3 };
use tantrum::{ ResourceCluster };

use super::{ KeliConstraint, KeliData };

/// finds base locations using an iterative algorithm
///
/// nudges location little by little until distance reaches a threshold
pub struct NudgeBaseLocatorLobe {
    locations:              Option<Rc<Vec<Point2>>>,
    debug_commands:         Option<Vec<sc2::Command>>,
    debug:                  bool
}

create_lobe_data! {
    module: nudge_base_locator,

    req clusters: Rc<Vec<ResourceCluster>>,

    out locations: Rc<Vec<Point2>>,
    out debug_commands: Vec<sc2::Command>,
}

pub use self::nudge_base_locator::{
    Input as NudgeBaseLocatorInput,
    Output as NudgeBaseLocatorOutput,
    FeedbackInput as NudgeBaseLocatorFeedbackInput,
    FeedbackOutput as NudgeBaseLocatorFeedbackOutput,
};

constrain_lobe! {
    lobe: NudgeBaseLocatorLobe,
    constraint: KeliConstraint,
    data: KeliData,

    input: NudgeBaseLocatorInput,
    output: NudgeBaseLocatorOutput,
    feedback_input: NudgeBaseLocatorFeedbackInput,
    feedback_output: NudgeBaseLocatorFeedbackOutput,

    req clusters: Resources,

    out locations: PotentialBaseLocations,
    out debug_commands: Commands,
}

impl NudgeBaseLocatorLobe {
    pub fn new() -> Self {
        Self {
            locations: Some(Rc::from(vec![ ])),
            debug_commands: Some(vec![ ]),
            debug: false
        }
    }

    pub fn with_debug() -> Self {
        Self {
            locations: Some(Rc::from(vec![ ])),
            debug_commands: Some(vec![ ]),
            debug: true
        }
    }

    fn find_closest_resource(&self, location: Point2, resources: &Vec<Point2>)
        -> Point2
    {
        assert!(resources.len() != 0);

        let mut min = distance_squared(&location, &resources[0]);
        let mut closest = resources[0];

        for r in resources.iter().skip(1) {
            let d = distance_squared(&location, &r);

            if d < min {
                min = d;
                closest = *r;
            }
        }

        closest
    }

    fn find_base_location(&mut self, cluster: &ResourceCluster) -> Point2 {
        let resources: Vec<Point2> = cluster.resources.iter().map(
            |r| Point2::new(r.pos.x, r.pos.y)
        ).collect();

        // initialize location as the center of mass
        let mut location = Point2::from_coordinates(
            resources.iter().fold(
                Vector2::zeros(), |acc, &r| acc + r.coords
            ) / resources.len() as f32
        );

        const MAX_ITERATIONS: usize = 10;
        const DESIRED: f32 = 37.0;
        const NUDGE_FACTOR: f32 = 4.0;

        for i in 0..MAX_ITERATIONS {
            // find the closest resource and nudge the location away from it
            let closest = self.find_closest_resource(location, &resources);

            let dist = distance_squared(&location, &closest);

            if self.debug {
                let g =
                    ((i as f32 + 1.0) / (MAX_ITERATIONS as f32) * 127.0) as u8
                    + 128
                ;

                let mut commands = vec![ ];

                commands.push(
                    sc2::Command::DebugSphere {
                        center: Point3::new(
                            location.x, location.y, cluster.resources[0].pos.z
                        ),
                        radius: 2.0,
                        color: (g, g, g)
                    }
                );
                commands.push(
                    sc2::Command::DebugText {
                        text: dist.to_string(),
                        color: (g, g, g),
                        target: Some(
                            sc2::DebugTextTarget::World(
                                Point3::new(
                                    location.x,
                                    location.y,
                                    cluster.resources[0].pos.z
                                )
                            )
                        )
                    }
                );

                self.debug_commands = Some(commands);
            }

            let direction = normalize(&(location.coords - closest.coords));

            if dist >= DESIRED {
                break
            }
            else {
                let nudge = ((DESIRED - dist) / DESIRED) * NUDGE_FACTOR;
                location += nudge * direction;
            }
        }

        location
    }
}

impl cortical::Lobe for NudgeBaseLocatorLobe {
    type Input = NudgeBaseLocatorInput;
    type Output = NudgeBaseLocatorOutput;
    type FeedbackInput = NudgeBaseLocatorFeedbackInput;
    type FeedbackOutput = NudgeBaseLocatorFeedbackOutput;

    fn update(&mut self, input: Self::Input) -> cortical::Result<()> {
        let mut locations = vec![ ];

        for cluster in input.clusters.iter() {
            locations.push(self.find_base_location(cluster));
        }

        self.locations = Some(Rc::from(locations));

        Ok(())
    }

    fn tailor_output(&mut self, _: cortical::NodeHdl)
        -> cortical::Result<Self::Output>
    {
        Ok(
            NudgeBaseLocatorOutput {
                locations: Rc::clone(self.locations.as_ref().unwrap()),
                debug_commands: self.debug_commands.as_ref().unwrap().clone()
            }
        )
    }

    fn tailor_feedback(&mut self, _: cortical::NodeHdl)
        -> cortical::Result<Self::FeedbackOutput>
    {
        Ok(NudgeBaseLocatorFeedbackOutput { })
    }
}
