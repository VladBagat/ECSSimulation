use bevy::state::state::States;

//Defines the current group of actions performed by player
#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameControlState {
    Default,
    Building,
}