

pub mod test_scenarios;


use game::World;
pub trait Scenario {
    fn initialize_scenario_world(&self) -> World;
}