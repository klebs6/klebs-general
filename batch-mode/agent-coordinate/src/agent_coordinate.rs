// ---------------- [ File: src/agent_coordinate.rs ]
crate::ix!();

#[derive(Clone,Getters,Debug)]
#[getset(get="pub")]
pub struct AgentCoordinate {
    location: Option<String>,
    goal:     Option<String>,
}

impl AgentCoordinate {

    pub fn here_without_a_goal(location: &str) -> Self {
        Self {
            location: Some(location.to_string()),
            goal:     None,
        }
    }

    pub fn here_with_a_goal(location: &str, goal: &str) -> Self {
        Self {
            location: Some(location.to_string()),
            goal:     Some(goal.to_string()),
        }
    }

    pub fn nowhere_with_no_goal() -> Self {
        Self {
            location: None,
            goal:     None,
        }
    }

    pub fn nowhere_with_a_goal(goal: &str) -> Self {
        Self {
            location: None,
            goal:     Some(goal.to_string()),
        }
    }
}

impl std::fmt::Display for AgentCoordinate {

    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        if self.location.is_none() && self.goal.is_none() {
            return write!(out,"Neither your goal nor your location are specified by the user.");
        }

        let mut x = format!("");

        if let Some(place) = &self.location {
            x.push_str(&format!("You are here: {}", place));
        }

        if let Some(goal) = &self.goal {
            x.push_str(&goal);
        }

        write!(out,"{}",x)
    }
}
