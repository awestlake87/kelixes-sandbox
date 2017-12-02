
use std::collections::{ HashMap };
use std::ops;
use std::rc::Rc;

use cortical;
use rand::random;
use sc2;
use super::{ KeliConstraint, KeliData };

/// a resource budget for a lobe
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Budget {
    /// the minerals this lobe is allowed to use
    pub minerals:           u32,
    /// the vespene this lobe is allowed to use
    pub vespene:            u32,
    /// the supply slots this lobe is allowed to use
    pub food:               u32,
    /// the total larva this lobe is allowed to use
    pub larva:              u32,
}

impl Default for Budget {
    fn default() -> Self {
        Self {
            minerals: 0,
            vespene: 0,
            food: 0,
            larva: 0
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct LobeBudget {
    pub lobe:               cortical::NodeHdl,
    pub budget:             Budget,
}

impl LobeBudget {
    pub fn is_zero(&self) -> bool {
        self.budget == Budget::default()
    }
}

impl ops::AddAssign for Budget {
    fn add_assign(&mut self, rhs: Budget) {
        *self = *self + rhs;
    }
}
impl ops::SubAssign for Budget {
    fn sub_assign(&mut self, rhs: Budget) {
        *self = *self - rhs;
    }
}
impl ops::MulAssign<u32> for Budget {
    fn mul_assign(&mut self, rhs: u32) {
        *self = *self * rhs;
    }
}
impl ops::DivAssign<u32> for Budget {
    fn div_assign(&mut self, rhs: u32) {
        *self = *self / rhs;
    }
}

impl ops::Add for Budget {
    type Output = Budget;

    fn add(self, rhs: Budget) -> Budget {
        Budget {
            minerals: self.minerals + rhs.minerals,
            vespene: self.vespene + rhs.vespene,
            food: self.food + rhs.food,
            larva: self.larva + rhs.larva
        }
    }
}
impl ops::Sub for Budget {
    type Output = Budget;

    fn sub(self, rhs: Budget) -> Budget {
        Budget {
            minerals: self.minerals - rhs.minerals,
            vespene: self.vespene - rhs.vespene,
            food: self.food - rhs.food,
            larva: self.larva - rhs.larva,
        }
    }
}
impl ops::Mul<u32> for Budget {
    type Output = Budget;

    fn mul(self, rhs: u32) -> Budget {
        Budget {
            minerals: self.minerals * rhs,
            vespene: self.vespene * rhs,
            food: self.food * rhs,
            larva: self.larva * rhs,
        }
    }
}
impl ops::Div<u32> for Budget {
    type Output = Budget;

    fn div(self, rhs: u32) -> Budget {
        Budget {
            minerals: self.minerals / rhs,
            vespene: self.vespene / rhs,
            food: self.food / rhs,
            larva: self.larva / rhs,
        }
    }
}

/// sets all resources as the budget
pub struct WholeBudgetLobe {
    budget:             Budget
}

impl WholeBudgetLobe {
    pub fn new() -> Self {
        Self { budget: Budget::default() }
    }
}

create_lobe_data! {
    module: whole_budget,

    req frame: Rc<sc2::FrameData>,
    out budget: LobeBudget,
}

pub use self::whole_budget::{
    Input as WholeBudgetInput,
    Output as WholeBudgetOutput,
    FeedbackInput as WholeBudgetFeedbackInput,
    FeedbackOutput as WholeBudgetFeedbackOutput,
};

constrain_lobe! {
    lobe: WholeBudgetLobe,
    constraint: KeliConstraint,
    data: KeliData,

    input: WholeBudgetInput,
    output: WholeBudgetOutput,
    feedback_input: WholeBudgetFeedbackInput,
    feedback_output: WholeBudgetFeedbackOutput,

    req frame: FrameData,
    out budget: Budget,
}

impl cortical::Lobe for WholeBudgetLobe {
    type Input = WholeBudgetInput;
    type Output = WholeBudgetOutput;
    type FeedbackInput = WholeBudgetFeedbackInput;
    type FeedbackOutput = WholeBudgetFeedbackOutput;

    fn update(&mut self, input: Self::Input) -> cortical::Result<()> {
        let food = input.frame.state.food_cap - input.frame.state.food_used;

        self.budget = Budget {
            minerals: input.frame.state.minerals,
            vespene: input.frame.state.vespene,
            food: food,
            larva: input.frame.state.larva_count
        };

        Ok(())
    }

    fn tailor_output(&mut self, output: cortical::NodeHdl)
        -> cortical::Result<Self::Output>
    {
        Ok(
            WholeBudgetOutput {
                budget: LobeBudget { lobe: output, budget: self.budget }
            }
        )
    }

    fn tailor_feedback(&mut self, _: cortical::NodeHdl)
        -> cortical::Result<Self::FeedbackOutput>
    {
        Ok(WholeBudgetFeedbackOutput { })
    }
}

/// sets all resources as the budget
pub struct EvenSplitBudgetLobe {
    num_outputs:            usize,
    budget:                 Budget,
}

impl EvenSplitBudgetLobe {
    pub fn new() -> Self {
        Self { num_outputs: 0, budget: Budget::default() }
    }
}

create_lobe_data! {
    module: even_split_budget,

    req budget: LobeBudget,
    out split_budget: LobeBudget,
}

pub use self::even_split_budget::{
    Input as EvenSplitBudgetInput,
    Output as EvenSplitBudgetOutput,
    FeedbackInput as EvenSplitBudgetFeedbackInput,
    FeedbackOutput as EvenSplitBudgetFeedbackOutput,
};

constrain_lobe! {
    lobe: EvenSplitBudgetLobe,
    constraint: KeliConstraint,
    data: KeliData,

    input: EvenSplitBudgetInput,
    output: EvenSplitBudgetOutput,
    feedback_input: EvenSplitBudgetFeedbackInput,
    feedback_output: EvenSplitBudgetFeedbackOutput,

    req budget: Budget,
    out split_budget: Budget,
}

impl cortical::Lobe for EvenSplitBudgetLobe {
    type Input = EvenSplitBudgetInput;
    type Output = EvenSplitBudgetOutput;
    type FeedbackInput = EvenSplitBudgetFeedbackInput;
    type FeedbackOutput = EvenSplitBudgetFeedbackOutput;

    fn start(
        &mut self,
        _: cortical::NodeHdl,
        _: Vec<cortical::NodeHdl>,
        outputs: Vec<cortical::NodeHdl>
    )
        -> cortical::Result<()>
    {
        self.num_outputs = outputs.len();

        Ok(())
    }

    fn update(&mut self, input: Self::Input) -> cortical::Result<()> {
        self.budget = input.budget.budget;

        Ok(())
    }

    fn tailor_output(&mut self, output: cortical::NodeHdl)
        -> cortical::Result<Self::Output>
    {
        let factor = self.num_outputs as u32;

        Ok(
            EvenSplitBudgetOutput {
                split_budget: LobeBudget {
                    lobe: output,
                    budget: Budget {
                        minerals: self.budget.minerals / factor,
                        vespene: self.budget.vespene / factor,
                        food: self.budget.food / factor,
                        larva: self.budget.larva / factor,
                    }
                }
            }
        )
    }

    fn tailor_feedback(&mut self, _: cortical::NodeHdl)
        -> cortical::Result<Self::FeedbackOutput>
    {
        Ok(EvenSplitBudgetFeedbackOutput { })
    }
}

pub struct EvenSplitLedgerLobe {
    hdl: Option<cortical::NodeHdl>,

    total_spent: Budget,

    allotted: HashMap<cortical::NodeHdl, Budget>,
    spenders: HashMap<cortical::NodeHdl, Budget>,
    spent: HashMap<cortical::NodeHdl, Budget>,
}

create_lobe_data! {
    module: even_split_ledger,

    req allotted: LobeBudget,
    out split_budget: LobeBudget,

    fbk var each_spent: LobeBudget,
    fbk out spent: LobeBudget,
}

pub use self::even_split_ledger::{
    Input as EvenSplitLedgerInput,
    Output as EvenSplitLedgerOutput,
    FeedbackInput as EvenSplitLedgerFeedbackInput,
    FeedbackOutput as EvenSplitLedgerFeedbackOutput,
};

constrain_lobe! {
    lobe: EvenSplitLedgerLobe,
    constraint: KeliConstraint,
    data: KeliData,

    input: EvenSplitLedgerInput,
    output: EvenSplitLedgerOutput,
    feedback_input: EvenSplitLedgerFeedbackInput,
    feedback_output: EvenSplitLedgerFeedbackOutput,

    req allotted: Budget,
    out split_budget: Budget,

    fbk var each_spent: Budget,
    fbk out spent: Budget,
}

impl EvenSplitLedgerLobe {
    pub fn new() -> Self {
        Self {
            hdl: None,

            total_spent: Budget::default(),

            allotted: HashMap::new(),
            spenders: HashMap::new(),
            spent: HashMap::new(),
        }
    }
}

impl cortical::Lobe for EvenSplitLedgerLobe {
    type Input = EvenSplitLedgerInput;
    type Output = EvenSplitLedgerOutput;
    type FeedbackInput = EvenSplitLedgerFeedbackInput;
    type FeedbackOutput = EvenSplitLedgerFeedbackOutput;

    fn start(
        &mut self,
        hdl: cortical::NodeHdl,
        _: Vec<cortical::NodeHdl>,
        outputs: Vec<cortical::NodeHdl>
    )
        -> cortical::Result<()>
    {
        self.hdl = Some(hdl);

        self.spenders.clear();
        self.allotted.clear();
        self.spent.clear();

        for o in outputs {
            self.spenders.insert(o, Budget::default());
            self.allotted.insert(o, Budget::default());
            self.spent.insert(o, Budget::default());
        }

        Ok(())
    }

    fn update(&mut self, input: Self::Input) -> cortical::Result<()> {
        // zero all allotted budgets
        for (_, budget) in &mut self.allotted {
            *budget = Budget::default();
        }

        if input.allotted.is_zero() {
            return Ok(())
        }

        let total = self.total_spent + input.allotted.budget;
        let split = total / self.allotted.len() as u32;

        for (hdl, total_spent) in &mut self.spenders {
            let mut allotted = split;

            if allotted.minerals > total_spent.minerals {
                allotted.minerals -= total_spent.minerals;
            }
            else {
                allotted.minerals = 0;
            }

            if allotted.vespene > total_spent.vespene {
                allotted.vespene -= total_spent.vespene;
            }
            else {
                allotted.vespene = 0;
            }

            if allotted.food > total_spent.food {
                allotted.food -= total_spent.food;
            }
            else {
                allotted.food = 0;
            }

            if allotted.larva > total_spent.larva {
                allotted.larva -= total_spent.larva;
            }
            else {
                allotted.larva = 0;
            }

            *self.allotted.get_mut(&hdl).unwrap() = allotted;
        }

        let remaining = total - split * self.allotted.len() as u32;

        // just distribute the remaining budget to a random output
        let n = random::<usize>() % self.allotted.len();
        *self.allotted.values_mut().nth(n).unwrap() += remaining;

        Ok(())
    }

    fn tailor_output(&mut self, output: cortical::NodeHdl)
        -> cortical::Result<Self::Output>
    {
        Ok(
            EvenSplitLedgerOutput {
                split_budget: LobeBudget {
                    lobe: self.hdl.unwrap(),
                    budget: self.allotted[&output],
                }
            }
        )
    }

    fn feedback(&mut self, input: Self::FeedbackInput) -> cortical::Result<()>
    {
        if input.each_spent.len() != self.allotted.len() {
            bail!("did not receive feedback from all outputs")
        }

        for spent in input.each_spent {
            self.total_spent += spent.budget;
            *self.spenders.get_mut(&spent.lobe).unwrap() += spent.budget;
            *self.spent.get_mut(&spent.lobe).unwrap() = spent.budget;
        }

        Ok(())
    }

    fn tailor_feedback(&mut self, _: cortical::NodeHdl)
        -> cortical::Result<Self::FeedbackOutput>
    {
        Ok(
            EvenSplitLedgerFeedbackOutput {
                spent: LobeBudget {
                    lobe: self.hdl.unwrap(),
                    budget: self.spent.values().fold(
                        Budget::default(),
                        |acc, spent| acc + *spent
                    )
                }
            }
        )
    }
}
